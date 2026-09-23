module a::edge_cases {
    struct UID {}
    // Test case with a different UID type
    struct DifferentUID {
        id: rtd::another::UID,
    }

    struct NotAnObject {
        id: UID,
    }
}

module rtd::object {
    struct UID has store {
        id: address,
    }
}

module rtd::another {
    struct UID has store {
        id: address,
    }
}
