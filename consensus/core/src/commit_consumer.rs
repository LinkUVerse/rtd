// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use linku_metrics::monitored_mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use parking_lot::Mutex;
use tokio::sync::watch;
use tracing::{debug, info};

use crate::{CommitIndex, CommittedSubDag, block::CertifiedBlocksOutput};

/// Maximum number of consensus commits that may be persisted ahead of the consumer's durable
/// state. Keeping this window bounded prevents a checkpoint-pipeline failure from leaving an
/// arbitrarily large consensus replay tail after a crash.
pub const MAX_PENDING_DURABLE_COMMITS: CommitIndex = 1_000;

/// Arguments from commit consumer to this consensus instance.
/// This includes both parameters and components for communications.
#[derive(Clone)]
pub struct CommitConsumerArgs {
    /// The consumer requests consensus to replay from commit replay_after_commit_index + 1.
    /// Set to 0 to replay from the start (as commit sequence starts at index = 1).
    pub(crate) replay_after_commit_index: CommitIndex,
    /// Index of the last commit that the consumer has processed.  This is useful during
    /// crash recovery when other components can wait for the consumer to finish processing
    /// up to this index.
    pub(crate) consumer_last_processed_commit_index: CommitIndex,

    /// A channel to output the committed sub dags.
    pub(crate) commit_sender: UnboundedSender<CommittedSubDag>,
    /// A channel to output blocks for processing, separated from consensus commits.
    /// In each block output, transactions that are not rejected are considered certified.
    pub(crate) block_sender: UnboundedSender<CertifiedBlocksOutput>,
    // Allows the commit consumer to report its progress.
    monitor: Arc<CommitConsumerMonitor>,
}

impl CommitConsumerArgs {
    pub fn new(
        replay_after_commit_index: CommitIndex,
        consumer_last_processed_commit_index: CommitIndex,
    ) -> (
        Self,
        UnboundedReceiver<CommittedSubDag>,
        UnboundedReceiver<CertifiedBlocksOutput>,
    ) {
        let monitor = Arc::new(CommitConsumerMonitor::new(
            replay_after_commit_index,
            consumer_last_processed_commit_index,
        ));
        Self::new_with_monitor(
            replay_after_commit_index,
            consumer_last_processed_commit_index,
            monitor,
        )
    }

    pub fn new_with_recovery_drain_ceiling(
        replay_after_commit_index: CommitIndex,
        consumer_last_processed_commit_index: CommitIndex,
        recovery_drain_ceiling: Option<CommitIndex>,
    ) -> (
        Self,
        UnboundedReceiver<CommittedSubDag>,
        UnboundedReceiver<CertifiedBlocksOutput>,
    ) {
        let monitor = Arc::new(CommitConsumerMonitor::new_with_recovery_drain_ceiling(
            replay_after_commit_index,
            consumer_last_processed_commit_index,
            recovery_drain_ceiling,
        ));
        Self::new_with_monitor(
            replay_after_commit_index,
            consumer_last_processed_commit_index,
            monitor,
        )
    }

    fn new_with_monitor(
        replay_after_commit_index: CommitIndex,
        consumer_last_processed_commit_index: CommitIndex,
        monitor: Arc<CommitConsumerMonitor>,
    ) -> (
        Self,
        UnboundedReceiver<CommittedSubDag>,
        UnboundedReceiver<CertifiedBlocksOutput>,
    ) {
        let (commit_sender, commit_receiver) = unbounded_channel("consensus_commit_output");
        let (block_sender, block_receiver) = unbounded_channel("consensus_block_output");

        (
            Self {
                replay_after_commit_index,
                consumer_last_processed_commit_index,
                commit_sender,
                block_sender,
                monitor,
            },
            commit_receiver,
            block_receiver,
        )
    }

    pub fn monitor(&self) -> Arc<CommitConsumerMonitor> {
        self.monitor.clone()
    }
}

/// Helps monitor the progress of the consensus commit handler (consumer).
///
/// This component tracks four related boundaries:
/// 1. Checking the highest commit index processed by the consensus commit handler.
///    Consensus components can decide to wait for more commits to be processed before proceeding with
///    their work.
/// 2. Waiting for consensus commit handler to finish processing replayed commits.
///    Current usage is actually outside of consensus.
/// 3. Tracking the highest consumer commit that is durable across process crashes.
/// 4. Bounding live consensus output against that durable watermark.
pub struct CommitConsumerMonitor {
    // highest commit that has been handled by the consumer.
    highest_handled_commit: watch::Sender<u32>,

