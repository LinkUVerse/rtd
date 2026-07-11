// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use parking_lot::RwLock;
use rtd_types::{
    accumulator_root::AccumulatorObjId, base_types::SequenceNumber,
    execution_params::FundsWithdrawStatus,
};
use tokio::sync::{oneshot, watch};
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::{
    accumulators::funds_read::AccountFundsRead,
    execution_scheduler::funds_withdraw_scheduler::object_funds::{
        ObjectFundsWithdrawSchedulerTrait, ObjectFundsWithdrawStatus,
    },
};

#[derive(Clone)]
pub(crate) struct NaiveObjectFundsWithdrawScheduler {
    funds_read: Arc<dyn AccountFundsRead>,
    inner: Arc<RwLock<Inner>>,
    accumulator_version_sender: Arc<watch::Sender<SequenceNumber>>,
    // We must keep a receiver alive to make sure sends go through and can update the last settled version.
    accumulator_version_receiver: Arc<watch::Receiver<SequenceNumber>>,
    epoch_ended: Arc<CancellationToken>,
}

#[derive(Default)]
struct Inner {
    /// Pending unsettled withdraws for each account and accumulator version.
    /// We must track these because when we execute a transaction, the witdhraws are not immediately settled,
    /// so we need to track them and check them again when we execute the next transaction from the same consensus commit.
    unsettled_withdraws: BTreeMap<AccumulatorObjId, BTreeMap<SequenceNumber, u128>>,
    /// Accounts with pending withdraws at each version, used to garbage collect committed reservations.
    unsettled_accounts: BTreeMap<SequenceNumber, BTreeSet<AccumulatorObjId>>,
}

impl NaiveObjectFundsWithdrawScheduler {
    pub fn new(
        funds_read: Arc<dyn AccountFundsRead>,
        starting_accumulator_version: SequenceNumber,
    ) -> Self {
        let (accumulator_version_sender, accumulator_version_receiver) =
            watch::channel(starting_accumulator_version);
        Self {
            funds_read,
            inner: Arc::new(RwLock::new(Inner::default())),
            accumulator_version_sender: Arc::new(accumulator_version_sender),
            accumulator_version_receiver: Arc::new(accumulator_version_receiver),
            epoch_ended: Arc::new(CancellationToken::new()),
        }
    }

    fn try_withdraw(
        &self,
        object_withdraws: &BTreeMap<AccumulatorObjId, u64>,
        accumulator_version: SequenceNumber,
    ) -> bool {
        for (obj_id, amount) in object_withdraws {
            // It is safe to get the latest funds here because this function is called during execution,
            // which means this transaction is not committed yet,
            // so the settlement transaction at the end of the same consensus commit cannot have settled yet.
            // That is, we must be blocked by this transaction in order to make progress.
            let funds = self
                .funds_read
                .get_account_amount_at_version(obj_id, accumulator_version);
            let unsettled_withdraw = self
                .inner
                .read()
                .unsettled_withdraws
                .get(obj_id)
                .and_then(|withdraws| withdraws.get(&accumulator_version))
                .copied()
                .unwrap_or_default();
            debug!(
                ?obj_id,
                ?funds,
                ?accumulator_version,
                ?unsettled_withdraw,
                ?amount,
                "Trying to withdraw"
            );
            assert!(funds >= unsettled_withdraw);
            if funds - unsettled_withdraw < *amount as u128 {
                return false;
            }
        }
        let mut inner = self.inner.write();
        for (obj_id, amount) in object_withdraws {
            let entry = inner
                .unsettled_withdraws
                .entry(*obj_id)
                .or_default()
                .entry(accumulator_version)
                .or_default();
            debug!(?obj_id, ?amount, ?entry, "Updating unsettled withdraws");
            *entry = entry.checked_add(*amount as u128).unwrap();

            inner
                .unsettled_accounts
                .entry(accumulator_version)
                .or_default()
                .insert(*obj_id);
        }
        true
    }

    fn return_insufficient_funds() -> ObjectFundsWithdrawStatus {
        let (sender, receiver) = oneshot::channel();
        // unwrap is safe because the receiver is defined right above.
        sender.send(FundsWithdrawStatus::Insufficient).unwrap();
        ObjectFundsWithdrawStatus::Pending(receiver)
    }
}

impl ObjectFundsWithdrawSchedulerTrait for NaiveObjectFundsWithdrawScheduler {
    fn schedule(
        &self,
        object_withdraws: BTreeMap<AccumulatorObjId, u64>,
        accumulator_version: SequenceNumber,
    ) -> ObjectFundsWithdrawStatus {
        let last_settled_version = *self.accumulator_version_receiver.borrow();
        debug!(
            last_settled_version =? last_settled_version.value(),
            withdraw_accumulator_version =? accumulator_version.value(),
            "Scheduling object funds withdraws"
        );
        // It is possible for the settled version to be ahead of the last scheduled version,
        // because settlement transactions that come from checkpoint executor do not depend
        // on the object funds withdraws, and can execute in parallel or in advance.
        if accumulator_version <= last_settled_version {
            if self.try_withdraw(&object_withdraws, accumulator_version) {
                return ObjectFundsWithdrawStatus::SufficientFunds;
            } else {
                return Self::return_insufficient_funds();
            }
        }

        // Spawn a task to wait for the last settled version to become accumulator_version,
        // before we could check again.
        let accumulator_version_sender = self.accumulator_version_sender.clone();
        let epoch_cancel = self.epoch_ended.child_token();
        let (sender, receiver) = oneshot::channel();
        tokio::spawn(async move {
            let mut version_receiver = accumulator_version_sender.subscribe();
            tokio::select! {
                res = version_receiver.wait_for(|v| *v >= accumulator_version) => {
                    if res.is_err() {
                        // This shouldn't happen, but just to be safe.
                        tracing::error!("Accumulator version receiver channel closed while waiting for accumulator version");
                        return;
                    }
                    // We notify the waiter that the funds are now deterministically known,
                    // but we don't need to check here whether they are sufficient or not.
                    // Next time during execution we will check again.
                    let _ = sender.send(FundsWithdrawStatus::MaybeSufficient);
                }
                _ = epoch_cancel.cancelled() => {}
            }
        });
        ObjectFundsWithdrawStatus::Pending(receiver)
    }

    fn settle_accumulator_version(&self, next_accumulator_version: SequenceNumber) {
        // unwrap is safe because the scheduler always holds a reference to the receiver.
        self.accumulator_version_sender
            .send(next_accumulator_version)
            .unwrap();
    }

    fn commit_accumulator_versions(&self, committed_accumulator_versions: Vec<SequenceNumber>) {
        let mut inner = self.inner.write();
        for accumulator_version in committed_accumulator_versions {
            let accounts = inner
                .unsettled_accounts
                .remove(&accumulator_version)
                .unwrap_or_default();
            for account in accounts {
                if let Some(withdraws) = inner.unsettled_withdraws.get_mut(&account) {
                    withdraws.remove(&accumulator_version);
                    if withdraws.is_empty() {
                        inner.unsettled_withdraws.remove(&account);
                    }
                }
            }
        }
    }

    fn close_epoch(&self) {
        self.epoch_ended.cancel();
    }

    #[cfg(test)]
    fn get_current_accumulator_version(&self) -> SequenceNumber {
        *self.accumulator_version_receiver.borrow()
    }
}
