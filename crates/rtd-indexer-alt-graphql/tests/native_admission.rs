// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Read-only runtime acceptance against an explicitly supplied private fixture.
//! The fixture owns only new listeners/processes; existing databases are never mutated.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

use reqwest::Client;
use rtd_rpc::proto::rtd::rpc::v2 as grpc;
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio::task::JoinSet;
use tonic::transport::Certificate;
use tonic::transport::ClientTlsConfig;
use tonic::transport::Endpoint;
use tonic::transport::Identity;

#[path = "native_admission/business.rs"]
mod business;
#[path = "native_admission/ingress.rs"]
mod ingress;
#[path = "native_admission/subscriptions.rs"]
mod subscriptions;

#[derive(Deserialize)]
struct ProcessSpec {
    binary: PathBuf,
    args: Vec<String>,
    env: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct TlsFiles {
    ca: PathBuf,
    cert: PathBuf,
    key: PathBuf,
}

#[derive(Deserialize)]
struct ProxySpec {
    listen: String,
    target: String,
}

#[derive(Deserialize)]
struct Fixture {
    root: PathBuf,
    chain: String,
    history_height: u64,
    graphql_url: String,
    baseline_graphql_url: String,
    ledger_url: String,
    guard_url: String,
    ledger_tls: TlsFiles,
    guard_tls: TlsFiles,
    postgres: ProxySpec,
    hbase: ProxySpec,
    processes: BTreeMap<String, ProcessSpec>,
    ingress: Option<ingress::Ingress>,
    subscriptions: Option<ProxySpec>,
    subscription_event_checkpoint: Option<u64>,
    business: Option<business::Business>,
}

struct Children(Vec<Child>);

impl Children {
    fn start(&mut self, root: &std::path::Path, name: &str, spec: &ProcessSpec) {
        let log = fs::File::create(root.join(format!("{name}.log"))).unwrap();
        self.0.push(
            Command::new(&spec.binary)
                .args(&spec.args)
                .env_clear()
                .env("PATH", "/usr/bin:/bin")
                .env("TOKIO_WORKER_THREADS", "2")
                .env("RUST_LOG", "warn")
                .envs(&spec.env)
                .stdin(Stdio::null())
                .stdout(log.try_clone().unwrap())
                .stderr(log)
                .spawn()
                .expect("spawn the supplied native binary"),
        );
    }

    fn assert_running(&mut self) {
        for child in &mut self.0 {
            assert!(
                child.try_wait().unwrap().is_none(),
                "owned service exited; inspect private fixture logs"
            );
        }
    }