    // Highest commit whose consumer output has been atomically persisted. This is intentionally
    // separate from `highest_handled_commit`: handling a commit only puts its output in RTD's
    // in-memory checkpoint quarantine, while this watermark is safe to resume from after SIGKILL.
    highest_durable_commit: watch::Sender<u32>,

    // Consensus recovery discovers its target only after opening the consensus store. The
    // checkpoint builder uses this status to process replay concurrently, but does not announce
    // startup completion until the complete recovered stream has reached the consumer.
    recovery_status: watch::Sender<CommitRecoveryStatus>,

    // A persisted maximum consensus head for databases which already exceeded the normal
    // durable-lag bound before the guard was introduced. Reusing this ceiling across restarts
    // prevents repeated crashes from granting a fresh drain allowance every time.
    recovery_drain_ceiling: Option<CommitIndex>,

    // Serializes retirement of the temporary legacy allowance with admission of commit-sync
    // ranges, which are fetched outside Core and handed to it asynchronously.
    recovery_drain_gate: Mutex<RecoveryDrainGate>,

    // Generic consensus users do not have RTD's checkpoint quarantine and historically report
    // only handled progress. Preserve that API behavior unless the RTD constructor explicitly
    // enables crash-safe durable tracking.
    explicit_durable_tracking: bool,

    // At node startup, the last consensus commit processed by the commit consumer from the previous run.
    // This can be 0 if starting a new epoch.
    consumer_last_processed_commit_index: CommitIndex,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CommitRecoveryStatus {
    target_commit: CommitIndex,
    started: bool,
    scan_complete: bool,
}

#[derive(Debug)]
struct RecoveryDrainGate {
    active: bool,
    highest_reserved_consensus_head: CommitIndex,
}

impl CommitConsumerMonitor {
    pub(crate) fn new(
        replay_after_commit_index: CommitIndex,
        consumer_last_processed_commit_index: CommitIndex,
    ) -> Self {
        Self::new_internal(
            replay_after_commit_index,
            consumer_last_processed_commit_index,
            None,
            false,
        )
    }

    pub fn new_with_recovery_drain_ceiling(
        replay_after_commit_index: CommitIndex,
        consumer_last_processed_commit_index: CommitIndex,
        recovery_drain_ceiling: Option<CommitIndex>,
    ) -> Self {
        Self::new_internal(
            replay_after_commit_index,
            consumer_last_processed_commit_index,
            recovery_drain_ceiling,
            true,
        )
    }

    fn new_internal(
        replay_after_commit_index: CommitIndex,
        consumer_last_processed_commit_index: CommitIndex,
        recovery_drain_ceiling: Option<CommitIndex>,
        explicit_durable_tracking: bool,
    ) -> Self {
        Self {
            highest_handled_commit: watch::Sender::new(replay_after_commit_index),
            highest_durable_commit: watch::Sender::new(consumer_last_processed_commit_index),
            recovery_status: watch::Sender::new(CommitRecoveryStatus::default()),
            recovery_drain_ceiling,
            recovery_drain_gate: Mutex::new(RecoveryDrainGate {
                active: recovery_drain_ceiling.is_some(),
                highest_reserved_consensus_head: replay_after_commit_index
                    .max(consumer_last_processed_commit_index),
            }),
            explicit_durable_tracking,
            consumer_last_processed_commit_index,
        }
    }

    /// Gets the highest commit index processed by the consensus commit handler.
    pub fn highest_handled_commit(&self) -> CommitIndex {
        *self.highest_handled_commit.borrow()
    }

