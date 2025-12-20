
// tests that the example which is allowed in rtd mode is not allowed outside of that mode

module a::m {
    struct Obj has key { id: rtd::object::UID }
}

module rtd::object {
    struct UID has store { value: address }
    public fun borrow_address(id: &UID): &address { &id.value }
}
