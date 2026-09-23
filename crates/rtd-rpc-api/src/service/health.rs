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

pub(crate) fn serving_status(readiness: Option<&ReadinessCheck>) -> tonic_health::ServingStatus {
    if check_node_readiness(readiness).is_ok() {
        tonic_health::ServingStatus::Serving
    } else {
        tonic_health::ServingStatus::NotServing
    }
}

/// The largest gap, in checkpoints, between the latest executed checkpoint and
/// the highest checkpoint the live-object index has committed while still
/// considered healthy.
///
/// The embedded indexer follows the tip asynchronously, so the live-object
/// index always trails the executed tip by a little (roughly the indexer's
/// snapshot window). A gap larger than this means the live index has fallen
/// behind -- e.g. the indexer has stalled -- and cannot serve current
/// live-object reads. The ledger-history cohort backfills separately after a
/// restore and is deliberately excluded from this gap, so a node is healthy as
/// soon as its live-object reads are caught up.
const MAX_HEALTHY_INDEX_LAG: u64 = 60;

impl RpcService {
    /// Perform a simple health check on the service.
    ///
    /// The threshold, or delta, between the server's system time and the
    /// timestamp in the most recently executed checkpoint for which the server
    /// is considered to be healthy. If not provided, the server's tip is not
    /// subject to a staleness check.
    ///
    /// Independent of the threshold, when indexing is enabled the server is
    /// only considered healthy once its live-object indexes have caught up to
    /// within `MAX_HEALTHY_INDEX_LAG` checkpoints of the latest executed
    /// checkpoint. When indexing is disabled this check is skipped.
    pub fn health_check(&self, threshold_seconds: Option<u32>) -> Result<()> {
        check_node_readiness(self.readiness_check.as_ref())?;
        let latest = self.reader.inner().get_latest_checkpoint()?;

        // If we have a provided threshold, check that it's close to the current
        // time.
        if let Some(threshold_seconds) = threshold_seconds {
            let latest_chain_time = latest.timestamp();

            let threshold = SystemTime::now() - Duration::from_secs(threshold_seconds as u64);

            if latest_chain_time < threshold {
                return Err(anyhow::anyhow!(
                    "The latest checkpoint timestamp is less than the provided threshold"
                )
                .into());
            }
        }

        // When indexing is enabled, the node is only healthy once its
        // live-object indexes (owned objects, types, balances) have kept up
        // with the executed tip. Those indexes are restored to the tip and
        // follow it, so a healthy node's live frontier trails execution by at
        // most the indexer's snapshot window. The ledger-history cohort
        // backfills independently after a restore and is deliberately excluded:
        // gating on it would report a node unhealthy for the whole backfill
        // even though its live-object reads are already caught up. The executed
        // tip is read unbounded (rather than via `get_latest_checkpoint`, which
        // is itself bounded to the live frontier) so a stalled live indexer,
        // whose frontier falls behind ongoing execution, is still detected. A
        // node without an index surface (indexing disabled) skips this check.
        if let Some(indexes) = self.reader.inner().indexes() {
            let executed = self
                .reader
                .inner()
                .get_highest_executed_checkpoint_seq_number()?;
            let highest_live_indexed = indexes.get_highest_live_indexed_checkpoint_seq_number()?;

            if !index_caught_up(executed, highest_live_indexed, MAX_HEALTHY_INDEX_LAG) {
                return Err(anyhow::anyhow!(
                    "the live-object index is not caught up to within {MAX_HEALTHY_INDEX_LAG} \
                     checkpoints of the latest executed checkpoint"
                )
                .into());
            }
        }

        Ok(())
    }
}

