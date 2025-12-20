// not allowed, it re-uses an ID in a new object
module a::m {
    use rtd::object::UID;
    use rtd::transfer::transfer;
    use rtd::tx_context::{Self, TxContext};

    struct Cat has key {
        id: UID,
    }

    struct Dog has key {
        id: UID,
    }

    public fun transmute(cat: Cat, ctx: &mut TxContext) {
        let Cat { id } = cat;
        let dog = Dog { id };
        transfer(dog, tx_context::sender(ctx));
    }
}

module rtd::object {
    struct UID has store {
        id: address,
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
