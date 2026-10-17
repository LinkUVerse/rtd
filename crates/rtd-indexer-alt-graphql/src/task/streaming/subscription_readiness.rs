// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use std::sync::OnceLock;

use tokio::sync::watch;

use crate::task::watermark::KV_PACKAGES_PIPELINE;
use crate::task::watermark::Watermarks;

/// Coordinates readiness of streaming subscriptions at service startup.
///
/// When the service begins streaming at checkpoint C, any package published at
/// checkpoints before C lives only in the database. A subscriber reading from the
/// streaming tip needs those earlier packages resolvable, so subscriptions must wait
/// until `kv_packages` has caught up to at least C - 1 before starting.
pub(crate) struct SubscriptionReadiness {
    /// Sequence number of the first checkpoint received from the live upstream gRPC stream
    /// (the broadcast tip at startup). Not from the kv-rpc resume fallback.
    first_live_checkpoint: OnceLock<u64>,
    watermarks_rx: watch::Receiver<Arc<Watermarks>>,
}

impl SubscriptionReadiness {
    pub(crate) fn new(watermarks_rx: watch::Receiver<Arc<Watermarks>>) -> Arc<Self> {
        Arc::new(Self {
            first_live_checkpoint: OnceLock::new(),
            watermarks_rx,
        })
    }

    /// Record the first checkpoint received from the live upstream stream. Idempotent — only
    /// the first call has effect.
    pub(crate) fn record_first_live_checkpoint(&self, checkpoint_seq: u64) {
        let _ = self.first_live_checkpoint.set(checkpoint_seq);
    }

    /// The first live checkpoint, once recorded. Admission and the live broadcast
    /// use this identity without waiting to bind the ordinary query listener.
    pub(crate) fn first_live_checkpoint(&self) -> Option<u64> {
        self.first_live_checkpoint.get().copied()
    }

    /// Queries can start immediately; subscriptions remain closed until the original
    /// package coverage requirement is met. Recheck rather than cache readiness.
    pub(crate) fn is_ready(&self) -> bool {
        let Some(first_cp) = self.first_live_checkpoint() else {
            return false;
        };
        self.watermarks_rx
            .borrow()
            .per_pipeline()
            .get(KV_PACKAGES_PIPELINE)
            .is_some_and(|p| p.hi().checkpoint() >= first_cp.saturating_sub(1))
    }

    /// A publication-bounded historical feed only needs packages through its
    /// admitted height. It cannot emit live checkpoints above that bound.
    pub(crate) fn is_ready_at(&self, checkpoint: u64) -> bool {
        self.first_live_checkpoint().is_some()
            && self
                .watermarks_rx
                .borrow()
                .per_pipeline()
                .get(KV_PACKAGES_PIPELINE)
                .is_some_and(|p| p.hi().checkpoint() >= checkpoint)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio::sync::watch;

    use super::SubscriptionReadiness;
    use crate::task::watermark::KV_PACKAGES_PIPELINE;
    use crate::task::watermark::Watermarks;

    #[tokio::test]
    async fn readiness_requires_first_live_and_current_package_coverage() {
        let (tx, rx) = watch::channel(Arc::new(Watermarks::for_test(&[(
            KV_PACKAGES_PIPELINE,
            99,
        )])));
        let ready = SubscriptionReadiness::new(rx);
        assert!(!ready.is_ready());
        assert!(!ready.is_ready_at(99));
        ready.record_first_live_checkpoint(100);
        assert!(ready.is_ready());
        assert!(ready.is_ready_at(99));
        assert!(!ready.is_ready_at(100));
        tx.send_replace(Arc::new(Watermarks::for_test(&[(
            KV_PACKAGES_PIPELINE,
            98,
        )])));
        assert!(!ready.is_ready());
        assert!(ready.is_ready_at(98));
        assert!(!ready.is_ready_at(99));
        tx.send_replace(Arc::new(Watermarks::for_test(&[])));
        assert!(!ready.is_ready());
        tx.send_replace(Arc::new(Watermarks::for_test(&[(
            KV_PACKAGES_PIPELINE,
            100,
        )])));
        assert!(ready.is_ready());
        ready.record_first_live_checkpoint(200);
        assert_eq!(ready.first_live_checkpoint(), Some(100));
    }
}
