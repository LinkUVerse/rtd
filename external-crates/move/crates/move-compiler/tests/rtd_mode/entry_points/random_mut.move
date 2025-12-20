// invalid Random by mutable reference

module a::m {
    public entry fun no_random_mut(_: &mut rtd::random::Random) {
        abort 0
    }
}

module rtd::random {
    struct Random has key {
        id: rtd::object::UID,
    }
}

module rtd::object {
    struct UID has store {
        id: address,
    }
}