    /// Updates the highest commit index processed by the consensus commit handler.
    pub fn set_highest_handled_commit(&self, highest_handled_commit: CommitIndex) {
        debug!("Highest handled commit set to {}", highest_handled_commit);
        let previous = self.highest_handled_commit();
        let updated = self.highest_handled_commit.send_if_modified(|current| {
            if highest_handled_commit > *current {
                *current = highest_handled_commit;
                true
            } else {
                false
            }
        });
        if updated && !self.explicit_durable_tracking {
            self.set_highest_durable_commit(highest_handled_commit);
        }
        let recovery_status = *self.recovery_status.borrow();
        if updated
            && recovery_status.started
            && highest_handled_commit < recovery_status.target_commit
            && highest_handled_commit / 10_000 > previous / 10_000
        {
            info!(
                target: "rtd_startup",
                processed_commit = highest_handled_commit,
                target_commit = recovery_status.target_commit,
                remaining_commits = recovery_status
                    .target_commit
                    .saturating_sub(highest_handled_commit),
                "Startup consensus replay is being applied"
            );
        }
        if updated
            && recovery_status.started
            && previous < recovery_status.target_commit
            && highest_handled_commit >= recovery_status.target_commit
        {
            info!(
                target: "rtd_startup",
                processed_commit = highest_handled_commit,
                target_commit = recovery_status.target_commit,
                "Startup consensus replay has reached its persisted target"
            );
        }
    }

    /// Gets the highest commit whose consumer output is durable across process crashes.
    pub fn highest_durable_commit(&self) -> CommitIndex {
        *self.highest_durable_commit.borrow()
    }

    /// Updates the crash-safe consumer watermark after the containing database batch commits.
    pub fn set_highest_durable_commit(&self, highest_durable_commit: CommitIndex) {
        debug!(
            "Highest durable consumer commit set to {}",
            highest_durable_commit
        );
        let previous = self.highest_durable_commit();
        let updated = self.highest_durable_commit.send_if_modified(|current| {
            if highest_durable_commit > *current {
                *current = highest_durable_commit;
                true
            } else {
                false
            }
        });
        if updated && highest_durable_commit / 10_000 > previous / 10_000 {
            info!(
                target: "rtd_startup",
                durable_commit = highest_durable_commit,
                "Crash-safe consensus cursor advanced"
            );
        }
    }

    fn maximum_allowed_consensus_head(&self) -> CommitIndex {
        let highest_durable_commit = self.highest_durable_commit();
        let normal_limit = highest_durable_commit.saturating_add(MAX_PENDING_DURABLE_COMMITS);
        let recovery_status = *self.recovery_status.borrow();
        let recovery_drain_gate = self.recovery_drain_gate.lock();

        if let Some(recovery_drain_ceiling) = self.recovery_drain_ceiling
            && recovery_drain_gate.active
            && recovery_status.started
            && recovery_status.scan_complete
        {
            // A database created before this guard can already be far beyond `normal_limit`.
            // After its ordered replay has been enqueued, permit commits only up to a ceiling
            // fixed and persisted on the first migration attempt. Keep that ceiling active until
            // the startup target itself is durable and all work currently pending in Core
            // fits in the normal window. The startup target is only the old consensus head and can
            // fall behind newly-created signature commits, so reaching `D + window >= target` is
            // not sufficient. `try_finish_recovery_drain` performs the stricter transition.
            normal_limit.max(recovery_drain_ceiling)
        } else {
            normal_limit
        }
    }

    fn try_finish_recovery_drain(&self, highest_pending_consensus_head: CommitIndex) {
        let recovery_status = *self.recovery_status.borrow();
        let highest_durable_commit = self.highest_durable_commit();
        let normal_limit = highest_durable_commit.saturating_add(MAX_PENDING_DURABLE_COMMITS);
        let mut recovery_drain_gate = self.recovery_drain_gate.lock();
        let highest_pending_consensus_head =
            highest_pending_consensus_head.max(recovery_drain_gate.highest_reserved_consensus_head);
        if recovery_drain_gate.active
            && recovery_status.started
            && recovery_status.scan_complete
            && highest_durable_commit >= recovery_status.target_commit
            && highest_pending_consensus_head <= normal_limit
        {
            recovery_drain_gate.active = false;
            info!(
                target: "rtd_startup",
                startup_target_commit = recovery_status.target_commit,
                durable_commit = highest_durable_commit,
                consensus_head = highest_pending_consensus_head,
                "Legacy consensus recovery drain is complete; normal crash-safe backpressure is now active"
            );
        }
    }

