// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::checkpoints::CheckpointStore;
use crate::rpc_index::RpcIndexStore;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum FullnodeReadinessLag {
    #[error("embedded validator network startup has not completed")]
    NetworkStartup,
    #[error("pending transaction recovery has not started")]
    PendingRecovery,
    #[error("executed checkpoint is behind the startup target")]
    ExecutedCheckpoint,
    #[error("object state is behind the startup target")]
    ObjectState,
    #[error("secondary index is behind the startup target")]
    SecondaryIndex,
    #[error("RPC index is behind the startup target")]
    RpcIndex,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FullnodeReadinessStatus {
    pub network_startup_complete: bool,
    pub startup_target: u64,
    pub highest_executed_checkpoint: Option<u64>,
    pub object_state_checkpoint: Option<u64>,
    pub secondary_index_checkpoint: Option<u64>,
    pub rpc_index_checkpoint: Option<u64>,
    pub secondary_index_required: bool,
    pub rpc_index_required: bool,
    pub pending_recovery_started: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FullnodeCatchingUp {
    pub lag: FullnodeReadinessLag,
    pub status: FullnodeReadinessStatus,
}

impl fmt::Display for FullnodeCatchingUp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Fullnode is catching up: ")?;
        match self.lag {
            FullnodeReadinessLag::NetworkStartup => {
                formatter.write_str("validator network recovery is still in progress")
            }
            FullnodeReadinessLag::PendingRecovery => {
                formatter.write_str("pending transaction recovery has not started")
            }
            FullnodeReadinessLag::ExecutedCheckpoint => write_checkpoint_progress(
                formatter,
                "executed checkpoint",
                self.status.highest_executed_checkpoint,
                self.status.startup_target,
            ),
            FullnodeReadinessLag::ObjectState => write_checkpoint_progress(
                formatter,
                "object state checkpoint",
                self.status.object_state_checkpoint,
                self.status.startup_target,
            ),
            FullnodeReadinessLag::SecondaryIndex => write_checkpoint_progress(
                formatter,
                "secondary index checkpoint",
                self.status.secondary_index_checkpoint,
                self.status.startup_target,
            ),
            FullnodeReadinessLag::RpcIndex => write_checkpoint_progress(
                formatter,
                "RPC index checkpoint",
                self.status.rpc_index_checkpoint,
                self.status.startup_target,
            ),
        }
    }
}

impl std::error::Error for FullnodeCatchingUp {}

fn write_checkpoint_progress(
    formatter: &mut fmt::Formatter<'_>,
    checkpoint_name: &str,
    current: Option<u64>,
    target: u64,
) -> fmt::Result {
    match current {
        Some(current) => write!(formatter, "{checkpoint_name} is at {current}/{target}"),
        None => write!(
            formatter,
            "{checkpoint_name} is unavailable; startup target is {target}"
        ),
    }
}

impl FullnodeReadinessStatus {
    pub fn ensure_ready(self) -> Result<(), FullnodeCatchingUp> {
        let lag = if !self.network_startup_complete {
            Some(FullnodeReadinessLag::NetworkStartup)
        } else if !self.pending_recovery_started {
            Some(FullnodeReadinessLag::PendingRecovery)
        } else if !checkpoint_reached(self.highest_executed_checkpoint, self.startup_target) {
            Some(FullnodeReadinessLag::ExecutedCheckpoint)
        } else if !checkpoint_reached(self.object_state_checkpoint, self.startup_target) {
            Some(FullnodeReadinessLag::ObjectState)
        } else if self.secondary_index_required
            && !checkpoint_reached(self.secondary_index_checkpoint, self.startup_target)
        {
            Some(FullnodeReadinessLag::SecondaryIndex)
        } else if self.rpc_index_required
            && !checkpoint_reached(self.rpc_index_checkpoint, self.startup_target)
        {
            Some(FullnodeReadinessLag::RpcIndex)
        } else {
            None
        };

        match lag {
            Some(lag) => Err(FullnodeCatchingUp { lag, status: self }),
            None => Ok(()),
        }
    }
}

