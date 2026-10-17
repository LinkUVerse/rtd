// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::time::Duration;

use reqwest::Client;
use reqwest::Response;
use serde_json::Value;
use serde_json::json;
use tokio::sync::watch;
use tokio::time::Instant;
use tokio::time::timeout;

use super::Fixture;
use super::http_client;
use super::query;
use super::success;

struct Events {
    response: Response,
    pending: Vec<u8>,
}

impl Events {
    async fn next(&mut self) -> Option<Value> {
        loop {
            if let Some(end) = self.pending.windows(2).position(|bytes| bytes == b"\n\n") {
                let frame: Vec<_> = self.pending.drain(..end + 2).collect();
                let frame = std::str::from_utf8(&frame).unwrap();
                if frame.starts_with(':') {
                    continue;
                }
                assert!(frame.lines().any(|line| line.trim_end() == "event: next"));
                let data = frame
                    .lines()
                    .filter_map(|line| line.strip_prefix("data: "))
                    .collect::<Vec<_>>()
                    .join("\n");
                return Some(serde_json::from_str(&data).unwrap());
            }
            let Some(bytes) = self.response.chunk().await.unwrap() else {
                assert!(self.pending.is_empty(), "truncated SSE frame");
                return None;
            };
            self.pending.extend_from_slice(&bytes);
            assert!(
                self.pending.len() < 4 * 1024 * 1024,
                "SSE frame size budget"
            );
        }
    }

    async fn required(&mut self) -> Value {
        timeout(Duration::from_secs(25), self.next())
            .await
            .expect("SSE delivery timeout")
            .expect("SSE ended before its expected payload")
    }
}

async fn open(client: &Client, url: &str, selection: &str) -> Response {
    client
        .post(url)
        .header("accept", "text/event-stream")
        .header("x-rtd-visible-checkpoint", "18446744073709551615")
        .json(&json!({"query":selection}))
        .send()
        .await
        .unwrap()
}

