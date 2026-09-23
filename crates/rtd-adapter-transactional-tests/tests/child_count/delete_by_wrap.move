// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// DEPRECATED child count no longer tracked
// tests invalid wrapping of a parent object with children

//# init --addresses test=0x0 --accounts A B

//# publish

module test::m {
    use rtd::dynamic_object_field as ofield;

    public struct S has key, store {
        id: rtd::object::UID,
    }

    public struct R has key, store {
        id: rtd::object::UID,
        s: S,
    }

    public entry fun mint(ctx: &mut TxContext) {
        let id = rtd::object::new(ctx);
        rtd::transfer::public_transfer(S { id }, tx_context::sender(ctx))
    }

    public entry fun add(parent: &mut S, idx: u64, ctx: &mut TxContext) {
        let child = S { id: rtd::object::new(ctx) };
        ofield::add(&mut parent.id, idx, child);
    }

    public entry fun wrap(s: S, ctx: &mut TxContext) {
        let r = R { id: rtd::object::new(ctx), s };
        rtd::transfer::public_transfer(r, tx_context::sender(ctx))
    }
}

//# run test::m::mint --sender A

//# run test::m::add --sender A --args object(2,0) 0

//# view-object 2,0

//# run test::m::wrap --sender A --args object(2,0)
