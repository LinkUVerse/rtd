// object has store, might be transferred elsewhere
module a::has_store {
    use rtd::object::UID;
    use rtd::transfer;
    use rtd::tx_context::TxContext;

    struct Obj has key, store {
        id: UID,
    }

    public fun make_obj(ctx: &mut TxContext): Obj {
        Obj { id: rtd::object::new(ctx) }
    }

    public fun share(o: Obj) {
        let arg = o;
        transfer::public_share_object(arg);
    }
}

// object does not have store and is transferred
module a::is_transferred {
    use rtd::object::UID;
    use rtd::transfer;
    use rtd::tx_context::{Self, TxContext};

    struct Obj has key {
        id: UID,
    }

    public fun make_obj(ctx: &mut TxContext): Obj {
        Obj { id: rtd::object::new(ctx) }
    }

    public fun transfer(o: Obj, ctx: &mut TxContext) {
        let arg = o;
        transfer::transfer(arg, tx_context::sender(ctx));
    }

    public fun share(o: Obj) {
        let arg = o;
        transfer::share_object(arg);
    }
}

module rtd::tx_context {
    struct TxContext has drop {}

    public fun sender(_: &TxContext): address {
        @0
    }
}

module rtd::object {
    const ZERO: u64 = 0;

    struct UID has store {
        id: address,
    }

    public fun delete(_: UID) {
        abort ZERO
    }

    public fun new(_: &mut rtd::tx_context::TxContext): UID {
        abort ZERO
    }
}

module rtd::transfer {
    const ZERO: u64 = 0;

    public fun transfer<T: key>(_: T, _: address) {
        abort ZERO
    }

    public fun share_object<T: key>(_: T) {
        abort ZERO
    }

    public fun public_share_object<T: key>(_: T) {
        abort ZERO
    }
}
