// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// A sponsored transaction cannot pay gas via an address-balance withdrawal.

//# init --addresses test=0x0 --accounts A B

// Seed B's address balance so a hypothetical sponsor-side withdrawal would
// have funds available.
//# programmable --sender B --inputs 100000000000 @B
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::into_balance<rtd::rtd::RTD>(Result(0));
//> 2: rtd::balance::send_funds<rtd::rtd::RTD>(Result(1), Input(1));

//# create-checkpoint

// Sponsored tx with withdraw gas payment -- expected to be rejected by
// validation. The withdraw amount targets B (the sponsor) at the test-runner
// level, but validation resolves it against A (sender), producing a mismatch.
//# programmable --sender A --sponsor B --gas-payment withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(500000000) --inputs 100000 @A
//> 0: SplitCoins(Gas, [Input(0)]);
//> TransferObjects([Result(0)], Input(1))
