// valid Random by immutable reference

module a::m {
    public entry fun yes_random_ref(_: &rtd::random::Random) {
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
