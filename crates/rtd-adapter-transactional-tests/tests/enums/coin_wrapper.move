// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//# init --addresses Test=0x0 --accounts A

//# publish --upgradeable --sender A
module Test::f {
    use rtd::coin::Coin;
    use rtd::rtd::RTD;

    public struct Other { }

    public enum CoinWrapper has store {
        Rtd(Coin<RTD>),
        Other(Coin<Other>),
    }

    public struct CoinObject has key, store {
        id: UID,
        coin: CoinWrapper,
    }

    public fun split_off(coin: &mut CoinObject, amount: u64, ctx: &mut TxContext): CoinObject {
        match (&mut coin.coin) {
            CoinWrapper::Rtd(c) => {
                let new_coin = CoinObject {
                    id: object::new(ctx),
                    coin: CoinWrapper::Rtd(c.split(amount, ctx)),
                };
                new_coin
            },
            CoinWrapper::Other(c) => {
                let new_coin = CoinObject {
                    id: object::new(ctx),
                    coin: CoinWrapper::Other(c.split(amount, ctx)),
                };
                new_coin
            },
        }
    }

    public fun create_rtd(coin: &mut Coin<RTD>, amount: u64, ctx: &mut TxContext): CoinObject {
        CoinObject {
            id: object::new(ctx),
            coin: CoinWrapper::Rtd(coin.split(amount, ctx)),
        }
    }
}

//# programmable --sender A --inputs 10 @A
//> 0: Test::f::create_rtd(Gas, Input(0));
//> 1: TransferObjects([Result(0)], Input(1))
