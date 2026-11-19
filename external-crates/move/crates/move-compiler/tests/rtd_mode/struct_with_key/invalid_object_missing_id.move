module a::test {
    use rtd::object::UID;

    struct S has key {
        id: UID,
    }

    fun make(): S {
        S {}
    }
}

module rtd::object {
    struct UID has store {
        id: address,
    }
}

module rtd::transfer {
    public fun transfer<T: key>(_: T, _: address) {
        abort 0
    }
}
