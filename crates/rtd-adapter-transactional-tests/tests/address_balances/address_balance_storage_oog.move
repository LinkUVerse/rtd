// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Storage-OOG reset + re-smash for pure `[AddressBalance]` gas payment. The
// workload creates many new objects; with a tight budget, storage charging
// fails, triggering reset + re-smash. Verifies the re-smash invariant holds
// and the post-reset accumulator event is emitted cleanly without
// duplication.

//# init --addresses test=0x0 --accounts A B

//# publish
module test::oog;
public struct W has key, store { id: UID }
public fun make(n: u64, ctx: &mut TxContext) {
    let mut i = 0;
    while (i < n) {
        rtd::transfer::public_transfer(W { id: object::new(ctx) }, ctx.sender());
        i = i + 1;
    }
}

// Seed A's address balance.
//# programmable --sender A --inputs 100000000000 @A
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::into_balance<rtd::rtd::RTD>(Result(0));
//> 2: rtd::balance::send_funds<rtd::rtd::RTD>(Result(1), Input(1));

//# create-checkpoint

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> A

// Pure address-balance gas payment; large object count exceeds budget storage.
//# programmable --sender A --gas-budget 5000000 --gas-payment withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(5000000) --inputs 100
//> test::oog::make(Input(0))

//# create-checkpoint

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> A
