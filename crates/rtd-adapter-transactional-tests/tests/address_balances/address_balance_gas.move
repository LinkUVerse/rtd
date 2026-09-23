// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Tests that --address-balance-gas pays for gas from the address balance,
// leaving owned gas objects untouched.

//# init --addresses test=0x0 --accounts A

// View gas coin before any transactions
//# view-object 0,0

// First send funds to A's address balance so we can pay for gas from it
//# programmable --sender A --inputs 10000000000 @A
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::into_balance<rtd::rtd::RTD>(Result(0));
//> 2: rtd::balance::send_funds<rtd::rtd::RTD>(Result(1), Input(1));

//# create-checkpoint

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> A

// Empty transaction using address balance gas
//# programmable --sender A --address-balance-gas

// Use the object, but not as gas
//# programmable --sender A --address-balance-gas --inputs object(0,0)

// View gas coin after -- balance should be unchanged (except for the initial send_funds tx)
//# view-object 0,0

//# create-checkpoint

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> A
