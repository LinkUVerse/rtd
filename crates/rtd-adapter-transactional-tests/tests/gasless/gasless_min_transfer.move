// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Tests minimum transfer amount enforcement for gasless transactions.

//# init --addresses test=0x0 --accounts A B C --enable-gasless

//# publish --sender A
#[allow(deprecated_usage)]
module test::usdc {
    use rtd::coin;

    public struct USDC has drop {}

    fun init(otw: USDC, ctx: &mut TxContext) {
        let (treasury_cap, metadata) = coin::create_currency(
            otw, 6, b"USDC", b"USD Coin", b"", option::none(), ctx,
        );
        transfer::public_freeze_object(metadata);
        transfer::public_transfer(treasury_cap, ctx.sender());
    }
}

// Register USDC with a minimum transfer of 1000
//# gasless-allow-token test::usdc::USDC --min-transfer 1000

//# programmable --sender A --inputs 50000 object(1,1) @A
// Mint 50000 USDC and send to A's address balance
//> 0: rtd::coin::mint<test::usdc::USDC>(Input(1), Input(0));
//> 1: rtd::coin::into_balance<test::usdc::USDC>(Result(0));
//> 2: rtd::balance::send_funds<test::usdc::USDC>(Result(1), Input(2));

//# create-checkpoint

//# view-funds rtd::balance::Balance<test::usdc::USDC> A

//# programmable --sender A --address-balance-gas --gas-price 0 --gas-budget 0 --inputs withdraw<rtd::balance::Balance<test::usdc::USDC>>(500) @B
// Test 1: Transfer BELOW minimum (500 < 1000) should fail
//> 0: rtd::balance::redeem_funds<test::usdc::USDC>(Input(0));
//> 1: rtd::balance::send_funds<test::usdc::USDC>(Result(0), Input(1));

//# programmable --sender A --address-balance-gas --gas-price 0 --gas-budget 0 --inputs withdraw<rtd::balance::Balance<test::usdc::USDC>>(1000) @B
// Test 2: Transfer AT minimum (1000) should succeed
//> 0: rtd::balance::redeem_funds<test::usdc::USDC>(Input(0));
//> 1: rtd::balance::send_funds<test::usdc::USDC>(Result(0), Input(1));

//# create-checkpoint

//# view-funds rtd::balance::Balance<test::usdc::USDC> A

//# view-funds rtd::balance::Balance<test::usdc::USDC> B

//# programmable --sender A --address-balance-gas --gas-price 0 --gas-budget 0 --inputs withdraw<rtd::balance::Balance<test::usdc::USDC>>(500) withdraw<rtd::balance::Balance<test::usdc::USDC>>(500) @B
// Test 3: Two 500 transfers to SAME recipient, aggregated 1000 meets minimum
//> 0: rtd::balance::redeem_funds<test::usdc::USDC>(Input(0));
//> 1: rtd::balance::send_funds<test::usdc::USDC>(Result(0), Input(2));
//> 2: rtd::balance::redeem_funds<test::usdc::USDC>(Input(1));
//> 3: rtd::balance::send_funds<test::usdc::USDC>(Result(2), Input(2));

//# create-checkpoint

//# view-funds rtd::balance::Balance<test::usdc::USDC> B

//# programmable --sender A --address-balance-gas --gas-price 0 --gas-budget 0 --inputs withdraw<rtd::balance::Balance<test::usdc::USDC>>(1500) withdraw<rtd::balance::Balance<test::usdc::USDC>>(500) @B @C
// Test 4: Two recipients, one above minimum (1500), one below (500) should fail
//> 0: rtd::balance::redeem_funds<test::usdc::USDC>(Input(0));
//> 1: rtd::balance::send_funds<test::usdc::USDC>(Result(0), Input(2));
//> 2: rtd::balance::redeem_funds<test::usdc::USDC>(Input(1));
//> 3: rtd::balance::send_funds<test::usdc::USDC>(Result(2), Input(3));
