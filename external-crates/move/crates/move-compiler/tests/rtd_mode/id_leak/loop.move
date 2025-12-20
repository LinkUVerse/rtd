// allowed, even though a bit pointless
module a::m {
    use rtd::object::{Self, UID};
    use rtd::transfer::transfer;
    use rtd::tx_context::{Self, TxContext};

    struct Obj has key {
        id: UID,
    }

    public entry fun transmute(ctx: &mut TxContext) {
        let i = 0;
        let id = object::new(ctx);
        while (i <= 10) {
            object::delete(id);
            id = object::new(ctx);
            i = i + 1u64;
        };
        let obj = Obj { id };
        transfer(obj, tx_context::sender(ctx))
    }
}

module rtd::object {
    struct UID has store {
        id: address,
    }

    public fun new(_: &mut rtd::tx_context::TxContext): UID {
        abort 0
    }

    public fun delete(_: UID) {
        abort 0
    }
}

module rtd::tx_context {
    struct TxContext has drop {}

    public fun sender(_: &TxContext): address {
        @0
    }
}

module rtd::transfer {
    public fun transfer<T: key>(_: T, _: address) {
        abort 0
    }
}