fn checkpoint_reached(current: Option<u64>, target: u64) -> bool {
    current.is_some_and(|checkpoint| checkpoint >= target)
}

pub struct FullnodeReadiness {
    startup_target: u64,
    checkpoint_store: Arc<CheckpointStore>,
    rpc_index: Option<Arc<RpcIndexStore>>,
    secondary_index_required: bool,
    network_startup_complete: AtomicBool,
    pending_recovery_started: AtomicBool,
}

impl FullnodeReadiness {
    pub fn new(
        startup_target: u64,
        checkpoint_store: Arc<CheckpointStore>,
        rpc_index: Option<Arc<RpcIndexStore>>,
        secondary_index_required: bool,
        network_startup_required: bool,
        pending_recovery_required: bool,
    ) -> Self {
        Self {
            startup_target,
            checkpoint_store,
            rpc_index,
            secondary_index_required,
            network_startup_complete: AtomicBool::new(!network_startup_required),
            pending_recovery_started: AtomicBool::new(!pending_recovery_required),
        }
    }

    pub fn startup_target(&self) -> u64 {
        self.startup_target
    }

    pub fn mark_pending_recovery_started(&self) {
        self.pending_recovery_started.store(true, Ordering::Release);
    }

    pub fn mark_network_startup_complete(&self) {
        self.network_startup_complete.store(true, Ordering::Release);
    }

    pub fn status(&self) -> FullnodeReadinessStatus {
        let highest_executed_checkpoint = self
            .checkpoint_store
            .get_highest_executed_checkpoint_seq_number()
            .ok()
            .flatten();
        let rpc_index_checkpoint = self.rpc_index.as_ref().and_then(|index| {
            index
                .get_highest_indexed_checkpoint_seq_number()
                .ok()
                .flatten()
        });

        // HighestExecuted is bumped only after object writes, synchronous secondary indexing,
        // and the RPC-index checkpoint commit have completed.
        FullnodeReadinessStatus {
            network_startup_complete: self.network_startup_complete.load(Ordering::Acquire),
            startup_target: self.startup_target,
            highest_executed_checkpoint,
            object_state_checkpoint: highest_executed_checkpoint,
            secondary_index_checkpoint: self
                .secondary_index_required
                .then_some(highest_executed_checkpoint)
                .flatten(),
            rpc_index_checkpoint,
            secondary_index_required: self.secondary_index_required,
            rpc_index_required: self.rpc_index.is_some(),
            pending_recovery_started: self.pending_recovery_started.load(Ordering::Acquire),
        }
    }

    pub fn ensure_ready(&self) -> Result<(), FullnodeCatchingUp> {
        self.status().ensure_ready()
    }