/// Whether the highest live-indexed checkpoint is close enough to the executed
/// tip to be considered healthy.
///
/// `highest_live_indexed` is `None` when the live-object index has not committed
/// any checkpoint yet, which is never healthy. The live frontier never runs
/// ahead of execution (it indexes executed checkpoints), but an equal frontier
/// saturates to a zero lag rather than underflowing.
fn index_caught_up(executed_seq: u64, highest_live_indexed: Option<u64>, max_lag: u64) -> bool {
    match highest_live_indexed {
        Some(indexed) => executed_seq.saturating_sub(indexed) <= max_lag,
        None => false,
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Threshold {
    /// The threshold, or delta, between the server's system time and the timestamp in the most
    /// recently executed checkpoint for which the server is considered to be healthy.
    ///
    /// If not provided, the server will be considered healthy if it can simply fetch the latest
    /// checkpoint from its store and, when indexing is enabled, its indexes have caught up to it.
    pub threshold_seconds: Option<u32>,

    /// Include the failed startup condition in an unhealthy response.
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
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[tokio::test]
    async fn readiness_failure_makes_health_check_fail() {
        let readiness: ReadinessCheck = std::sync::Arc::new(|| {
            Err(anyhow::anyhow!(
                "Fullnode is catching up: RPC live index checkpoint is at 5/7"
            ))
        });

        let error = check_node_readiness(Some(&readiness))
            .unwrap_err()
            .into_status_proto();

        assert!(
            error
                .message
                .contains("RPC live index checkpoint is at 5/7")
        );
    }

    #[tokio::test]
    async fn verbose_failure_only_exposes_readiness_reason() {
        let readiness = anyhow::anyhow!(
            "Fullnode is catching up: pending transaction recovery has not started"
        )
        .into();
        let unrelated = anyhow::anyhow!("database failed").into();

        assert_eq!(
            unhealthy_response_body(readiness, true),
            "down: Fullnode is catching up: pending transaction recovery has not started"
        );
        assert_eq!(unhealthy_response_body(unrelated, true), "down");
    }

    #[tokio::test]
    async fn grpc_serving_status_follows_readiness() {
        let ready = std::sync::Arc::new(AtomicBool::new(false));
        let ready_for_check = ready.clone();
        let readiness: ReadinessCheck = std::sync::Arc::new(move || {
            if ready_for_check.load(Ordering::Acquire) {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Fullnode is catching up"))
            }
        });

        assert_eq!(
            serving_status(Some(&readiness)),
            tonic_health::ServingStatus::NotServing
        );
        ready.store(true, Ordering::Release);
        assert_eq!(
            serving_status(Some(&readiness)),
            tonic_health::ServingStatus::Serving
        );
    }

    // The live-object index has not committed any checkpoint yet: never
    // healthy.
    #[test]
    fn not_caught_up_when_unindexed() {
        assert!(!index_caught_up(100, None, MAX_HEALTHY_INDEX_LAG));
        assert!(!index_caught_up(0, None, MAX_HEALTHY_INDEX_LAG));
    }

    // The live frontier is within (or at) the allowed lag of the executed tip:
    // healthy.
    #[test]
    fn caught_up_within_lag() {
        assert!(index_caught_up(100, Some(100), 60)); // no lag
        assert!(index_caught_up(100, Some(40), 60)); // exactly at the bound
        assert!(index_caught_up(100, Some(41), 60)); // inside the bound
    }

    // The live frontier trails the executed tip by more than the allowed lag
    // (e.g. the live indexer stalled while execution advanced): unhealthy. A
    // lagging ledger-history backfill does not reach this path -- it is not part
    // of the live frontier.
    #[test]
    fn not_caught_up_beyond_lag() {
        assert!(!index_caught_up(100, Some(39), 60)); // one past the bound
        assert!(!index_caught_up(1_000, Some(0), 60)); // live index far behind
    }

    // A live frontier level with the executed tip saturates to zero lag rather
    // than underflowing.
    #[test]
    fn caught_up_when_index_at_tip() {
        assert!(index_caught_up(100, Some(100), 60));
        assert!(index_caught_up(100, Some(200), 60));
    }
}
