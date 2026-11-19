// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Benchmark the send_funds command with a RTD withdrawal from A's address balance
// sent to B, with gas also paid from A's address balance.

//# init --addresses test=0x0 --accounts A B C D E

// Seed A's address balance so it can both fund the withdrawal and pay for gas.
//# programmable --sender A --inputs 20000000000 @A
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::into_balance<rtd::rtd::RTD>(Result(0));
//> 2: rtd::balance::send_funds<rtd::rtd::RTD>(Result(1), Input(1));

//# create-checkpoint

// Benchmark: withdraw 100 from A's address balance and send_funds to B,
// with gas paid from the address balance.
//# bench ptb --sender A --address-balance-gas --inputs withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(100) @B
//> 0: rtd::balance::redeem_funds<rtd::rtd::RTD>(Input(0));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(1));

// Benchmark: withdraw 400 from A's address balance and send it to B, C, D, and E
// with gas paid from the address balance.
//# bench ptb --sender A --address-balance-gas --inputs withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(100) withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(100) withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(100) withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(100) @B @C @D @E
//> 0: rtd::balance::redeem_funds<rtd::rtd::RTD>(Input(0));
//> 1: rtd::balance::redeem_funds<rtd::rtd::RTD>(Input(1));
//> 2: rtd::balance::redeem_funds<rtd::rtd::RTD>(Input(2));
//> 3: rtd::balance::redeem_funds<rtd::rtd::RTD>(Input(3));
//> 4: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(4));
//> 5: rtd::balance::send_funds<rtd::rtd::RTD>(Result(1), Input(5));
//> 6: rtd::balance::send_funds<rtd::rtd::RTD>(Result(2), Input(6));
//> 7: rtd::balance::send_funds<rtd::rtd::RTD>(Result(3), Input(7));

//# create-checkpoint

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> A

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> B
