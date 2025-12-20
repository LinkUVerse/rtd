// valid, Clock by immutable reference

module a::m {
    public entry fun yes_clock_ref(_: &rtd::clock::Clock) {
        abort 0
    }
}

module rtd::clock {
    struct Clock has key {
        id: rtd::object::UID,
    }
}

module rtd::object {
    struct UID has store {
        id: address,
    }
}
