// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;
use tokio::sync::watch;
use tokio::task::JoinHandle;

use super::Fixture;
use super::ProxySpec;
use super::http_client;
use super::proxy;
use super::query;
use super::success;

#[derive(Deserialize)]
pub(super) struct Business {
    checkpoint: u64,
    transaction: String,
    account: String,
    package: String,
    publish_transaction: String,
    fullnode: ProxySpec,
    consistent: ProxySpec,
}

pub(super) struct Runtime {
    fullnode: watch::Sender<bool>,
    consistent: watch::Sender<bool>,
    fullnode_task: JoinHandle<()>,
    consistent_task: JoinHandle<()>,
}

impl Runtime {
    pub async fn start(fixture: &Fixture, business: &Business) -> Self {
        for (argument, spec) in [
            ("--fullnode-rpc-url", &business.fullnode),
            ("--consistent-store-url", &business.consistent),
        ] {
            let args = &fixture.processes["graphql"].args;
            let i = args.iter().position(|a| a == argument).unwrap();
            assert_eq!(args[i + 1], format!("http://{}", spec.listen));
            assert_ne!(spec.listen, spec.target);
        }
        let (fullnode, fullnode_task) = proxy(&business.fullnode).await;
        let (consistent, consistent_task) = proxy(&business.consistent).await;
        Self {
            fullnode,
            consistent,
            fullnode_task,
            consistent_task,
        }
    }

    pub async fn stop(self) {
        drop(self.fullnode);
        drop(self.consistent);
        self.fullnode_task.await.unwrap();
        self.consistent_task.await.unwrap();
    }
}

