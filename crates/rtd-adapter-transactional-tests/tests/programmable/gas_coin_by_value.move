// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// tests valid gas coin usage by value

//# init --addresses test=0x0 --accounts A B

//# programmable --sender A --inputs @B
//> TransferObjects([Gas], Input(0))

//# view-object 0,0

//# programmable --sender B --inputs @A --gas-payment object(0,0)
//> rtd::coin::send_funds<rtd::rtd::RTD>(Gas, Input(0))

//# view-object 0,0

//# create-checkpoint

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> A

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> B
