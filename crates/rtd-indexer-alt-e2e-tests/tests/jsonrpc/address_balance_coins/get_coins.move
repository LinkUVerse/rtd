// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Fund an address balance for B and verify that rtdx_getCoins returns the address balance coin.

//# init --protocol-version 119 --addresses Test=0x0 --accounts A B --simulator

// Send 1_000_000_000 from A to B's address balance
//# programmable --sender A --inputs 1000000000 @B
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::into_balance<rtd::rtd::RTD>(Result(0));
//> 2: rtd::balance::send_funds<rtd::rtd::RTD>(Result(1), Input(1));

//# create-checkpoint

// B should see the address balance coin in getCoins
//# run-jsonrpc
{
  "method": "rtdx_getCoins",
  "params": ["@{B}"]
}
