module a::m {
    use rtd::tx_context;

    public entry fun foo<T>(_: T, _: &mut tx_context::TxContext) {
        abort 0
    }
}

module rtd::tx_context {
    struct TxContext has drop {}
}
