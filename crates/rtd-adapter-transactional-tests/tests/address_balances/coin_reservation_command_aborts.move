// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// A legacy coin reservation input in a PTB whose command aborts. The injected
// conversion to a `Coin` runs, then the original command fails.

//# init --addresses test=0x0 --accounts A B

// Seed A's address balance.
//# programmable --sender A --inputs 100000000000 @A
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::into_balance<rtd::rtd::RTD>(Result(0));
//> 2: rtd::balance::send_funds<rtd::rtd::RTD>(Result(1), Input(1));

//# create-checkpoint

// Splitting more than the reserved amount aborts.
//# programmable --sender A --inputs coin_reservation<rtd::balance::Balance<rtd::rtd::RTD>>(500000000) 1000000000 @B
//> 0: rtd::coin::split<rtd::rtd::RTD>(Input(0), Input(1));
//> 1: TransferObjects([Result(0)], Input(2))

//# create-checkpoint

// The failed transaction leaves A's balance unchanged.
//# view-funds rtd::balance::Balance<rtd::rtd::RTD> A