    fn try_reserve_consensus_head(&self, target_consensus_head: CommitIndex) -> bool {
        let highest_durable_commit = self.highest_durable_commit();
        let normal_limit = highest_durable_commit.saturating_add(MAX_PENDING_DURABLE_COMMITS);
        let recovery_status = *self.recovery_status.borrow();
        let mut recovery_drain_gate = self.recovery_drain_gate.lock();
        let maximum_allowed_consensus_head = if let Some(recovery_drain_ceiling) =
            self.recovery_drain_ceiling
            && recovery_drain_gate.active
            && recovery_status.started
            && recovery_status.scan_complete
        {
            normal_limit.max(recovery_drain_ceiling)
        } else {
            normal_limit
        };

        if target_consensus_head > maximum_allowed_consensus_head {
            return false;
        }

        recovery_drain_gate.highest_reserved_consensus_head = recovery_drain_gate
            .highest_reserved_consensus_head
            .max(target_consensus_head);
        true
    }

    /// Returns how many additional live commits consensus may persist without exceeding the
    /// crash-safe replay window (or the fixed drain allowance for a pre-existing oversized tail).
    pub(crate) fn remaining_durable_commit_capacity(
        &self,
        consensus_head: CommitIndex,
        highest_pending_consensus_head: CommitIndex,
    ) -> CommitIndex {
        debug_assert!(highest_pending_consensus_head >= consensus_head);
        self.try_finish_recovery_drain(highest_pending_consensus_head);
        self.maximum_allowed_consensus_head()
            .saturating_sub(consensus_head)
    }

    /// Wait until persisting `target_consensus_head` would stay within the bounded replay window.
    pub(crate) async fn wait_for_durable_commit_capacity(
        &self,
        target_consensus_head: CommitIndex,
    ) {
        let mut durable_rx = self.highest_durable_commit.subscribe();
        let mut recovery_rx = self.recovery_status.subscribe();
        loop {
            if self.try_reserve_consensus_head(target_consensus_head) {
                return;
            }
            tokio::select! {
                result = durable_rx.changed() => {
                    result.expect("commit consumer durable-progress sender cannot be dropped");
                }
                result = recovery_rx.changed() => {
                    result.expect("commit recovery-status sender cannot be dropped");
                }
            }
        }
    }

    pub(crate) fn start_recovery(&self, target_commit: CommitIndex) {
        let mut recovery_drain_gate = self.recovery_drain_gate.lock();
        recovery_drain_gate.highest_reserved_consensus_head = recovery_drain_gate
            .highest_reserved_consensus_head
            .max(target_commit);
        drop(recovery_drain_gate);
        self.recovery_status.send_replace(CommitRecoveryStatus {
            target_commit,
            started: true,
            scan_complete: false,
        });
    }

    pub(crate) fn finish_recovery_scan(&self) {
        self.recovery_status.send_modify(|status| {
            assert!(status.started, "consensus recovery was not started");
            status.scan_complete = true;
        });
        let status = *self.recovery_status.borrow();
        let processed_commit = self.highest_handled_commit();
        info!(
            target: "rtd_startup",
            processed_commit,
            target_commit = status.target_commit,
            remaining_commits = status.target_commit.saturating_sub(processed_commit),
            "Consensus database scan is complete; waiting for replayed commits to finish applying"
        );
    }

    pub fn is_recovery_complete(&self) -> bool {
        let status = *self.recovery_status.borrow();
        status.started
            && status.scan_complete
            && self.highest_handled_commit() >= status.target_commit
    }

    pub async fn wait_for_recovery_complete(&self) {
        let mut recovery_rx = self.recovery_status.subscribe();
        let mut handled_rx = self.highest_handled_commit.subscribe();
        loop {
            let status = *recovery_rx.borrow_and_update();
            let highest_handled_commit = *handled_rx.borrow_and_update();
            if status.started
                && status.scan_complete
                && highest_handled_commit >= status.target_commit
            {
                return;
            }
            tokio::select! {
                result = recovery_rx.changed() => {
                    result.expect("commit recovery-status sender cannot be dropped");
                }
                result = handled_rx.changed() => {
                    result.expect("commit consumer progress sender cannot be dropped");
                }
            }
        }
    }