    fn stop(mut self) -> Vec<Value> {
        self.0
            .drain(..)
            .map(|mut child| {
                let pid = child.id();
                child.kill().expect("stop owned native service");
                let status = child.wait().expect("reap owned native service");
                json!({"pid":pid,"exit":status.to_string()})
            })
            .collect()
    }
}

impl Drop for Children {
    fn drop(&mut self) {
        for child in &mut self.0 {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

async fn proxy(spec: &ProxySpec) -> (watch::Sender<bool>, tokio::task::JoinHandle<()>) {
    assert!(spec.listen.starts_with("127.0.0.1:"));
    assert!(spec.target.starts_with("127.0.0.1:"));
    let listener = TcpListener::bind(&spec.listen).await.unwrap();
    let target = spec.target.clone();
    let (switch, mut state) = watch::channel(true);
    let task = tokio::spawn(async move {
        let mut connections = JoinSet::new();
        loop {
            tokio::select! {
                changed = state.changed() => {
                    if changed.is_err() { break; }
                    if !*state.borrow_and_update() { connections.abort_all(); }
                }
                accepted = listener.accept() => {
                    let (mut incoming, _) = accepted.unwrap();
                    if !*state.borrow() { continue; }
                    let target = target.clone();
                    connections.spawn(async move {
                        if let Ok(mut outgoing) = TcpStream::connect(target).await {
                            let _ = tokio::io::copy_bidirectional(&mut incoming, &mut outgoing).await;
                        }
                    });
                }
                _ = connections.join_next(), if !connections.is_empty() => {}
            }
        }
        connections.shutdown().await;
    });
    (switch, task)
}

fn http_client(tls: Option<&TlsFiles>) -> Client {
    let mut builder = Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(25));
    if let Some(tls) = tls {
        let mut identity = fs::read(&tls.cert).unwrap();
        identity.extend(fs::read(&tls.key).unwrap());
        builder = builder
            .add_root_certificate(
                reqwest::Certificate::from_pem(&fs::read(&tls.ca).unwrap()).unwrap(),
            )
            .identity(reqwest::Identity::from_pem(&identity).unwrap());
    }
    builder.build().unwrap()
}

async fn query(client: &Client, url: &str, query: &str, heights: &[&str]) -> Value {
    let mut request = client
        .post(url)
        .header("content-type", "application/json")
        .body(json!({"query": query}).to_string());
    for value in heights {
        request = request.header("x-rtd-visible-checkpoint", *value);
    }
    let response = request.send().await.unwrap();
    assert_eq!(response.status(), 200);
    serde_json::from_slice(&response.bytes().await.unwrap()).unwrap()
}

fn success(value: &Value) {
    assert!(value.get("errors").is_none(), "{value}");
    assert!(!value["data"].is_null(), "{value}");
}

fn denied(value: &Value) {
    assert!(
        value["errors"].as_array().is_some_and(|v| !v.is_empty()),
        "{value}"
    );
    assert!(value["data"].is_null(), "{value}");
}

fn field_denied(value: &Value, field: &str) {
    assert!(
        value["errors"].as_array().is_some_and(|v| !v.is_empty()),
        "{value}"
    );
    assert!(value["data"][field].is_null(), "{value}");
}

async fn guard(client: &Client, fixture: &Fixture, surface: &str) -> reqwest::Response {
    client
        .get(&fixture.guard_url)
        .header("x-rtd-read-surface", surface)
        .send()
        .await
        .unwrap()
}

async fn ready(client: &Client, fixture: &Fixture, children: &mut Children) -> Value {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
    while tokio::time::Instant::now() < deadline {
        children.assert_running();
        if let Ok(response) = client
            .post(&fixture.graphql_url)
            .header("content-type", "application/json")
            .body(r#"{"query":"{ chainIdentifier checkpoint { sequenceNumber } }"}"#)
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            if let Ok(bytes) = response.bytes().await {
                if let Ok(value) = serde_json::from_slice::<Value>(&bytes) {
                    if value.get("errors").is_none() && !value["data"]["checkpoint"].is_null() {
                        return value;
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    panic!("native services did not become ready; inspect private fixture logs");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit RTD_NATIVE_QUERY_FIXTURE and existing read-only backends"]
async fn real_native_query_admission_and_dependency_recovery() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let fixture: Fixture = serde_json::from_slice(
        &fs::read(std::env::var("RTD_NATIVE_QUERY_FIXTURE").unwrap()).unwrap(),
    )
    .unwrap();
    let graphql = &fixture.processes["graphql"];
    let secret = graphql
        .args
        .iter()
        .position(|arg| arg == "--database-url-file")
        .unwrap();
    let database = fs::read_to_string(&graphql.args[secret + 1]).unwrap();
    let database = url::Url::parse(database.trim()).unwrap();
    assert_eq!(database.host_str(), Some("127.0.0.1"));
    assert!(
        !database.query().unwrap().contains('+'),
        "PostgreSQL URL options require percent encoding, not form encoding"
    );
    let options: Vec<_> = database
        .query_pairs()
        .filter(|(name, _)| name == "options")
        .collect();
    assert_eq!(options.len(), 1);
    assert_eq!(options[0].1, "-c default_transaction_read_only=on");
    for (name, expected) in [
        ("graphql", "rtd-indexer-alt-graphql"),
        ("ledger", "rtd-kv-rpc"),
        ("guard", "rtd-archive-query-gateway"),
    ] {
        assert_eq!(
            fixture.processes[name].binary.file_name().unwrap(),
            expected
        );
    }
    let (postgres, pg_task) = proxy(&fixture.postgres).await;
    let (hbase, hbase_task) = proxy(&fixture.hbase).await;
    let ingress_runtime = if let Some(ingress) = &fixture.ingress {
        Some(ingress::Runtime::start(ingress).await)
    } else {
        None
    };
    let subscription_proxy = if let Some(spec) = &fixture.subscriptions {
        Some(proxy(spec).await)
    } else {
        None
    };
    let mut children = Children(Vec::new());
    let business_runtime = if let Some(business) = &fixture.business {
        Some(business::Runtime::start(&fixture, business).await)
    } else {
        None
    };
    if fixture.subscriptions.is_some() {
        let spec = &fixture.processes["subscription_guard"];
        assert_eq!(
            spec.binary.file_name().unwrap(),
            "rtd-archive-query-gateway"
        );
        assert!(
            spec.args
                .iter()
                .any(|arg| arg == "--subscription-authority")
        );
        children.start(&fixture.root, "subscription_guard", spec);
    }
    for name in ["ledger", "guard", "graphql"] {
        children.start(&fixture.root, name, &fixture.processes[name]);
    }
    let http = http_client(None);
    let authority = http_client(Some(&fixture.guard_tls));
    let initial = ready(&http, &fixture, &mut children).await;
    assert_eq!(initial["data"]["chainIdentifier"], fixture.chain);
    let mut checks = Vec::new();
    let mut height = 0;
    for surface in ["graphql", "ledger", "consistent"] {
        let response = guard(&authority, &fixture, surface).await;
        assert_eq!(response.status(), 204, "{surface} authority");
        let current: u64 = response.headers()["x-rtd-visible-checkpoint"]
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        assert!(current > 0);
        if surface == "graphql" {
            height = current;
        }
        assert!(response.bytes().await.unwrap().is_empty());
        checks.push(json!({"case":format!("native-authority-{surface}"),"height":current}));
    }
    let anonymous = Client::builder()
        .no_proxy()
        .add_root_certificate(
            reqwest::Certificate::from_pem(&fs::read(&fixture.guard_tls.ca).unwrap()).unwrap(),
        )
        .build()
        .unwrap();
    assert!(
        anonymous
            .get(&fixture.guard_url)
            .timeout(Duration::from_secs(3))
            .send()
            .await
            .is_err()
    );
    checks.push(json!({"case":"authority-requires-client-certificate"}));
    // Freeze a height already covered by both reads, even if indexing advances
    // while the independent authority requests are being verified.
    height = height.min(
        initial["data"]["checkpoint"]["sequenceNumber"]
            .as_u64()
            .unwrap(),
    );
    let head = height.to_string();
    let selection = "{ chainIdentifier checkpoint { sequenceNumber digest } }";
    let admitted = query(&http, &fixture.graphql_url, selection, &[&head]).await;
    success(&admitted);
    let checkpoint = admitted["data"]["checkpoint"]["sequenceNumber"]
        .as_u64()
        .unwrap();
    assert!(checkpoint <= height);
    assert!(fixture.history_height > 0 && fixture.history_height < height);
    let history = fixture.history_height.to_string();
    let historical = query(&http, &fixture.graphql_url, selection, &[&history]).await;
    success(&historical);
    assert_eq!(
        historical["data"]["checkpoint"]["sequenceNumber"],
        fixture.history_height
    );
    assert_ne!(
        historical["data"]["checkpoint"]["digest"],
        admitted["data"]["checkpoint"]["digest"]
    );
    checks.push(
        json!({"case":"real-checkpoint-summary-pins-request", "head":admitted,"historical":historical}),
    );
    for (checkpoint, expected) in [(fixture.history_height, &historical), (height, &admitted)] {
        let explicit = format!(
            "{{ chainIdentifier checkpoint(sequenceNumber:{checkpoint}) {{ sequenceNumber digest }} }}"
        );
        assert_eq!(
            query(&http, &fixture.baseline_graphql_url, &explicit, &[]).await,
            *expected
        );
    }
    checks.push(json!({"case":"existing-service-parity-at-two-fixed-heights"}));
    denied(&query(&http, &fixture.graphql_url, selection, &["0"]).await);
    checks.push(json!({"case":"below-common-retention-window-rejected","height":0,"consistentLower":fixture.history_height}));
    for values in [
        vec!["01"],
        vec!["-1"],
        vec!["18446744073709551616"],
        vec!["0", "0"],
    ] {
        denied(&query(&http, &fixture.graphql_url, selection, &values).await);
        checks.push(json!({"case":"invalid-or-duplicate-boundary","headers":values}));
    }
    assert_eq!(
        query(
            &http,
            &fixture.graphql_url,
            &format!(
                "{{ checkpoint(sequenceNumber:{}) {{ sequenceNumber }} }}",
                fixture.history_height + 1
            ),
            &[&history],
        )
        .await,
        json!({"data":{"checkpoint":null}}),
    );
    checks.push(json!({"case":"future-explicit-checkpoint-hidden"}));
    let future_page = query(&http, &fixture.graphql_url,
        &format!("{{ checkpoints(first:1,filter:{{atCheckpoint:{}}}) {{ nodes {{ sequenceNumber }} }} }}", fixture.history_height + 1),
        &[&history]).await;
    field_denied(&future_page, "checkpoints");
    assert_eq!(
        future_page["errors"][0]["extensions"]["code"],
        "BAD_USER_INPUT"
    );
    checks.push(json!({"case":"future-checkpoint-collection-rejected"}));
    let package = query(&http, &fixture.graphql_url,
        "{ package(address:\"0x2\",atCheckpoint:0) { address version } type(type:\"0x2::coin::Coin<0x2::rtd::RTD>\") { layout } }", &[&history]).await;
    success(&package);
    assert_eq!(package["data"]["package"]["version"], 1);
    assert!(!package["data"]["type"]["layout"].is_null());
    checks.push(json!({"case":"real-genesis-package-and-historical-layout"}));
    assert_eq!(
        query(&http, &fixture.graphql_url, selection, &[&head]).await,
        admitted
    );
    checks.push(json!({"case":"historical-request-does-not-poison-next-request"}));

    let tls = &fixture.ledger_tls;
    let endpoint = Endpoint::from_shared(fixture.ledger_url.clone())
        .unwrap()
        .timeout(Duration::from_secs(8))
        .tls_config(
            ClientTlsConfig::new()
                .ca_certificate(Certificate::from_pem(fs::read(&tls.ca).unwrap()))
                .identity(Identity::from_pem(
                    fs::read(&tls.cert).unwrap(),
                    fs::read(&tls.key).unwrap(),
                )),
        )
        .unwrap();
    let mut ledger = rtd_rpc::Client::from_endpoint(&endpoint).ledger_client();
    let mut request = tonic::Request::new(grpc::GetServiceInfoRequest::default());
    request
        .metadata_mut()
        .insert("x-rtd-visible-checkpoint", "0".parse().unwrap());
    let info = ledger.get_service_info(request).await.unwrap().into_inner();
    assert_eq!(info.checkpoint_height, Some(0));
    checks.push(json!({"case":"real-native-ledger-bounded-service-info"}));
    if let (Some(ingress), Some(runtime)) = (&fixture.ingress, &ingress_runtime) {
        checks.extend(ingress::verify_open(&fixture, ingress, runtime).await);
    }
    if let Some((authority, _)) = &subscription_proxy {
        checks.extend(subscriptions::verify(&fixture, authority).await);
    }
    if let (Some(business), Some(runtime)) = (&fixture.business, &business_runtime) {
        checks.extend(business::verify(&fixture, business, runtime).await);
    }

    postgres.send(false).unwrap();
    assert_eq!(guard(&authority, &fixture, "graphql").await.status(), 403);
    field_denied(
        &query(
            &http,
            &fixture.graphql_url,
            "{ package(address:\"0x2\",atCheckpoint:0) { address version } }",
            &[&history],
        )
        .await,
        "package",
    );
    checks.push(json!({"case":"postgres-unavailable-refuses-authority-and-query"}));
    if let (Some(ingress), Some(runtime)) = (&fixture.ingress, &ingress_runtime) {
        checks.push(ingress::verify_publication_closed(ingress, runtime).await);
    }
    postgres.send(true).unwrap();
    ready(&http, &fixture, &mut children).await;
    assert_eq!(guard(&authority, &fixture, "graphql").await.status(), 204);
    checks.push(json!({"case":"postgres-recovery-revalidated"}));
    if let Some(ingress) = &fixture.ingress {
        checks.push(ingress::verify_recovered(&fixture, ingress).await);
    }

    hbase.send(false).unwrap();
    tokio::time::sleep(Duration::from_millis(1500)).await;
    assert!(
        ledger
            .get_service_info(grpc::GetServiceInfoRequest::default())
            .await
            .is_err()
    );
    denied(&query(&http, &fixture.graphql_url, selection, &[&history]).await);
    checks.push(json!({"case":"hbase-unavailable-clears-ledger-success-and-refuses-query"}));
    if let Some(ingress) = &fixture.ingress {
        checks.push(ingress::verify_hbase_closed(ingress).await);
    }
    hbase.send(true).unwrap();
    ready(&http, &fixture, &mut children).await;
    assert_eq!(
        query(&http, &fixture.graphql_url, selection, &[&history]).await,
        historical
    );
    checks.push(json!({"case":"hbase-recovery-revalidated"}));
    if let Some(ingress) = &fixture.ingress {
        checks.push(ingress::verify_recovered(&fixture, ingress).await);
    }
    let exits = children.stop();
    if let Some(runtime) = business_runtime {
        runtime.stop().await;
    }
    if let Some(runtime) = ingress_runtime {
        runtime.stop().await;
    }
    if let Some((switch, task)) = subscription_proxy {
        drop(switch);
        task.await.unwrap();
    }
    drop(postgres);
    drop(hbase);
    pg_task.await.unwrap();
    hbase_task.await.unwrap();
    fs::write(fixture.root.join("runtime-acceptance.json"), serde_json::to_vec_pretty(&json!({
        "result":"passed", "canonicalMutated":false, "ownedProcessesReaped":true,
        "ownedProcessExits":exits,
        "scope":"native GraphQL/Ledger/authority with real read-only PG/HBase/publication backends; optional middleware ingress and real SSE",
        "ingressEnabled":fixture.ingress.is_some(), "subscriptionsEnabled":fixture.subscriptions.is_some(),
        "businessEnabled":fixture.business.is_some(), "checks":checks
    })).unwrap()).unwrap();
}
