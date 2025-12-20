module a::trigger_lint_cases {
    use rtd::object::UID;

    // 4. Suppress warning
    #[allow(lint(missing_key))]
    struct SuppressWarning {
        id: UID,
    }
}

module rtd::object {
    struct UID has store {
        id: address,
    }
}
