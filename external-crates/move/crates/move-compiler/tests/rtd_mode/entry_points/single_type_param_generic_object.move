module a::m {
    use rtd::object;
    use rtd::tx_context;

    struct Obj<T> has key {
        id: object::UID,
        value: T,
    }

    public entry fun foo<T: store>(_: Obj<T>, _: &mut tx_context::TxContext) {
        abort 0
    }
}

module rtd::object {
    struct UID has store {
        id: address,
    }
}

module rtd::tx_context {
    struct TxContext has drop {}
}
