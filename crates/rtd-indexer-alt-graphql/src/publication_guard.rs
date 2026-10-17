// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use async_graphql::Response;
use async_graphql::ServerError;
use fastcrypto::encoding::Base58;
use fastcrypto::encoding::Encoding;
use futures::Stream;
use futures::StreamExt;
use reqwest::Client;
use reqwest::StatusCode;
use url::Url;

use crate::admission;
use crate::args::SubscriptionArgs;
use crate::task::chain_identifier::ChainIdentifier;

#[cfg(test)]
mod transport_tests;

#[derive(Clone, Default)]
pub(crate) struct PublicationGuard(Option<Arc<Authority>>);

struct Authority {
    client: Client,
    url: Url,
    chain: String,
}

impl PublicationGuard {
    pub(crate) async fn from_args(
        args: &SubscriptionArgs,
        chain: &ChainIdentifier,
    ) -> anyhow::Result<Self> {
        let Some(url) = &args.publication_guard_url else {
            return Ok(Self::default());
        };
        anyhow::ensure!(
            url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none()
                && url.path() == "/authorize",
            "Publication authority must use a fixed /authorize URL without credentials or query"
        );
        let loopback = url
            .host_str()
            .and_then(|host| {
                host.trim_matches(['[', ']'])
                    .parse::<std::net::IpAddr>()
                    .ok()
            })
            .is_some_and(|ip| ip.is_loopback());
        anyhow::ensure!(
            url.scheme() == "https" || (url.scheme() == "http" && loopback),
            "Publication authority requires HTTPS except for literal loopback test addresses"
        );
        let mut client = Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(Duration::from_secs(1))
            .timeout(Duration::from_secs(18));
        if url.scheme() == "https" {
            let ca = args
                .publication_guard_ca
                .as_ref()
                .context("Publication authority CA is required")?;
            let identity = args
                .publication_guard_identity
                .as_ref()
                .context("Publication authority client identity is required")?;
            client = client
                .tls_built_in_root_certs(false)
                .add_root_certificate(reqwest::Certificate::from_pem(&std::fs::read(ca)?)?)
                .identity(reqwest::Identity::from_pem(&std::fs::read(identity)?)?);
        }
        let chain = tokio::time::timeout(Duration::from_secs(10), chain.wait())
            .await
            .context("Chain identity unavailable for publication authority")?;
        Ok(Self(Some(Arc::new(Authority {
            client: client.build()?,
            url: url.clone(),
            chain: Base58::encode(chain.as_bytes()),
        }))))
    }

    pub(crate) async fn height(&self, minimum: Option<u64>) -> Result<Option<u64>, ServerError> {
        let Some(authority) = &self.0 else {
            return if minimum.is_some() {
                Err(unavailable())
            } else {
                Ok(None)
            };
        };
        let response = authority
            .client
            .get(authority.url.clone())
            .send()
            .await
            .map_err(|_| unavailable())?;
        if response.status() != StatusCode::NO_CONTENT {
            return Err(unavailable());
        }
        validate_headers(response.headers(), &authority.chain, minimum).map(Some)
    }

    pub(crate) fn enabled(&self) -> bool {
        self.0.is_some()
    }

    /// Recheck idle streams and rendered payloads; no previous success authorizes a later frame.
    #[cfg(test)]
    pub(crate) fn protect<S>(
        self,
        source: S,
        minimum: Option<u64>,
    ) -> impl Stream<Item = Response> + Send
    where
        S: Stream<Item = Response> + Send,
    {
        self.protect_follow(source, minimum, None)
    }

