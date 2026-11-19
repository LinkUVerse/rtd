// invalid, objects need UID not ID
module a::m {
    use rtd::object;

    struct S has key {
        id: object::ID,
    }
}

module rtd::object {
    struct ID has store {
        id: address,
    }
}