async fn connect(client: &Client, url: &str, selection: &str) -> Events {
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        let response = open(client, url, selection).await;
        if response.status() == 200 {
            assert!(
                response.headers()["content-type"]
                    .to_str()
                    .unwrap()
                    .starts_with("text/event-stream")
            );
            return Events {
                response,
                pending: Vec::new(),
            };
        }
        assert_eq!(response.status(), 503);
        let body = response.text().await.unwrap();
        assert!(Instant::now() < deadline, "SSE readiness: {body}");
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

pub(super) async fn verify(fixture: &Fixture, authority: &watch::Sender<bool>) -> Vec<Value> {
    let ingress = fixture
        .ingress
        .as_ref()
        .expect("SSE acceptance requires the real ingress");
    let client = Client::builder()
        .no_proxy()
        .tls_built_in_root_certs(false)
        .add_root_certificate(
            reqwest::Certificate::from_pem(&fs::read(&ingress.client.ca).unwrap()).unwrap(),
        )
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(60))
        .build()
        .unwrap();
    let url = format!("{}/subscriptions", ingress.graphql_url);
    let historical = fixture.history_height;
    let mut checks = Vec::new();
    let selection = format!(
        "subscription {{ checkpoints(afterCheckpoint:{historical}) {{ cursor node {{ sequenceNumber digest timestamp }} }} }}"
    );
    let mut events = connect(&client, &url, &selection).await;
    let admitted: u64 = events.response.headers()["x-rtd-visible-checkpoint"]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!(admitted >= historical + 3 && admitted < u64::MAX);
    let legacy = http_client(None);
    let mut last_cursor = String::new();
    for expected in historical + 1..=historical + 2 {
        let payload = events.required().await;
        success(&payload);
        let edge = &payload["data"]["checkpoints"];
        assert_eq!(edge["node"]["sequenceNumber"], expected);
        let baseline = query(
            &legacy,
            &fixture.baseline_graphql_url,
            &format!(
                "{{ checkpoint(sequenceNumber:{expected}) {{ sequenceNumber digest timestamp }} }}"
            ),
            &[],
        )
        .await;
        success(&baseline);
        assert_eq!(edge["node"], baseline["data"]["checkpoint"]);
        last_cursor = edge["cursor"].as_str().unwrap().to_owned();
        checks.push(json!({"case":"sse-real-historical-checkpoint-parity","checkpoint":expected,"admittedHeight":admitted}));
    }
    drop(events);
    let resume = format!(
        "subscription {{ checkpoints(after:{}) {{ cursor node {{ sequenceNumber digest }} }} }}",
        serde_json::to_string(&last_cursor).unwrap()
    );
    let mut events = connect(&client, &url, &resume).await;
    let payload = events.required().await;
    success(&payload);
    assert_eq!(
        payload["data"]["checkpoints"]["node"]["sequenceNumber"],
        historical + 3
    );
    drop(events);
    checks.push(json!({"case":"sse-cursor-resume-no-repeat-or-gap","checkpoint":historical + 3}));

    let fields = "digest effects { checkpoint { sequenceNumber } }";
    let filter = format!("{{afterCheckpoint:{historical}}}");
    let selection = format!(
        "subscription {{ transactions(filter:{filter}) {{ cursor node {{ {fields} }} }} }}"
    );
    let mut transactions = connect(&client, &url, &selection).await;
    let payload = transactions.required().await;
    success(&payload);
    let baseline = query(
        &legacy,
        &fixture.baseline_graphql_url,
        &format!("{{ transactions(first:1,filter:{filter}) {{ nodes {{ {fields} }} }} }}"),
        &[],
    )
    .await;
    success(&baseline);
    assert_eq!(
        baseline["data"]["transactions"]["nodes"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        payload["data"]["transactions"]["node"],
        baseline["data"]["transactions"]["nodes"][0]
    );
    drop(transactions);
    checks.push(json!({"case":"sse-real-transaction-effects-parity","checkpoint":payload["data"]["transactions"]["node"]["effects"]["checkpoint"]["sequenceNumber"]}));

    let event_checkpoint = fixture
        .subscription_event_checkpoint
        .expect("reviewed nonempty event checkpoint");
    assert!(event_checkpoint > historical && event_checkpoint <= admitted);
    let fields = "sequenceNumber eventBcs timestamp contents { type { repr } json } transaction { digest effects { checkpoint { sequenceNumber } } }";
    let baseline = query(&legacy, &fixture.baseline_graphql_url,
        &format!("{{ events(first:2,filter:{{atCheckpoint:{event_checkpoint}}}) {{ nodes {{ {fields} }} }} }}"), &[]).await;
    success(&baseline);
    let expected = baseline["data"]["events"]["nodes"].as_array().unwrap();
    assert_eq!(expected.len(), 2, "event fixture requires two real events");
    let selection = format!(
        "subscription {{ events(filter:{{afterCheckpoint:{}}}) {{ cursor node {{ {fields} }} }} }}",
        event_checkpoint - 1
    );
    let mut events = connect(&client, &url, &selection).await;
    let payload = events.required().await;
    success(&payload);
    assert_eq!(payload["data"]["events"]["node"], expected[0]);
    let cursor = payload["data"]["events"]["cursor"].as_str().unwrap();
    drop(events);
    checks.push(json!({"case":"sse-real-event-bcs-historical-layout-and-transaction-parity","checkpoint":event_checkpoint}));
    let selection = format!(
        "subscription {{ events(after:{}) {{ cursor node {{ {fields} }} }} }}",
        serde_json::to_string(cursor).unwrap()
    );
    let mut events = connect(&client, &url, &selection).await;
    let payload = events.required().await;
    success(&payload);
    assert_eq!(payload["data"]["events"]["node"], expected[1]);
    drop(events);
    checks.push(json!({"case":"sse-event-cursor-resumes-inside-checkpoint-without-repeat","checkpoint":event_checkpoint}));

    let head = query(
        &client,
        &ingress.graphql_url,
        "{ checkpoint { sequenceNumber } }",
        &[],
    )
    .await;
    success(&head);
    let future = head["data"]["checkpoint"]["sequenceNumber"]
        .as_u64()
        .unwrap()
        + 250;
    let selection = format!(
        "subscription {{ checkpoints(afterCheckpoint:{future}) {{ cursor node {{ sequenceNumber }} }} }}"
    );
    let mut idle = connect(&client, &url, &selection).await;
    let failure = idle.required().await;
    assert!(
        failure["data"].is_null() && failure["errors"].is_array(),
        "{failure}"
    );
    assert!(
        timeout(Duration::from_secs(3), idle.next())
            .await
            .unwrap()
            .is_none()
    );
    checks.push(json!({"case":"sse-future-resume-beyond-publication-refused"}));
    // This syntactically valid, unpublished type has no events. It keeps a real
    // scan idle without requiring a transaction or an unadmitted future cursor.
    let selection = format!(
        "subscription {{ events(filter:{{afterCheckpoint:{},type:\"0x0::native_migration_no_events::Never\"}}) {{ cursor }} }}",
        future - 250
    );
    let mut idle = connect(&client, &url, &selection).await;
    let observed = timeout(Duration::from_millis(250), idle.next()).await;
    assert!(
        observed.is_err(),
        "expected no events for an unpublished type: {observed:?}"
    );
    let started = Instant::now();
    authority.send(false).unwrap();
    let failure = idle.required().await;
    assert!(failure["data"].is_null(), "{failure}");
    assert!(
        failure["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["message"]
                .as_str()
                .unwrap()
                .contains("Publication authority is unavailable")),
        "{failure}"
    );
    assert!(
        timeout(Duration::from_secs(3), idle.next())
            .await
            .unwrap()
            .is_none()
    );
    checks.push(json!({"case":"sse-idle-authority-loss-errors-and-ends","elapsedMillis":started.elapsed().as_millis()}));
    let closed = open(&client, &url, &selection).await;
    assert_eq!(closed.status(), 503);
    let failure: Value = closed.json().await.unwrap();
    assert!(failure["data"].is_null() && failure["errors"].is_array());
    checks.push(json!({"case":"sse-fresh-connection-refused-after-authority-loss"}));
    success(&query(&client, &ingress.graphql_url, "{ chainIdentifier }", &[]).await);
    checks.push(json!({"case":"sse-dedicated-authority-fault-does-not-stop-query-service"}));
    authority.send(true).unwrap();
    let mut recovered = connect(&client, &url, &resume).await;
    let payload = recovered.required().await;
    success(&payload);
    assert_eq!(
        payload["data"]["checkpoints"]["node"]["sequenceNumber"],
        historical + 3
    );
    drop(recovered);
    checks.push(json!({"case":"sse-recovery-rechecks-authority-and-preserves-resume-cursor"}));
    checks
}
