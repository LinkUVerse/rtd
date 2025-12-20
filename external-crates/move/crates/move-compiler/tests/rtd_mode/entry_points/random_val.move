// invalid Random by value

module a::m {
    public entry fun no_random_val(_: rtd::random::Random) {
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
