// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use rtd_types::{
    accumulator_root::AccumulatorObjId,
    base_types::SequenceNumber,
    error::{RtdErrorKind, RtdResult, UserInputError},
};

pub trait AccountFundsRead: Send + Sync {
    /// Gets the unsequenced latest amount in an account, or zero when it does not exist.
    fn get_latest_account_amount(&self, account_id: &AccumulatorObjId) -> u128;

    /// Gets an account amount paired with a stable accumulator root version.
    fn get_consistent_latest_account_amount_and_version(
        &self,
        account_id: &AccumulatorObjId,
    ) -> (u128, SequenceNumber);

    /// Read the amount at a precise version. Care must be taken to only call this function if we
    /// can guarantee that objects behind this version have not yet been pruned.
    fn get_account_amount_at_version(
        &self,
        account_id: &AccumulatorObjId,
        version: SequenceNumber,
    ) -> u128;

    /// Checks if given amounts are available in the latest versions of the referenced acccumulator
    /// objects. This does un-sequenced reads and can only be used on the signing/voting path
    /// where deterministic results are not required.
    fn check_amounts_available(
        &self,
        requested_amounts: &BTreeMap<AccumulatorObjId, u64>,
    ) -> RtdResult {
        for (object_id, requested_amount) in requested_amounts {
            let actual_amount = self.get_latest_account_amount(object_id);

            if actual_amount < *requested_amount as u128 {
                return Err(RtdErrorKind::UserInputError {
                    error: UserInputError::InvalidWithdrawReservation {
                        error: format!(
                            "Available amount in account for object id {} is less than requested: {} < {}",
                            object_id, actual_amount, requested_amount
                        ),
                    },
                }
                .into());
            }
        }

        Ok(())
    }
}
