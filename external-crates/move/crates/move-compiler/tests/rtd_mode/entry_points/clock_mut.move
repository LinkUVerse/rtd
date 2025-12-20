// invalid, Clock by mutable reference

module a::m {
    public entry fun no_clock_mut(_: &mut rtd::clock::Clock) {
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
