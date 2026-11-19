// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// The funder is also the gas sponsor, paying gas from their own address
// balance. Two reservation sources -- the gas budget and the allowance
// withdrawal -- coalesce on one key that is neither the sender's nor a
// self-allowance, and the net withdrawal is capped by their sum.

//# init --accounts B S

//# programmable --sender S --inputs 10000000000 @S
// Fund S's (the funder and gas sponsor) address balance.
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::send_funds<rtd::rtd::RTD>(Result(0), Input(1));

//# create-checkpoint

//# programmable --sender S --inputs b"sponsor-funder" @B vector[100000u256] vector[] vector[99999999999999]
// S issues an allowance to B: 100000 lifetime cap.
//> 0: std::option::none<rtd::allowance::RateLimit>();
//> 1: rtd::allowance::new<rtd::balance::Balance<rtd::rtd::RTD>>(Input(0), Input(1), Input(2), Input(3), Input(4), Result(0));

//# programmable --sender B --sponsor S --address-balance-gas --inputs allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(30000,@S,object(3,0)) mutshared(3,0) immshared(6) @B
// B spends S's allowance in a tx S sponsors from S's address balance: the gas
// key and the allowance key are the same (S, Balance<RTD>), and S is neither
// the sender nor the funder of a self-allowance.
//> 0: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(1), Input(0), Input(2));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(3));

//# view-object 3,0
// current_spend is 30000: only the allowance-sourced spend, not the gas.

//# create-checkpoint

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> S
// 10000000000 - 30000, minus the sponsored task's gas.

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> B
// 30000.