async fn variables(client: &Client, url: &str, selection: &str, values: Value) -> Value {
    let response = client
        .post(url)
        .json(&json!({"query":selection,"variables":values}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    response.json().await.unwrap()
}

fn unavailable(value: &Value) {
    let errors = value["errors"]
        .as_array()
        .expect("dependency error required");
    assert!(!errors.is_empty(), "{value}");
    assert!(
        errors
            .iter()
            .all(|e| e["extensions"]["code"] == "DEPENDENCY_UNAVAILABLE"),
        "{value}"
    );
}

pub(super) async fn verify(
    fixture: &Fixture,
    business: &Business,
    runtime: &Runtime,
) -> Vec<Value> {
    let client = http_client(None);
    let url = &fixture.graphql_url;
    let legacy = &fixture.baseline_graphql_url;
    let height = business.checkpoint.to_string();
    let before = (business.checkpoint - 1).to_string();
    assert!(business.checkpoint > fixture.history_height);
    let mut checks = Vec::new();
    let object_fields = "id address version digest objectBcs";
    let tx_fields = format!(
        "id digest transactionBcs effects {{ status checkpoint {{ sequenceNumber }} objectChanges(first:10) {{ nodes {{ address idCreated idDeleted inputState {{ {object_fields} }} outputState {{ {object_fields} }} }} }} }}"
    );
    let selection = format!(
        "{{ transaction(digest:{}) {{ {tx_fields} }} }}",
        json!(business.transaction)
    );
    let actual = query(&client, url, &selection, &[&height]).await;
    let baseline = query(&client, legacy, &selection, &[]).await;
    success(&actual);
    success(&baseline);
    assert_eq!(actual, baseline);
    let tx = &actual["data"]["transaction"];
    assert_eq!(
        tx["effects"]["checkpoint"]["sequenceNumber"],
        business.checkpoint
    );
    let changes = tx["effects"]["objectChanges"]["nodes"].as_array().unwrap();
    let modified = changes
        .iter()
        .find(|o| o["inputState"].is_object() && o["outputState"].is_object())
        .expect("nonempty historical modification");
    let created = changes
        .iter()
        .find(|o| o["idCreated"] == true)
        .expect("nonempty historical creation");
    assert!(
        modified["outputState"]["version"].as_u64().unwrap()
            > modified["inputState"]["version"].as_u64().unwrap()
    );
    checks.push(json!({"case":"business-transaction-nested-input-output-bcs-parity","checkpoint":business.checkpoint,"objectChanges":changes.len()}));

    for state in ["inputState", "outputState"] {
        let selection = format!(
            "{{ node(id:{}) {{ ... on Object {{ {object_fields} }} }} }}",
            modified[state]["id"]
        );
        let node = query(&client, url, &selection, &[&height]).await;
        success(&node);
        assert_eq!(node["data"]["node"], modified[state]);
        checks.push(json!({"case":"business-node-historical-object-refetch","state":state,"version":modified[state]["version"]}));
    }
    for (kind, id) in [
        ("Transaction", &tx["id"]),
        ("Object", &created["outputState"]["id"]),
    ] {
        let fields = if kind == "Transaction" {
            "id digest"
        } else {
            "id address version"
        };
        let selection = format!("{{ node(id:{id}) {{ ... on {kind} {{ {fields} }} }} }}");
        let future = query(&client, url, &selection, &[&before]).await;
        success(&future);
        assert!(future["data"]["node"].is_null(), "{future}");
        let current = query(&client, url, &selection, &[&height]).await;
        success(&current);
        assert_eq!(current["data"]["node"]["id"], *id);
        checks
            .push(json!({"case":"business-node-future-identity-hidden-then-visible","kind":kind}));
    }
    let selection = format!(
        "{{ object(address:{},atCheckpoint:{}) {{ {object_fields} }} }}",
        modified["address"], business.checkpoint
    );
    let object = query(&client, url, &selection, &[&height]).await;
    success(&object);
    assert_eq!(object["data"]["object"], modified["outputState"]);
    checks.push(json!({"case":"business-explicit-historical-object-content-parity"}));

    let published = query(
        &client,
        legacy,
        &format!(
            "{{ transaction(digest:{}) {{ effects {{ checkpoint {{ sequenceNumber }} }} }} }}",
            json!(business.publish_transaction)
        ),
        &[],
    )
    .await;
    success(&published);
    let published = published["data"]["transaction"]["effects"]["checkpoint"]["sequenceNumber"]
        .as_u64()
        .unwrap();
    assert!(published > fixture.history_height);
    let selection = format!(
        "{{ package(address:{}) {{ id address version }} }}",
        json!(business.package)
    );
    let package = query(&client, url, &selection, &[&published.to_string()]).await;
    success(&package);
    assert_eq!(package["data"]["package"]["address"], business.package);
    let future = query(&client, url, &selection, &[&(published - 1).to_string()]).await;
    success(&future);
    assert!(future["data"]["package"].is_null(), "{future}");
    checks.push(
        json!({"case":"business-package-publication-boundary-enforced","publishedAt":published}),
    );

    let account = format!(
        "{{ address(address:{},atCheckpoint:{}) {{ address balance(coinType:\"0x2::rtd::RTD\") {{ totalBalance coinBalance addressBalance }} objects(first:2) {{ nodes {{ address version }} }} }} }}",
        json!(business.account),
        business.checkpoint
    );
    let balance = query(&client, url, &account, &[&height]).await;
    let baseline = query(&client, legacy, &account, &[]).await;
    success(&balance);
    success(&baseline);
    assert_eq!(balance, baseline);
    assert_ne!(balance["data"]["address"]["balance"]["totalBalance"], "0");
    assert_eq!(
        balance["data"]["address"]["objects"]["nodes"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    checks.push(json!({"case":"business-consistent-real-nonempty-balance-and-ownership-parity","checkpoint":business.checkpoint}));
    runtime.consistent.send(false).unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let failure = query(&client, url, &account, &[&height]).await;
    unavailable(&failure);
    assert!(
        failure["data"]["address"]["balance"].is_null()
            && failure["data"]["address"]["objects"].is_null(),
        "{failure}"
    );
    checks
        .push(json!({"case":"business-consistent-transport-failure-hides-balance-and-ownership"}));
    runtime.consistent.send(true).unwrap();
    let recovered = query(&client, url, &account, &[&height]).await;
    success(&recovered);
    assert_eq!(recovered, balance);
    checks.push(json!({"case":"business-consistent-recovery-same-history"}));

    let simulation = "query($tx: JSON!) { simulateTransaction(transaction:$tx,checksEnabled:false,doGasSelection:false) { effects { status } } }";
    let input = json!({"tx":{"sender":"0x0","kind":{"programmable_transaction":{"inputs":[],"commands":[]}}}});
    let simulated = variables(&client, url, simulation, input.clone()).await;
    let baseline = variables(&client, legacy, simulation, input.clone()).await;
    success(&simulated);
    success(&baseline);
    assert_eq!(simulated, baseline);
    assert_eq!(
        simulated["data"]["simulateTransaction"]["effects"]["status"],
        "SUCCESS"
    );
    checks.push(json!({"case":"business-fullnode-readonly-simulation-parity"}));
    runtime.fullnode.send(false).unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let failure = variables(&client, url, simulation, input.clone()).await;
    unavailable(&failure);
    assert!(failure["data"].is_null(), "{failure}");
    checks.push(json!({"case":"business-fullnode-simulation-transport-failure"}));
    // Exercise dispatch failure only while this owned transport is disabled.
    // No signatures are supplied and no execution request reaches the chain.
    assert!(!*runtime.fullnode.borrow());
    let execution = variables(&client, url,
        "mutation($tx: Base64!) { executeTransaction(transactionDataBcs:$tx,signatures:[]) { effects { status } } }",
        json!({"tx":tx["transactionBcs"]})).await;
    unavailable(&execution);
    assert!(execution["data"].is_null(), "{execution}");
    checks.push(json!({"case":"business-fullnode-execution-refused-with-owned-upstream-disabled","signed":false,"dispatchedToChain":false}));
    runtime.fullnode.send(true).unwrap();
    let recovered = variables(&client, url, simulation, input).await;
    success(&recovered);
    assert_eq!(recovered, simulated);
    checks.push(json!({"case":"business-fullnode-readonly-simulation-recovery"}));
    checks
}
