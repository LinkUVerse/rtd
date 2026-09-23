// valid, Receiving type with object type param

module a::m {
    use rtd::object;
    use rtd::transfer::Receiving;

    struct S has key { id: object::UID }

    public entry fun yes(_: Receiving<S>) {}
}

module rtd::object {
    struct UID has store {
        id: address,
    }
}

module rtd::transfer {
    struct Receiving<phantom T: key> has drop {
        id: address,
    }
}
