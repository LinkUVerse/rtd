module a::m {
    struct Obj has key { id: rtd::object::UID }
}

module rtd::object {
    struct UID has store { value: address }

    public fun borrow_address(id: &UID): &address { &id.value }
}
