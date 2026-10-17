// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;

use reqwest::Client;
use rtd_rpc::proto::rtd::rpc::v2 as grpc;
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::task::JoinSet;
use tonic::transport::Certificate;
use tonic::transport::ClientTlsConfig;
use tonic::transport::Endpoint;
use tonic::transport::Identity;

use super::Fixture;
use super::ProxySpec;
use super::TlsFiles;
use super::field_denied;
use super::query;
use super::success;

#[derive(Deserialize)]
pub(super) struct Ingress {
    pub graphql_url: String,
    pub ledger_url: String,
    pub client: TlsFiles,
    graphql_proxy: ProxySpec,
    ledger_proxy: ProxySpec,
}

struct Counter {
    bytes: Arc<AtomicU64>,
    stop: oneshot::Sender<()>,
    task: JoinHandle<()>,
}

pub(super) struct Runtime {
    graphql: Counter,
    ledger: Counter,
}

impl Counter {
    async fn start(spec: &ProxySpec) -> Self {
        assert!(spec.listen.starts_with("127.0.0.1:"));
        assert!(spec.target.starts_with("127.0.0.1:"));
        let listener = TcpListener::bind(&spec.listen).await.unwrap();
        let target = spec.target.clone();
        let bytes = Arc::new(AtomicU64::new(0));
        let observed = bytes.clone();
        let (stop, mut stopped) = oneshot::channel();
        let task = tokio::spawn(async move {
            let mut connections = JoinSet::new();
            loop {
                tokio::select! {
                    _ = &mut stopped => break,
                    accepted = listener.accept() => {
                        let (incoming, _) = accepted.unwrap();
                        let target = target.clone();
                        let observed = observed.clone();
                        connections.spawn(async move {
                            let Ok(outgoing) = TcpStream::connect(target).await else { return; };
                            let (mut client_read, mut client_write) = incoming.into_split();
                            let (mut backend_read, mut backend_write) = outgoing.into_split();
                            let upload = async {
                                let mut buffer = [0u8; 16384];
                                loop {
                                    let count = client_read.read(&mut buffer).await?;
                                    if count == 0 { break; }
                                    observed.fetch_add(count as u64, Ordering::SeqCst);
                                    backend_write.write_all(&buffer[..count]).await?;
                                }
                                backend_write.shutdown().await
                            };
                            let download = tokio::io::copy(&mut backend_read, &mut client_write);
                            let _ = tokio::try_join!(upload, download);
                        });
                    }
                    _ = connections.join_next(), if !connections.is_empty() => {}
                }
            }
            connections.shutdown().await;
        });
        Self { bytes, stop, task }
    }
}

impl Runtime {
    pub async fn start(ingress: &Ingress) -> Self {
        Self {
            graphql: Counter::start(&ingress.graphql_proxy).await,
            ledger: Counter::start(&ingress.ledger_proxy).await,
        }
    }

    pub async fn settled_bytes(&self) -> [u64; 2] {
        let read = || {
            [
                self.graphql.bytes.load(Ordering::SeqCst),
                self.ledger.bytes.load(Ordering::SeqCst),
            ]
        };
        for _ in 0..20 {
            let before = read();
            tokio::time::sleep(Duration::from_millis(50)).await;
            if read() == before {
                return before;
            }
        }
        panic!("ingress backend transport did not settle");
    }

    pub async fn stop(self) {
        let _ = self.graphql.stop.send(());
        let _ = self.ledger.stop.send(());
        self.graphql.task.await.unwrap();
        self.ledger.task.await.unwrap();
    }
}

fn public_http(ingress: &Ingress) -> Client {
    Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .add_root_certificate(
            reqwest::Certificate::from_pem(&fs::read(&ingress.client.ca).unwrap()).unwrap(),
        )
        .timeout(Duration::from_secs(25))
        .build()
        .unwrap()
}

fn ledger(ingress: &Ingress, authenticated: bool) -> rtd_rpc::Client {
    let mut tls = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(fs::read(&ingress.client.ca).unwrap()));
    if authenticated {
        tls = tls.identity(Identity::from_pem(
            fs::read(&ingress.client.cert).unwrap(),
            fs::read(&ingress.client.key).unwrap(),
        ));
    }
    let endpoint = Endpoint::from_shared(ingress.ledger_url.clone())
        .unwrap()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(25))
        .tls_config(tls)
        .unwrap();
    rtd_rpc::Client::from_endpoint(&endpoint)
}

