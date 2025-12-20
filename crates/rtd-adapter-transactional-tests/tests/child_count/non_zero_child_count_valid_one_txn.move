// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// DEPRECATED child count no longer tracked
// tests valid transfers of an object that has children
// all transfers done in a single transaction

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

    public entry fun share(ctx: &mut TxContext) {
        let mut id = rtd::object::new(ctx);
        let child = S { id: rtd::object::new(ctx) };
        ofield::add(&mut id, 0u64, child);
        rtd::transfer::public_share_object(S { id })
    }

}

//
// Test share object allows non-zero child count
//

//# run test::m::share --sender A

//# view-object 2,1