    pub fn is_ready(&self) -> bool {
        self.ensure_ready().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::{FullnodeReadiness, FullnodeReadinessLag, FullnodeReadinessStatus};
    use crate::authority::authority_store_pruner::PrunerWatermarks;
    use crate::checkpoints::CheckpointStore;
    use rtd_types::messages_checkpoint::VerifiedCheckpoint;
    use rtd_types::test_checkpoint_data_builder::TestCheckpointBuilder;
    use std::sync::Arc;

    fn ready_status() -> FullnodeReadinessStatus {
        FullnodeReadinessStatus {
            network_startup_complete: true,
            startup_target: 42,
            highest_executed_checkpoint: Some(42),
            object_state_checkpoint: Some(42),
            secondary_index_checkpoint: Some(42),
            rpc_index_checkpoint: Some(42),
            secondary_index_required: true,
            rpc_index_required: true,
            pending_recovery_started: true,
        }
    }

    #[test]
    fn embedded_validator_network_must_start_before_fullnode_is_ready() {
        let mut status = ready_status();
        status.network_startup_complete = false;

        let error = status.ensure_ready().unwrap_err();

        assert_eq!(error.lag, FullnodeReadinessLag::NetworkStartup);
        assert_eq!(
            error.to_string(),
            "Fullnode is catching up: validator network recovery is still in progress"
        );
    }

    #[test]
    fn pending_recovery_must_start_before_fullnode_is_ready() {
        let mut status = ready_status();
        status.pending_recovery_started = false;

        let error = status.ensure_ready().unwrap_err();

        assert_eq!(error.lag, FullnodeReadinessLag::PendingRecovery);
    }

    #[test]
    fn executed_checkpoint_must_reach_startup_target() {
        let mut status = ready_status();
        status.highest_executed_checkpoint = Some(41);

        let error = status.ensure_ready().unwrap_err();

        assert_eq!(error.lag, FullnodeReadinessLag::ExecutedCheckpoint);
    }

    #[test]
    fn object_state_must_reach_startup_target() {
        let mut status = ready_status();
        status.object_state_checkpoint = Some(41);

        let error = status.ensure_ready().unwrap_err();

        assert_eq!(error.lag, FullnodeReadinessLag::ObjectState);
    }

    #[test]
    fn enabled_secondary_index_must_reach_startup_target() {
        let mut status = ready_status();
        status.secondary_index_checkpoint = Some(41);

        let error = status.ensure_ready().unwrap_err();

        assert_eq!(error.lag, FullnodeReadinessLag::SecondaryIndex);
    }

    #[test]
    fn enabled_rpc_index_must_reach_startup_target() {
        let mut status = ready_status();
        status.rpc_index_checkpoint = Some(41);

        let error = status.ensure_ready().unwrap_err();

        assert_eq!(error.lag, FullnodeReadinessLag::RpcIndex);
    }

    #[test]
    fn fullnode_is_ready_when_all_required_watermarks_reach_target() {
        ready_status().ensure_ready().unwrap();
    }

    #[test]
    fn catching_up_error_uses_human_readable_checkpoint_progress() {
        let mut status = ready_status();
        status.rpc_index_checkpoint = None;

        let message = status.ensure_ready().unwrap_err().to_string();

        assert_eq!(
            message,
            "Fullnode is catching up: RPC index checkpoint is unavailable; startup target is 42"
        );
        assert!(!message.contains("FullnodeReadinessStatus"));
        assert!(!message.contains("Some("));
    }

    #[tokio::test]
    async fn shared_readiness_tracks_pending_recovery_and_checkpoint_progress() {
        let directory = linku_common::tempdir().unwrap();
        let checkpoint_store =
            CheckpointStore::new(directory.path(), Arc::new(PrunerWatermarks::default()));
        let mut builder = TestCheckpointBuilder::new(0);
        let genesis = VerifiedCheckpoint::new_unchecked(builder.build_checkpoint().summary);
        checkpoint_store
            .insert_verified_checkpoint(&genesis)
            .unwrap();
        checkpoint_store
            .update_highest_executed_checkpoint(&genesis)
            .unwrap();
        let readiness =
            FullnodeReadiness::new(1, checkpoint_store.clone(), None, false, false, true);

        assert_eq!(
            readiness.ensure_ready().unwrap_err().lag,
            FullnodeReadinessLag::PendingRecovery
        );
        readiness.mark_pending_recovery_started();
        assert_eq!(
            readiness.ensure_ready().unwrap_err().lag,
            FullnodeReadinessLag::ExecutedCheckpoint
        );

        let checkpoint = VerifiedCheckpoint::new_unchecked(builder.build_checkpoint().summary);
        checkpoint_store
            .insert_verified_checkpoint(&checkpoint)
            .unwrap();
        checkpoint_store
            .update_highest_executed_checkpoint(&checkpoint)
            .unwrap();

        readiness.ensure_ready().unwrap();
        assert_eq!(readiness.status().object_state_checkpoint, Some(1));
    }
}