    /// Advance only this connection's certified frontier. A later regression fails closed.
    pub(crate) fn protect_follow<S>(
        self,
        source: S,
        mut minimum: Option<u64>,
        follow: Option<tokio::sync::watch::Sender<u64>>,
    ) -> impl Stream<Item = Response> + Send
    where
        S: Stream<Item = Response> + Send,
    {
        async_stream::stream! {
            futures::pin_mut!(source);
            if self.0.is_none() {
                while let Some(response) = source.next().await { yield response; }
                return;
            }
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            loop {
                tokio::select! {
                    biased;
                    _ = interval.tick() => {
                        match self.height(minimum).await {
                            Ok(Some(height)) => {
                                minimum = Some(height);
                                if let Some(follow) = &follow { follow.send_if_modified(|value| {
                                    if height > *value { *value = height; true } else { false }
                                }); }
                            }
                            Ok(None) => {}
                            Err(error) => {
                                yield Response::from_errors(vec![error]);
                                return;
                            }
                        }
                        interval.reset();
                    }
                    response = source.next() => {
                        let Some(response) = response else { return; };
                        match self.height(minimum).await {
                            Ok(Some(height)) => {
                                minimum = Some(height);
                                if let Some(follow) = &follow { follow.send_if_modified(|value| {
                                    if height > *value { *value = height; true } else { false }
                                }); }
                            }
                            Ok(None) => {}
                            Err(error) => {
                                yield Response::from_errors(vec![error]);
                                return;
                            }
                        }
                        interval.reset();
                        yield response;
                    }
                }
            }
        }
    }
}

fn unavailable() -> ServerError {
    ServerError::new(
        "Publication authority is unavailable; reconnect after recovery",
        None,
    )
}

