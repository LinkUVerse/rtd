// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Multiple spends from one allowance in one PTB. Pins: all-or-nothing
// settlement (an abort on a later spend rolls back the earlier ones), rate
// enforcement happening at execution only, and the window rolling through
// the real clock.

//# init --accounts A B --simulator

//# programmable --sender A --inputs 5000 @A
// Fund A's (the funder) address balance.
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::send_funds<rtd::rtd::RTD>(Result(0), Input(1));

//# create-checkpoint

//# programmable --sender A --inputs b"multi" @B vector[10000u256] vector[] vector[] 100000 100u256
// A issues an allowance to B: 10000 lifetime cap, 100 per 100s window.
//> 0: rtd::allowance::periodic_rate_limit(Input(5), Input(6));
//> 1: std::option::some<rtd::allowance::RateLimit>(Result(0));
//> 2: rtd::allowance::new<rtd::balance::Balance<rtd::rtd::RTD>>(Input(0), Input(1), Input(2), Input(3), Input(4), Result(1));

//# programmable --sender B --inputs allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(30,@A,object(3,0)) allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(30,@A,object(3,0)) allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(30,@A,object(3,0)) mutshared(3,0) immshared(6) @B
// Three spends of 30 in one PTB, all within the window: all settle.
//> 0: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(3), Input(0), Input(4));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(5));
//> 2: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(3), Input(1), Input(4));
//> 3: rtd::balance::send_funds<rtd::rtd::RTD>(Result(2), Input(5));
//> 4: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(3), Input(2), Input(4));
//> 5: rtd::balance::send_funds<rtd::rtd::RTD>(Result(4), Input(5));

//# view-object 3,0
// current_spend is 90: three spends of 30.

//# programmable --sender B --inputs allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(5,@A,object(3,0)) allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(10,@A,object(3,0)) mutshared(3,0) immshared(6) @B
// 90 of the 100 window is used. Execution settles the first spend (95), then
// aborts on the second (105 > 100), rolling back the whole transaction.
//> 0: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(2), Input(0), Input(3));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(4));
//> 2: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(2), Input(1), Input(3));
//> 3: rtd::balance::send_funds<rtd::rtd::RTD>(Result(2), Input(4));

//# view-object 3,0
// current_spend is still 90: the abort rolled back the settled first spend too.

//# advance-clock --duration-ns 100000000000

//# programmable --sender B --inputs allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(5,@A,object(3,0)) allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(10,@A,object(3,0)) mutshared(3,0) immshared(6) @B
// The identical transaction after the window rolls over: both settle.
//> 0: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(2), Input(0), Input(3));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(4));
//> 2: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(2), Input(1), Input(3));
//> 3: rtd::balance::send_funds<rtd::rtd::RTD>(Result(2), Input(4));

//# view-object 3,0
// current_spend is 105: 90 + 15.

//# programmable --sender B --inputs allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(60,@A,object(3,0)) allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(60,@A,object(3,0)) mutshared(3,0) immshared(6) @B
// Two 60s on top of the 15 already spent this window: signing admits them
// (no limit math there); execution settles the first (75), aborts the second.
//> 0: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(2), Input(0), Input(3));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(4));
//> 2: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(2), Input(1), Input(3));
//> 3: rtd::balance::send_funds<rtd::rtd::RTD>(Result(2), Input(4));

//# view-object 3,0
// current_spend is still 105 and the window still has 15: full rollback again.