    /// Waits for consensus to replay commits until the consumer last processed commit index.
    pub async fn replay_to_consumer_last_processed_commit_complete(&self) {
        let mut rx = self.highest_handled_commit.subscribe();
        loop {
            let highest_handled = *rx.borrow_and_update();
            if highest_handled >= self.consumer_last_processed_commit_index {
                return;
            }
            rx.changed().await.unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::CommitConsumerMonitor;

    #[test]
    fn test_commit_consumer_monitor() {
        let monitor = CommitConsumerMonitor::new(0, 10);
        assert_eq!(monitor.highest_handled_commit(), 0);

        monitor.set_highest_handled_commit(100);
        assert_eq!(monitor.highest_handled_commit(), 100);
        assert_eq!(monitor.highest_durable_commit(), 100);
    }

    #[tokio::test]
    async fn durable_capacity_waits_for_crash_safe_progress() {
        let monitor = std::sync::Arc::new(CommitConsumerMonitor::new_with_recovery_drain_ceiling(
            0, 10, None,
        ));
        assert_eq!(monitor.highest_durable_commit(), 10);

        let waiter = tokio::spawn({
            let monitor = monitor.clone();
            async move {
                monitor
                    .wait_for_durable_commit_capacity(10 + super::MAX_PENDING_DURABLE_COMMITS + 1)
                    .await;
            }
        });
        tokio::task::yield_now().await;
        assert!(!waiter.is_finished());

        monitor.set_highest_durable_commit(11);
        waiter.await.unwrap();
    }

    #[tokio::test]
    async fn recovery_completes_only_after_scan_and_consumer_handling() {
        let monitor = std::sync::Arc::new(CommitConsumerMonitor::new_with_recovery_drain_ceiling(
            5, 10, None,
        ));
        monitor.start_recovery(20);
        monitor.set_highest_handled_commit(20);
        assert!(!monitor.is_recovery_complete());

        monitor.finish_recovery_scan();
        monitor.wait_for_recovery_complete().await;
        assert!(monitor.is_recovery_complete());
    }

    #[tokio::test]
    async fn oversized_existing_tail_gets_only_its_persisted_post_scan_ceiling() {
        let durable = 10;
        let target = durable + super::MAX_PENDING_DURABLE_COMMITS + 5_000;
        let recovery_drain_ceiling = target + 8_000;
        let monitor = std::sync::Arc::new(CommitConsumerMonitor::new_with_recovery_drain_ceiling(
            0,
            durable,
            Some(recovery_drain_ceiling),
        ));
        monitor.start_recovery(target);

        assert_eq!(monitor.remaining_durable_commit_capacity(target, target), 0);
        let waiter = tokio::spawn({
            let monitor = monitor.clone();
            async move {
                monitor.wait_for_durable_commit_capacity(target + 1).await;
            }
        });
        tokio::task::yield_now().await;
        assert!(!waiter.is_finished());

        monitor.finish_recovery_scan();
        waiter.await.unwrap();
        assert_eq!(
            monitor.remaining_durable_commit_capacity(target, target),
            recovery_drain_ceiling - target
        );
        assert_eq!(
            monitor
                .remaining_durable_commit_capacity(recovery_drain_ceiling, recovery_drain_ceiling),
            0
        );

        // Reaching the old startup target is not enough when live consensus has advanced beyond
        // the normal window to sequence checkpoint signatures.
        let live_consensus_head = target + 2_000;
        monitor.set_highest_durable_commit(target);
        assert!(monitor.try_reserve_consensus_head(live_consensus_head));
        assert_eq!(
            monitor.remaining_durable_commit_capacity(target, target),
            recovery_drain_ceiling - target,
        );

        // Once every startup commit is durable and Core's actual pending head is back inside the
        // normal window, retire the legacy allowance permanently for this process.
        monitor.set_highest_durable_commit(live_consensus_head - 500);
        assert_eq!(
            monitor.remaining_durable_commit_capacity(live_consensus_head, live_consensus_head),
            super::MAX_PENDING_DURABLE_COMMITS - 500,
        );
        assert_eq!(
            monitor
                .remaining_durable_commit_capacity(recovery_drain_ceiling, recovery_drain_ceiling),
            0,
            "a retired drain allowance must not reopen"
        );

        let restarted_target = recovery_drain_ceiling;
        let restarted = CommitConsumerMonitor::new_with_recovery_drain_ceiling(
            0,
            durable,
            Some(recovery_drain_ceiling),
        );
        restarted.start_recovery(restarted_target);
        restarted.finish_recovery_scan();
        assert_eq!(
            restarted.remaining_durable_commit_capacity(restarted_target, restarted_target),
            0,
            "a restart must not grant another drain budget"
        );
    }
}