fn validate_headers(
    headers: &reqwest::header::HeaderMap,
    chain: &str,
    minimum: Option<u64>,
) -> Result<u64, ServerError> {
    let mut chains = headers.get_all("x-rtd-chain-id").iter();
    if chains.next().and_then(|value| value.to_str().ok()) != Some(chain) || chains.next().is_some()
    {
        return Err(unavailable());
    }
    let height = admission::ceiling(headers)?.ok_or_else(unavailable)?;
    if minimum.is_some_and(|minimum| height < minimum) {
        return Err(unavailable());
    }
    Ok(height)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use async_graphql::Response;
    use futures::StreamExt;
    use reqwest::Client;
    use reqwest::StatusCode;

    use crate::args::SubscriptionArgs;
    use crate::task::chain_identifier::ChainIdentifier;

    use super::Authority;
    use super::PublicationGuard;
    use super::validate_headers;

    #[tokio::test]
    async fn authority_headers_require_one_chain_and_canonical_non_regressing_height() {
        for (height, chain, success) in [
            ("7", "chain", true),
            ("6", "chain", false),
            ("07", "chain", false),
            ("-1", "chain", false),
            ("18446744073709551616", "chain", false),
            ("7", "other", false),
        ] {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("x-rtd-visible-checkpoint", height.parse().unwrap());
            headers.insert("x-rtd-chain-id", chain.parse().unwrap());
            assert_eq!(
                validate_headers(&headers, "chain", Some(7)).is_ok(),
                success
            );
            headers.append("x-rtd-chain-id", chain.parse().unwrap());
            assert!(validate_headers(&headers, "chain", Some(7)).is_err());
        }
    }

    #[tokio::test]
    async fn bounded_subscriptions_require_configured_authority() {
        assert!(PublicationGuard::default().height(Some(7)).await.is_err());
        assert_eq!(
            PublicationGuard::default().height(None).await.unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn rendered_payload_is_discarded_after_authority_failure_and_new_stream_recovers() {
        use std::sync::atomic::AtomicBool;
        use std::sync::atomic::Ordering;

        let ready = Arc::new(AtomicBool::new(true));
        let state = ready.clone();
        let app = axum::Router::new().route(
            "/authorize",
            axum::routing::get(move || {
                let state = state.clone();
                async move {
                    (
                        if state.load(Ordering::SeqCst) {
                            StatusCode::NO_CONTENT
                        } else {
                            StatusCode::SERVICE_UNAVAILABLE
                        },
                        [
                            ("x-rtd-chain-id", "chain"),
                            ("x-rtd-visible-checkpoint", "7"),
                        ],
                    )
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/authorize", listener.local_addr().unwrap())
            .parse()
            .unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let guard = PublicationGuard(Some(Arc::new(Authority {
            client: Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(1))
                .build()
                .unwrap(),
            url,
            chain: "chain".into(),
        })));
        let stream = guard.clone().protect(
            futures::stream::iter([
                Response::new(async_graphql::Value::from("first")),
                Response::new(async_graphql::Value::from("must-not-escape")),
            ]),
            Some(7),
        );
        futures::pin_mut!(stream);
        assert_eq!(
            stream.next().await.unwrap().data,
            async_graphql::Value::from("first")
        );
        ready.store(false, Ordering::SeqCst);
        let rejected = stream.next().await.unwrap();
        assert!(!rejected.errors.is_empty());
        assert_eq!(rejected.data, async_graphql::Value::Null);
        assert!(stream.next().await.is_none());
        ready.store(true, Ordering::SeqCst);
        let recovered = guard.clone().protect(
            futures::stream::iter([Response::new(async_graphql::Value::from("recovered"))]),
            Some(7),
        );
        futures::pin_mut!(recovered);
        assert_eq!(
            recovered.next().await.unwrap().data,
            async_graphql::Value::from("recovered")
        );
        ready.store(false, Ordering::SeqCst);
        let idle = guard.protect(futures::stream::pending(), Some(7));
        futures::pin_mut!(idle);
        assert!(
            !tokio::time::timeout(Duration::from_secs(2), idle.next())
                .await
                .unwrap()
                .unwrap()
                .errors
                .is_empty()
        );
        server.abort();
    }

    #[tokio::test]
    async fn follow_advances_and_rejects_a_later_frontier_regression() {
        use std::sync::atomic::AtomicU64;
        use std::sync::atomic::Ordering;
        let height = Arc::new(AtomicU64::new(7));
        let current = height.clone();
        let app = axum::Router::new().route(
            "/authorize",
            axum::routing::get(move || {
                let current = current.clone();
                async move {
                    (
                        StatusCode::NO_CONTENT,
                        [
                            ("x-rtd-chain-id", "chain".to_owned()),
                            (
                                "x-rtd-visible-checkpoint",
                                current.load(Ordering::SeqCst).to_string(),
                            ),
                        ],
                    )
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/authorize", listener.local_addr().unwrap())
            .parse()
            .unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let guard = PublicationGuard(Some(Arc::new(Authority {
            client: Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(1))
                .build()
                .unwrap(),
            url,
            chain: "chain".into(),
        })));
        let (tx, rx) = tokio::sync::watch::channel(7);
        let stream = guard.protect_follow(
            futures::stream::repeat_with(|| Response::new(async_graphql::Value::from("frame"))),
            Some(7),
            Some(tx),
        );
        futures::pin_mut!(stream);
        assert!(stream.next().await.unwrap().errors.is_empty());
        height.store(9, Ordering::SeqCst);
        assert!(stream.next().await.unwrap().errors.is_empty());
        assert_eq!(*rx.borrow(), 9);
        height.store(8, Ordering::SeqCst);
        let failed = stream.next().await.unwrap();
        assert!(!failed.errors.is_empty());
        assert_eq!(failed.data, async_graphql::Value::Null);
        assert!(stream.next().await.is_none());
        server.abort();
    }

    #[tokio::test]
    async fn authority_configuration_rejects_unsafe_transport_before_chain_lookup() {
        for url in [
            "http://example.com/authorize",
            "http://127.0.0.1/authorize?cached=true",
            "http://user@127.0.0.1/authorize",
            "http://127.0.0.1/other",
            "https://example.com/authorize",
        ] {
            let args = SubscriptionArgs {
                publication_guard_url: Some(url.parse().unwrap()),
                ..Default::default()
            };
            assert!(
                PublicationGuard::from_args(&args, &ChainIdentifier::default())
                    .await
                    .is_err()
            );
        }
    }
}