pub(super) async fn verify_open(
    fixture: &Fixture,
    ingress: &Ingress,
    runtime: &Runtime,
) -> Vec<Value> {
    let http = public_http(ingress);
    let mut checks = Vec::new();
    let baseline = runtime.settled_bytes().await;
    let untrusted = Client::builder()
        .no_proxy()
        .tls_built_in_root_certs(false)
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    assert!(untrusted.get(&ingress.graphql_url).send().await.is_err());
    assert_eq!(runtime.settled_bytes().await, baseline);
    checks.push(json!({"case":"ingress-private-ca-required-before-backend"}));
    for headers in [
        vec![],
        vec!["0"],
        vec!["not-a-height"],
        vec!["0", "1"],
        vec!["18446744073709551615"],
    ] {
        let mut request = http
            .post(&ingress.graphql_url)
            .header("content-type", "application/json")
            .header("x-rtd-request-id", "untrusted-id")
            .header("x-rtd-read-surface", "invalid")
            .body(r#"{"query":"{ chainIdentifier checkpoint { sequenceNumber digest } }"}"#);
        for header in &headers {
            request = request.header("x-rtd-visible-checkpoint", *header);
        }
        let response = request.send().await.unwrap();
        assert_eq!(response.status(), 200);
        assert_ne!(response.headers()["x-rtd-request-id"], "untrusted-id");
        let boundary: u64 = response.headers()["x-rtd-visible-checkpoint"]
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        let value: Value = serde_json::from_slice(&response.bytes().await.unwrap()).unwrap();
        success(&value);
        assert_eq!(value["data"]["chainIdentifier"], fixture.chain);
        let checkpoint = value["data"]["checkpoint"]["sequenceNumber"]
            .as_u64()
            .unwrap();
        assert!(checkpoint > fixture.history_height && checkpoint <= boundary);
        checks.push(json!({"case":"ingress-graphql-overwrites-external-boundary","externalHeaders":headers,"authorityHeight":boundary,"servedHeight":checkpoint}));
    }
    let mut client = ledger(ingress, true).ledger_client();
    for headers in [
        vec![],
        vec!["0"],
        vec!["not-a-height"],
        vec!["0", "1"],
        vec!["18446744073709551615"],
    ] {
        let mut request = tonic::Request::new(grpc::GetServiceInfoRequest::default());
        for header in &headers {
            request
                .metadata_mut()
                .append("x-rtd-visible-checkpoint", header.parse().unwrap());
        }
        let response = client.get_service_info(request).await.unwrap().into_inner();
        assert_eq!(response.chain_id.as_deref(), Some(fixture.chain.as_str()));
        assert!(response.checkpoint_height.unwrap() > fixture.history_height);
        checks.push(json!({"case":"ingress-ledger-overwrites-external-boundary","externalHeaders":headers,"servedHeight":response.checkpoint_height}));
    }
    let after_reads = runtime.settled_bytes().await;
    assert!(after_reads[0] > baseline[0] && after_reads[1] > baseline[1]);
    let mut anonymous = ledger(ingress, false).ledger_client();
    assert!(
        anonymous
            .get_service_info(grpc::GetServiceInfoRequest::default())
            .await
            .is_err()
    );
    assert_eq!(runtime.settled_bytes().await, after_reads);
    checks.push(json!({"case":"ingress-ledger-mtls-denial-before-backend"}));
    let base = ingress.graphql_url.strip_suffix("/graphql").unwrap();
    for path in [
        "/_publication",
        "/%5fpublication",
        "/metrics",
        "/graphql/extra",
        "/rtd.rpc.v2.LedgerService/GetServiceInfo",
    ] {
        let response = http.get(format!("{base}{path}")).send().await.unwrap();
        assert_eq!(response.status(), 404, "{path}");
        let _ = response.bytes().await.unwrap();
    }
    assert_eq!(runtime.settled_bytes().await, after_reads);
    checks.push(json!({"case":"ingress-private-and-unknown-paths-not-routed","paths":5}));
    checks
}

pub(super) async fn verify_publication_closed(ingress: &Ingress, runtime: &Runtime) -> Value {
    let before = runtime.settled_bytes().await;
    let response = public_http(ingress)
        .post(&ingress.graphql_url)
        .header("content-type", "application/json")
        .header("x-rtd-visible-checkpoint", "18446744073709551615")
        .body(r#"{"query":"{ checkpoint { sequenceNumber digest } }"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 503);
    assert_eq!(response.headers()["retry-after"], "1");
    assert!(!response.headers().contains_key("x-rtd-visible-checkpoint"));
    let body = response.bytes().await.unwrap();
    assert!(!String::from_utf8_lossy(&body).contains("sequenceNumber"));
    let mut client = ledger(ingress, true).ledger_client();
    let error = client
        .get_service_info(grpc::GetServiceInfoRequest::default())
        .await
        .unwrap_err();
    assert_eq!(error.code(), tonic::Code::Unavailable, "{error:?}");
    let after = runtime.settled_bytes().await;
    assert_eq!(
        after, before,
        "denied requests reached a native business backend"
    );
    json!({"case":"ingress-fresh-publication-failure-before-both-backends","beforeBytes":before,"afterBytes":after})
}

pub(super) async fn verify_recovered(fixture: &Fixture, ingress: &Ingress) -> Value {
    let value = query(
        &public_http(ingress),
        &ingress.graphql_url,
        "{ chainIdentifier checkpoint { digest } }",
        &[],
    )
    .await;
    success(&value);
    assert_eq!(value["data"]["chainIdentifier"], fixture.chain);
    let mut client = ledger(ingress, true).ledger_client();
    assert!(
        client
            .get_service_info(grpc::GetServiceInfoRequest::default())
            .await
            .unwrap()
            .into_inner()
            .checkpoint_height
            .unwrap()
            > 0
    );
    json!({"case":"ingress-recovery-revalidated-through-real-native-services"})
}

pub(super) async fn verify_hbase_closed(ingress: &Ingress) -> Value {
    let value = query(
        &public_http(ingress),
        &ingress.graphql_url,
        "{ checkpoint { digest } }",
        &[],
    )
    .await;
    field_denied(&value, "checkpoint");
    let mut client = ledger(ingress, true).ledger_client();
    assert!(
        client
            .get_service_info(grpc::GetServiceInfoRequest::default())
            .await
            .is_err()
    );
    json!({"case":"ingress-native-hbase-failure-does-not-return-business-data"})
}
