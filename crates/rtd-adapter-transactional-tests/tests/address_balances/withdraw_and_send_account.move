// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Test send_funds and redeem_funds from rtd::balance

//# init --addresses test=0x0 --accounts A B --enable-accumulators --simulator

// Send 1000 from A to B
//# programmable --sender A --inputs 1000 @B
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::into_balance<rtd::rtd::RTD>(Result(0));
//> 2: rtd::balance::send_funds<rtd::rtd::RTD>(Result(1), Input(1));

//# create-checkpoint

// B withdraws 500 and send to A
//# programmable --sender B --inputs withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(500) @A
//> 0: rtd::balance::redeem_funds<rtd::rtd::RTD>(Input(0));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(1));

//# create-checkpoint

// B withdraws 500 and send to self
//# programmable --sender B --inputs withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(500) @B
//> 0: rtd::balance::redeem_funds<rtd::rtd::RTD>(Input(0));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(1));
