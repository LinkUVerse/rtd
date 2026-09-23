// valid
module a::m {
    use rtd::object;

    struct S has key {
        id: object::UID,
    }
}

module rtd::object {
    struct UID has store {
        id: address,
    }
}
