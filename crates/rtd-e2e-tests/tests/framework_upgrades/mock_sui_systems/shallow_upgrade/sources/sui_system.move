// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module rtd_system::rtd_system {
    use std::vector;

    use rtd::balance::Balance;
    use rtd::object::UID;
    use rtd::rtd::RTD;
    use rtd::transfer;
    use rtd::tx_context::{Self, TxContext};
    use rtd::dynamic_field;

    use rtd_system::validator::Validator;
    use rtd_system::rtd_system_state_inner::{Self, RtdSystemStateInner, RtdSystemStateInnerV2};

    public struct RtdSystemState has key {
        id: UID,
        version: u64,
    }

    public(package) fun create(
        id: UID,
        validators: vector<Validator>,
        storage_fund: Balance<RTD>,
        protocol_version: u64,
        epoch_start_timestamp_ms: u64,
        epoch_duration_ms: u64,
        ctx: &mut TxContext,
    ) {
        let system_state = rtd_system_state_inner::create(
            validators,
            storage_fund,
            protocol_version,
            epoch_start_timestamp_ms,
            epoch_duration_ms,
            ctx,
        );
        let version = rtd_system_state_inner::genesis_system_state_version();
        let mut self = RtdSystemState {
            id,
            version,
        };
        dynamic_field::add(&mut self.id, version, system_state);
        transfer::share_object(self);
    }

    fun advance_epoch(
        storage_reward: Balance<RTD>,
        computation_reward: Balance<RTD>,
        wrapper: &mut RtdSystemState,
        new_epoch: u64,
        next_protocol_version: u64,
        storage_rebate: u64,
        _non_refundable_storage_fee: u64,
        _storage_fund_reinvest_rate: u64, // share of storage fund's rewards that's reinvested
                                         // into storage fund, in basis point.
        _reward_slashing_rate: u64, // how much rewards are slashed to punish a validator, in bps.
        epoch_start_timestamp_ms: u64, // Timestamp of the epoch start
        ctx: &mut TxContext,
    ) : Balance<RTD> {
        let self = load_system_state_mut(wrapper);
        assert!(tx_context::sender(ctx) == @0x0, 0);
        let storage_rebate = rtd_system_state_inner::advance_epoch(
            self,
            new_epoch,
            next_protocol_version,
            storage_reward,
            computation_reward,
            storage_rebate,
            epoch_start_timestamp_ms,
        );

        storage_rebate
    }

    public fun active_validator_addresses(wrapper: &mut RtdSystemState): vector<address> {
        vector::empty()
    }

    fun load_system_state_mut(self: &mut RtdSystemState): &mut RtdSystemStateInnerV2 {
        load_inner_maybe_upgrade(self)
    }

    fun load_inner_maybe_upgrade(self: &mut RtdSystemState): &mut RtdSystemStateInnerV2 {
        let mut version = self.version;
        if (version == rtd_system_state_inner::genesis_system_state_version()) {
            let inner: RtdSystemStateInner = dynamic_field::remove(&mut self.id, version);
            let new_inner = rtd_system_state_inner::v1_to_v2(inner);
            version = rtd_system_state_inner::system_state_version(&new_inner);
            dynamic_field::add(&mut self.id, version, new_inner);
            self.version = version;
        };

        let inner: &mut RtdSystemStateInnerV2 = dynamic_field::borrow_mut(&mut self.id, version);
        assert!(rtd_system_state_inner::system_state_version(inner) == version, 0);
        inner
    }

    fun store_execution_time_estimates(wrapper: &mut RtdSystemState, estimates_bytes: vector<u8>) {
        let self = load_system_state_mut(wrapper);
        rtd_system_state_inner::store_execution_time_estimates(self, estimates_bytes)
    }

    fun store_execution_time_estimates_v2(wrapper: &mut RtdSystemState, estimate_chunks: vector<vector<u8>>) {
        let self = load_system_state_mut(wrapper);
        rtd_system_state_inner::store_execution_time_estimates_v2(self, estimate_chunks)
    }
}
