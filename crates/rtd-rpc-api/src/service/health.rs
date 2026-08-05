// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use axum::extract::{Query, State};
use std::time::Duration;
use std::time::SystemTime;

use crate::Result;
use crate::RpcService;

pub(crate) type ReadinessCheck =
    std::sync::Arc<dyn Fn() -> anyhow::Result<()> + Send + Sync + 'static>;

pub(crate) fn check_node_readiness(readiness: Option<&ReadinessCheck>) -> crate::Result<()> {
    if let Some(readiness) = readiness {
        readiness()?;
    }
    Ok(())
}

impl RpcService {
    /// Perform a simple health check on the service.
    ///
    /// The threshold, or delta, between the server's system time and the timestamp in the most
    /// recently executed checkpoint for which the server is considered to be healthy.
    ///
    /// If not provided, the server will be considered healthy if it can simply fetch the latest
    /// checkpoint from its store.
    pub fn health_check(&self, threshold_seconds: Option<u32>) -> Result<()> {
        check_node_readiness(self.readiness_check.as_ref())?;
        let summary = self.reader.inner().get_latest_checkpoint()?;

        // If we have a provided threshold, check that it's close to the current time
        if let Some(threshold_seconds) = threshold_seconds {
            let latest_chain_time = summary.timestamp();

            let threshold = SystemTime::now() - Duration::from_secs(threshold_seconds as u64);

            if latest_chain_time < threshold {
                return Err(anyhow::anyhow!(
                    "The latest checkpoint timestamp is less than the provided threshold"
                )
                .into());
            }
        }

        Ok(())
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Threshold {
    /// The threshold, or delta, between the server's system time and the timestamp in the most
    /// recently executed checkpoint for which the server is considered to be healthy.
    ///
    /// If not provided, the server will be considered healthy if it can simply fetch the latest
    /// checkpoint from its store.
    pub threshold_seconds: Option<u32>,

    /// Include the failed readiness condition in an unhealthy response body.
    ///
    /// The default response remains `down` for compatibility with existing health check clients.
    pub verbose: Option<bool>,
}

fn unhealthy_response_body(error: crate::RpcError, verbose: bool) -> String {
    if !verbose {
        return "down".to_owned();
    }

    let message = error.into_status_proto().message;
    if message.starts_with("Fullnode is catching up:") {
        format!("down: {message}")
    } else {
        "down".to_owned()
    }
}

pub async fn health(
    Query(Threshold {
        threshold_seconds,
        verbose,
    }): Query<Threshold>,
    State(state): State<RpcService>,
) -> impl axum::response::IntoResponse {
    match state.health_check(threshold_seconds) {
        Ok(()) => (axum::http::StatusCode::OK, "up".to_owned()),
        Err(error) => (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            unhealthy_response_body(error, verbose.unwrap_or_default()),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{ReadinessCheck, check_node_readiness, unhealthy_response_body};
    use std::sync::Arc;

    #[test]
    fn readiness_failure_makes_health_check_fail() {
        let readiness: ReadinessCheck = Arc::new(|| Err(anyhow::anyhow!("catching up")));

        let error = check_node_readiness(Some(&readiness))
            .unwrap_err()
            .into_status_proto();

        assert!(error.message.contains("catching up"));
    }

    #[test]
    fn default_health_failure_body_remains_compatible() {
        let error = anyhow::anyhow!("network startup is incomplete").into();

        assert_eq!(unhealthy_response_body(error, false), "down");
    }

    #[test]
    fn verbose_health_failure_body_includes_readiness_reason() {
        let error = anyhow::anyhow!(
            "Fullnode is catching up: validator network recovery is still in progress"
        )
        .into();

        assert_eq!(
            unhealthy_response_body(error, true),
            "down: Fullnode is catching up: validator network recovery is still in progress"
        );
    }

    #[test]
    fn verbose_health_failure_does_not_expose_unrelated_internal_error() {
        let error = anyhow::anyhow!("database path and internal failure details").into();

        assert_eq!(unhealthy_response_body(error, true), "down");
    }
}
