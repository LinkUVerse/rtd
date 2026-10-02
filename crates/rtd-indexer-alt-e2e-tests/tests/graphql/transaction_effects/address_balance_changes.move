// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Test send_funds and redeem_funds from rtd::balance

//# init --protocol-version 128 --addresses test=0x0 --accounts A B C --simulator

// Send 1000000000 from A to B and A to C
//# programmable --sender A --inputs 1000000000 @B @C
//> 0: SplitCoins(Gas, [Input(0)]);
//> 1: rtd::coin::into_balance<rtd::rtd::RTD>(Result(0));
//> 2: rtd::balance::send_funds<rtd::rtd::RTD>(Result(1), Input(1));
//> 3: SplitCoins(Gas, [Input(0)]);
//> 4: rtd::coin::into_balance<rtd::rtd::RTD>(Result(3));
//> 5: rtd::balance::send_funds<rtd::rtd::RTD>(Result(4), Input(2));

//# create-checkpoint

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> B

//# view-object 0,1

// Use address balance as gas
//# transfer-object --recipient A --sender B 0,1 --gas-budget 1000000000 --address-balance-gas

//# create-checkpoint

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> B

// Now have B send address balance to C using address balance as gas
//# programmable --sender B --inputs withdraw<rtd::balance::Balance<rtd::rtd::RTD>>(5000000) @C --gas-budget 1000000000 --address-balance-gas
//> 0: rtd::balance::redeem_funds<rtd::rtd::RTD>(Input(0));
//> 1: rtd::balance::send_funds<rtd::rtd::RTD>(Result(0), Input(1));

//# create-checkpoint

//# view-funds rtd::balance::Balance<rtd::rtd::RTD> B

//# run-graphql
{ # Test balance_changes field on address balance transfer
  addressBalanceTransferTransaction: transactionEffects(digest: "@{digest_1}") {
    balanceChanges {
      pageInfo {
        hasNextPage
        hasPreviousPage
      }
      nodes {
        owner {
          address
        }
        coinType { repr }
        amount
      }
    }
  }
}

//# run-graphql
{ # Test balance_changes field on transaction paid by address balance
  addressBalanceGasTransaction: transactionEffects(digest: "@{digest_5}") {
    balanceChanges {
      pageInfo {
        hasNextPage
        hasPreviousPage
      }
      nodes {
        owner {
          address
        }
        coinType { repr }
        amount
      }
    }
  }
}

//# run-graphql
{ # Test balance_changes field on ab transfer transaction paid by address balance
  addressBalanceGasTransaction: transactionEffects(digest: "@{digest_8}") {
    balanceChanges {
      pageInfo {
        hasNextPage
        hasPreviousPage
      }
      nodes {
        owner {
          address
        }
        coinType { repr }
        amount
      }
    }
  }
}
