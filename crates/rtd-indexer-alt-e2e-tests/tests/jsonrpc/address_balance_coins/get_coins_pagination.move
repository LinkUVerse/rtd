// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Test pagination of rtdx_getCoins when address balance coins are mixed with real coins.

//# init --protocol-version 119 --addresses Test=0x0 --accounts A B --simulator

//# programmable --sender A --inputs 500 @B
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: TransferObjects([Result(0)], Input(1))

//# programmable --sender A --inputs 100 @B
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: TransferObjects([Result(0)], Input(1))

//# programmable --sender A --inputs 300 @B
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::into_balance<rtd::rtd::RTD>(Result(0));
//> 2: rtd::balance::send_funds<rtd::rtd::RTD>(Result(1), Input(1));

//# create-checkpoint

//# run-jsonrpc
{
  "method": "rtdx_getCoins",
  "params": ["@{B}", null, null, 2]
}

//# run-jsonrpc
{
  "method": "rtdx_getCoins",
  "params": ["@{B}"]
}
