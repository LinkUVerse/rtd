// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// A self-allowance: one address as funder, spender, and sender. Spends work,
// and an allowance withdrawal and a plain sender withdrawal coalesce at the
// same (address, type) reservation key.

//# init --accounts A B

//# programmable --sender A --inputs 5000 @A
// Fund A's (the funder) address balance.
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::send_funds<rtd::rtd::RTD>(Result(0), Input(1));

//# create-checkpoint

//# programmable --sender A --inputs b"self" @A vector[1000u256] vector[] vector[99999999999999]
// A issues an allowance to themselves: 1000 lifetime cap.
//> 0: std::option::none<rtd::allowance::RateLimit>();
//> 1: rtd::allowance::new<rtd::balance::Balance<rtd::rtd::RTD>>(Input(0), Input(1), Input(2), Input(3), Input(4), Result(0));

//# programmable --sender A --inputs allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(400,@A,object(3,0)) mutshared(3,0) immshared(6) @B
// A spends 400 through their own allowance.
//> 0: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(1), Input(0), Input(2));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(3));

//# programmable --sender A --inputs allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(200,@A,object(3,0)) withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(100) mutshared(3,0) immshared(6) @B
// An allowance withdrawal and a plain withdrawal, both against A's balance:
// the reservations coalesce at one key and settle as one Split.
//> 0: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(2), Input(0), Input(3));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(4));
//> 2: rtd::balance::redeem_funds<rtd::rtd::RTD>(Input(1));
//> 3: rtd::balance::send_funds<rtd::rtd::RTD>(Result(2), Input(4));

//# view-object 3,0
// current_spend is 600: the allowance-sourced 400 + 200 only.

//# create-checkpoint

//# view-object 1,0
// A's settled balance: 5000 - 400 - 300 = 4300.

//# view-object 4,0
// B's settled balance: 400 + 300 = 700.

//# programmable --sender A --inputs allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(100,@A,object(3,0)) mutshared(3,0) object(3,1) immshared(6) @B
// Spend and revoke in one PTB: the withdrawal consumes first, then the
// revoke deletes the allowance and its cap.
//> 0: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(1), Input(0), Input(3));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(4));
//> 2: rtd::allowance::revoke<rtd::balance::Balance<rtd::rtd::RTD>>(Input(2), Input(1));

//# programmable --sender A --inputs b"self2" @A vector[1000u256] vector[] vector[99999999999999]
// A second self-allowance, for the reversed order below.
//> 0: std::option::none<rtd::allowance::RateLimit>();
//> 1: rtd::allowance::new<rtd::balance::Balance<rtd::rtd::RTD>>(Input(0), Input(1), Input(2), Input(3), Input(4), Result(0));

//# programmable --sender A --inputs allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(100,@A,object(11,0)) mutshared(11,0) object(11,1) immshared(6) @B
// Revoke first: the spend cannot use the consumed allowance, so the whole
// transaction rolls back, revocation included.
//> 0: rtd::allowance::revoke<rtd::balance::Balance<rtd::rtd::RTD>>(Input(2), Input(1));
//> 1: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(1), Input(0), Input(3));
//> 2: rtd::balance::send_funds<rtd::rtd::RTD>(Result(1), Input(4));

//# view-object 11,0
// The second allowance survives the failed revoke; current_spend is 0.

//# create-checkpoint

//# view-object 1,0
// A's settled balance: 4300 - 100 = 4200.

//# view-object 4,0
// B's settled balance: 700 + 100 = 800.
