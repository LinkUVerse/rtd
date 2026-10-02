// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Binding violations die at transaction input validation: a sender who is not
// the spender, and a declared funds type that doesn't match the allowance's.

//# init --accounts A B C --addresses test=0x0

//# publish --sender A
#[allow(deprecated_usage)]
module test::foo;

use rtd::coin;

public struct FOO has drop {}

fun init(otw: FOO, ctx: &mut TxContext) {
    let (treasury_cap, metadata) = coin::create_currency(
        otw, 6, b"FOO", b"Foo", b"", option::none(), ctx,
    );
    transfer::public_freeze_object(metadata);
    transfer::public_transfer(treasury_cap, ctx.sender());
}

//# programmable --sender A --inputs 5000 @A
// Fund A's (the funder) RTD address balance.
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::send_funds<rtd::rtd::RTD>(Result(0), Input(1));

//# programmable --sender A --inputs object(1,1) 500 @A
// Fund A's FOO address balance, so the type-mismatch case below gets past the
// balance-availability check and reaches the allowance checks.
//> 0: rtd::coin::mint<test::foo::FOO>(Input(0), Input(1));
//> 1: rtd::coin::send_funds<test::foo::FOO>(Result(0), Input(2));

//# create-checkpoint

//# programmable --sender A --inputs b"txn_test" @B vector[1000u256] vector[] vector[99999999999999]
// A issues a Balance<RTD> allowance to B: 1000 lifetime cap.
//> 0: std::option::none<rtd::allowance::RateLimit>();
//> 1: rtd::allowance::new<rtd::balance::Balance<rtd::rtd::RTD>>(Input(0), Input(1), Input(2), Input(3), Input(4), Result(0));

//# view-object 5,0

//# programmable --sender C --inputs allowance_withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(100,@A,object(5,0)) mutshared(5,0) immshared(6)
// C is not the spender: rejected at signing.
//> 0: rtd::allowance::balance_spend<rtd::rtd::RTD>(Input(1), Input(0), Input(2));

//# programmable --sender B --inputs allowance_withdraw<rtd::balance::Balance<test::foo::FOO>>(100,@A,object(5,0)) mutshared(5,0) immshared(6)
// B declares a Balance<FOO> withdrawal against the Balance<RTD> allowance:
// rejected at signing on the funds type, not on FOO availability.
//> 0: rtd::allowance::balance_spend<test::foo::FOO>(Input(1), Input(0), Input(2));

//# view-object 5,0
// current_spend is untouched by either attempt.
