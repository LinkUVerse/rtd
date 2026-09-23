// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module tto::M1 {
    use rtd::coin::Coin;
    use rtd::rtd::RTD;
    use rtd::object::{Self, UID};
    use rtd::tx_context::{Self, TxContext};
    use rtd::transfer::{Self, Receiving};

    public struct A has key, store {
        id: UID,
    }

    public fun start(coin: Coin<RTD>, ctx: &mut TxContext) {
        let a = A { id: object::new(ctx) };
        let a_address = object::id_address(&a);

        transfer::public_transfer(a, tx_context::sender(ctx));
        transfer::public_transfer(coin, a_address);
    }

    public entry fun receive(parent: &mut A, x: Receiving<Coin<RTD>>) {
        let coin = transfer::public_receive(&mut parent.id, x);
        transfer::public_transfer(coin, @tto);
    }
}
