// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module locked_stake::locked_stake;

use locked_stake::epoch_time_lock::{Self, EpochTimeLock};
use rtd::balance::{Self, Balance};
use rtd::coin;
use rtd::rtd::RTD;
use rtd::vec_map::{Self, VecMap};
use rtd_system::staking_pool::StakedRtd;
use rtd_system::rtd_system::{Self, RtdSystemState};

const EInsufficientBalance: u64 = 0;
const EStakeObjectNonExistent: u64 = 1;

/// An object that locks RTD tokens and stake objects until a given epoch, and allows
/// staking and unstaking operations when locked.
public struct LockedStake has key {
    id: UID,
    staked_rtd: VecMap<ID, StakedRtd>,
    rtd: Balance<RTD>,
    locked_until_epoch: EpochTimeLock,
}

// ============================= basic operations =============================

/// Create a new LockedStake object with empty staked_rtd and rtd balance given a lock time.
/// Aborts if the given epoch has already passed.
public fun new(locked_until_epoch: u64, ctx: &mut TxContext): LockedStake {
    LockedStake {
        id: object::new(ctx),
        staked_rtd: vec_map::empty(),
        rtd: balance::zero(),
        locked_until_epoch: epoch_time_lock::new(locked_until_epoch, ctx),
    }
}

/// Unlocks and returns all the assets stored inside this LockedStake object.
/// Aborts if the unlock epoch is in the future.
public fun unlock(ls: LockedStake, ctx: &TxContext): (VecMap<ID, StakedRtd>, Balance<RTD>) {
    let LockedStake { id, staked_rtd, rtd, locked_until_epoch } = ls;
    epoch_time_lock::destroy(locked_until_epoch, ctx);
    object::delete(id);
    (staked_rtd, rtd)
}

/// Deposit a new stake object to the LockedStake object.
public fun deposit_staked_rtd(ls: &mut LockedStake, staked_rtd: StakedRtd) {
    let id = object::id(&staked_rtd);
    // This insertion can't abort since each object has a unique id.
    vec_map::insert(&mut ls.staked_rtd, id, staked_rtd);
}

/// Deposit rtd balance to the LockedStake object.
public fun deposit_rtd(ls: &mut LockedStake, rtd: Balance<RTD>) {
    balance::join(&mut ls.rtd, rtd);
}

/// Take `amount` of RTD from the rtd balance, stakes it, and puts the stake object
/// back into the staked rtd vec map.
public fun stake(
    ls: &mut LockedStake,
    rtd_system: &mut RtdSystemState,
    amount: u64,
    validator_address: address,
    ctx: &mut TxContext,
) {
    assert!(balance::value(&ls.rtd) >= amount, EInsufficientBalance);
    let stake = rtd_system::request_add_stake_non_entry(
        rtd_system,
        coin::from_balance(balance::split(&mut ls.rtd, amount), ctx),
        validator_address,
        ctx,
    );
    deposit_staked_rtd(ls, stake);
}

/// Unstake the stake object with `staked_rtd_id` and puts the resulting principal
/// and rewards back into the locked rtd balance.
/// Returns the amount of RTD unstaked, including both principal and rewards.
/// Aborts if no stake exists with the given id.
public fun unstake(
    ls: &mut LockedStake,
    rtd_system: &mut RtdSystemState,
    staked_rtd_id: ID,
    ctx: &mut TxContext,
): u64 {
    assert!(vec_map::contains(&ls.staked_rtd, &staked_rtd_id), EStakeObjectNonExistent);
    let (_, stake) = vec_map::remove(&mut ls.staked_rtd, &staked_rtd_id);
    let rtd_balance = rtd_system::request_withdraw_stake_non_entry(rtd_system, stake, ctx);
    let amount = balance::value(&rtd_balance);
    deposit_rtd(ls, rtd_balance);
    amount
}

// ============================= getters =============================

public fun staked_rtd(ls: &LockedStake): &VecMap<ID, StakedRtd> {
    &ls.staked_rtd
}

public fun rtd_balance(ls: &LockedStake): u64 {
    balance::value(&ls.rtd)
}

public fun locked_until_epoch(ls: &LockedStake): u64 {
    epoch_time_lock::epoch(&ls.locked_until_epoch)
}

// TODO: possibly add some scenarios like switching stake, creating a new LockedStake and transferring
// it to the sender, etc. But these can also be done as PTBs.
