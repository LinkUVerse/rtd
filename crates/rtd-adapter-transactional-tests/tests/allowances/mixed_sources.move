// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// An allowance spend and the sender's own withdrawal in one PTB: two
// reservation keys (funder and sender), each settled against its own
// address balance.

//# init --accounts A B

//# programmable --sender A --inputs 5000 @A
// Fund A's (the funder) address balance.
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::send_funds<rtd::rtd::RTD>(Result(0), Input(1));

//# programmable --sender B --inputs 1000 @B
// Fund B's (the spender) address balance.
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::send_funds<rtd::rtd::RTD>(Result(0), Input(1));

//# create-checkpoint

//# programmable --sender A --inputs b"mixed" @B vector[1000u256] vector[] vector[99999999999999]
// A issues an allowance to B: 1000 lifetime cap.
//> 0: std::option::none<rtd::allowance::RateLimit>();
//> 1: rtd::allowance::new<rtd::balance::Balance<rtd::rtd::RTD>>(Input(0), Input(1), Input(2), Input(3), Input(4), Result(0));

//# programmable --sender B --inputs allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(400,@A,object(4,0)) withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(300) mutshared(4,0) immshared(6) @A @B
// B spends 400 of A's allowance to themselves and 300 of their own balance
// to A. Settlement nets per address: A -100, B +100.
//> 0: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(2), Input(0), Input(3));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(5));
//> 2: rtd::balance::redeem_funds<rtd::rtd::RTD>(Input(1));
//> 3: rtd::balance::send_funds<rtd::rtd::RTD>(Result(2), Input(4));

//# programmable --sender B --inputs allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(200,@A,object(4,0)) mutshared(4,0) immshared(6) @A
// B spends 200 and sends all of it back to A: the flow nets to zero at both
// addresses, yet the allowance is still charged the gross amount.
//> 0: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(1), Input(0), Input(2));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(3));

//# view-object 4,0
// current_spend is 600: the gross 400 + 200; the sender-sourced withdrawal
// and the net-zero settlement never touched it.

//# create-checkpoint

//# view-object 1,0
// A's settled balance: 5000 - 400 + 300 = 4900; the round trip nets out.

//# view-object 2,0
// B's settled balance: 1000 - 300 + 400 = 1100.
