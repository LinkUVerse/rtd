// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// A legacy coin reservation input consumed by a command of the PTB. Only the
// conversion to a `Coin` is injected, ahead of the original command.

//# init --addresses test=0x0 --accounts A B

// Seed A's address balance.
//# programmable --sender A --inputs 100000000000 @A
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::into_balance<rtd::rtd::RTD>(Result(0));
//> 2: rtd::balance::send_funds<rtd::rtd::RTD>(Result(1), Input(1));

//# create-checkpoint

//# programmable --sender A --inputs coin_reservation<rtd::balance::Balance<rtd::rtd::RTD>>(500000000) @B
//> TransferObjects([Input(0)], Input(1))

//# create-checkpoint

// The reserved amount left A's balance as a coin now owned by B.
//# view-funds rtd::balance::Balance<rtd::rtd::RTD> A
