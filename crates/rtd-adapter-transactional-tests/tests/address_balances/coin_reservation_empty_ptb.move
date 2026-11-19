// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// A legacy coin reservation input in a PTB with no commands of its own. The
// conversion to a `Coin` and the send-back of the unused coin are both injected
// during typing, so every executed command is annotated with an original command
// index that does not exist.

//# init --addresses test=0x0 --accounts A B

// Seed A's address balance.
//# programmable --sender A --inputs 100000000000 @A
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::into_balance<rtd::rtd::RTD>(Result(0));
//> 2: rtd::balance::send_funds<rtd::rtd::RTD>(Result(1), Input(1));

//# create-checkpoint

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> A

//# programmable --sender A --inputs coin_reservation<rtd::balance::Balance<rtd::rtd::RTD>>(500000000)

//# create-checkpoint

// The reservation is withdrawn and immediately sent back, so A's balance is unchanged.
//# view-funds rtd::balance::Balance<rtd::rtd::RTD> A
