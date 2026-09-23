// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module example::example;

public struct Obj has key, store {
    id: UID,
}

// Transferring a freshly created object to the transaction sender triggers the
// Rtd-specific `self_transfer` lint. This lint only runs when the compiler is in
// Rtd mode, which `rtd move lint` must enable; a plain `rtd move build` does not
// report it.
public fun mint(ctx: &mut TxContext) {
    transfer::public_transfer(Obj { id: object::new(ctx) }, ctx.sender())
}
