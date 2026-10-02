// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Test send_funds and redeem_funds from rtd::balance

//# init --addresses test=0x0 --accounts A B C

// Send 1000000000 from A to B
//# programmable --sender A --inputs 1000000000 @B
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::send_funds<rtd::rtd::RTD>(Result(0), Input(1));

//# create-checkpoint

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> B

//# view-object 0,1

// Use address balance as gas
//# transfer-object --recipient A --sender B 0,1 --gas-budget 1000000000 --address-balance-gas

//# create-checkpoint

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> B
