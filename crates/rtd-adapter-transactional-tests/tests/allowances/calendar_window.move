// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// A monthly rate limit driven by the real Clock: the first spend anchors the
// window, exhausting it blocks further spends, and advancing the clock past
// the anniversary rolls the window over.

//# init --accounts A B --simulator

//# programmable --sender A --inputs 5000 @A
// Fund A's (the funder) address balance.
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::send_funds<rtd::rtd::RTD>(Result(0), Input(1));

//# create-checkpoint

//# programmable --sender A --inputs b"monthly" @B vector[] vector[] vector[] 500u256
// A issues an allowance to B: 500 per month, no lifetime cap.
//> 0: rtd::allowance::monthly_rate_limit(Input(5));
//> 1: std::option::some<rtd::allowance::RateLimit>(Result(0));
//> 2: rtd::allowance::new<rtd::balance::Balance<rtd::rtd::RTD>>(Input(0), Input(1), Input(2), Input(3), Input(4), Result(1));

//# programmable --sender B --inputs allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(500,@A,object(3,0)) mutshared(3,0) immshared(6) @B
// The first spend anchors the window and exhausts the month.
//> 0: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(1), Input(0), Input(2));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(3));

//# view-object 3,0

//# programmable --sender B --inputs allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(1,@A,object(3,0)) mutshared(3,0) immshared(6) @B
// One more unit in the same window: aborts.
//> 0: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(1), Input(0), Input(2));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(3));

//# advance-clock --duration-ns 2764800000000000
// 32 days: past the next monthly anniversary from any anchor date.

//# programmable --sender B --inputs allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(500,@A,object(3,0)) mutshared(3,0) immshared(6) @B
// The window rolled over: a full month's worth spends again.
//> 0: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(1), Input(0), Input(2));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(3));

//# view-object 3,0
// The anchor is unchanged, the window index advanced, and spent reset to the
// new window's 500; current_spend is 1000.

//# create-checkpoint

//# view-object 1,0
// A's settled balance: 5000 - 1000 = 4000.

//# view-object 4,0
// B's settled balance: 1000.
