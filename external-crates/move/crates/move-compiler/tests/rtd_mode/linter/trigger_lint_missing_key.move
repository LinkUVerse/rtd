module a::trigger_lint_cases {
    use rtd::object::UID;

    // This should trigger the linter warning (true positive)
    struct MissingKeyAbility {
        id: UID,
    }
}

module rtd::object {
    struct UID has store {
        id: address,
    }
}
