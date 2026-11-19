// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Unsponsored tx, sender A pays with `[AB(A), Coin]`, workload
// `send_funds(Gas, @A)` sends the ephemeral gas coin's value back into A's
// own address balance. At gas finalization, the gas-charge location is
// overridden to the recipient's AB -- the same address as the original
// payer, so the override is logically a no-op.

//# init --addresses test=0x0 --accounts A

// Seed A's address balance.
//# programmable --sender A --inputs 100000000000 @A
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::into_balance<rtd::rtd::RTD>(Result(0));
//> 2: rtd::balance::send_funds<rtd::rtd::RTD>(Result(1), Input(1));

// Create a 1B coin owned by A - this will be the secondary in the smash.
//# programmable --sender A --inputs 1000000000 @A
//> 0: SplitCoins(Gas, [Input(0)]);
//> TransferObjects([Result(0)], Input(1))

//# create-checkpoint

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> A

//# view-object 2,0

// Pay with [AB(500M), Coin(1B)]; send_funds(Gas, @A) - recipient is the
// original AB owner. The override target equals the original gas charge
// location.
//# programmable --sender A --gas-budget 500000000 --gas-payment withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(500000000) --gas-payment object(2,0) --inputs @A
//> rtd::coin::send_funds<rtd::rtd::RTD>(Gas, Input(0))

//# create-checkpoint

// After: A's AB should reflect all accumulator events - deposit-back from
// smashing the secondary coin (Merge: coin value), the send_funds transfer
// of the ephemeral coin to A's AB (Merge: full smashed value), the override-
// driven debit (Split: full smashed value), and the final gas charge.
//# view-funds rtd::balance::Balance<rtd::rtd::RTD> A

//# view-object 2,0
