// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//# init --addresses ex=0x0

//# publish
module ex::m;

public struct Priv has key {
    id: UID,
}

public fun mint(ctx: &mut TxContext) {
    let q = Priv { id: object::new(ctx) };
    transfer::transfer(q, ctx.sender());
}

public fun priv(ctx: &mut TxContext): Priv {
    Priv { id: object::new(ctx) }
}

//# run ex::m::mint

// Does not have store
//# programmable --inputs object(2,0) @0
//> 0: rtd::party::single_owner(Input(1));
//> rtd::transfer::public_party_transfer<ex::m::Priv>(Input(0), Result(0))

// Does not have store
//# programmable --inputs @0
//> 0: ex::m::priv();
//> 1: rtd::party::single_owner(Input(0));
//> rtd::transfer::public_party_transfer<ex::m::Priv>(Result(0), Result(1))

// Private transfer
//# programmable --inputs object(2,0) @0
//> 0: rtd::party::single_owner(Input(1));
//> rtd::transfer::party_transfer<ex::m::Priv>(Input(0), Result(0))

// Private transfer
//# programmable --inputs @0
//> 0: ex::m::priv();
//> 1: rtd::party::single_owner(Input(0));
//> rtd::transfer::party_transfer<ex::m::Priv>(Result(0), Result(1))
