// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module rtd_system::rtd_system_state_inner {
    use rtd::balance::{Self, Balance};
    use rtd::rtd::RTD;
    use rtd::tx_context::TxContext;
    use rtd::bag::{Self, Bag};
    use rtd::table::{Self, Table};
    use rtd::object::ID;

    use rtd_system::validator::Validator;
    use rtd_system::validator_wrapper::ValidatorWrapper;

    const SYSTEM_STATE_VERSION_V1: u64 = 18446744073709551605;  // u64::MAX - 10

    public struct SystemParameters has store {
        epoch_duration_ms: u64,
        extra_fields: Bag,
    }

    public struct ValidatorSet has store {
        active_validators: vector<Validator>,
        inactive_validators: Table<ID, ValidatorWrapper>,
        extra_fields: Bag,
    }

    public struct RtdSystemStateInner has store {
        epoch: u64,
        protocol_version: u64,
        system_state_version: u64,
        validators: ValidatorSet,
        storage_fund: Balance<RTD>,
        parameters: SystemParameters,
        reference_gas_price: u64,
        safe_mode: bool,
        epoch_start_timestamp_ms: u64,
        extra_fields: Bag,
    }

    public(package) fun create(
        validators: vector<Validator>,
        storage_fund: Balance<RTD>,
        protocol_version: u64,
        epoch_start_timestamp_ms: u64,
        epoch_duration_ms: u64,
        ctx: &mut TxContext,
    ): RtdSystemStateInner {
        let system_state = RtdSystemStateInner {
            epoch: 0,
            protocol_version,
            system_state_version: genesis_system_state_version(),
            validators: ValidatorSet {
                active_validators: validators,
                inactive_validators: table::new(ctx),
                extra_fields: bag::new(ctx),
            },
            storage_fund,
            parameters: SystemParameters {
                epoch_duration_ms,
                extra_fields: bag::new(ctx),
            },
            reference_gas_price: 1,
            safe_mode: false,
            epoch_start_timestamp_ms,
            extra_fields: bag::new(ctx),
        };
        system_state
    }

    public(package) fun advance_epoch(
        self: &mut RtdSystemStateInner,
        storage_reward: Balance<RTD>,
        computation_reward: Balance<RTD>,
        storage_rebate_amount: u64,
    ) : Balance<RTD> {
        balance::join(&mut self.storage_fund, computation_reward);
        balance::join(&mut self.storage_fund, storage_reward);
        let storage_rebate = balance::split(&mut self.storage_fund, storage_rebate_amount);
        storage_rebate
    }

    public(package) fun genesis_system_state_version(): u64 {
        SYSTEM_STATE_VERSION_V1
    }
}
