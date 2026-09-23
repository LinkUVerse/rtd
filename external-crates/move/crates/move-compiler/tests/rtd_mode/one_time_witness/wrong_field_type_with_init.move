// invalid, wrong struct field type

module a::m {
    use rtd::tx_context;

    struct M has drop { value: u64 }

    fun init(_otw: M, _ctx: &mut tx_context::TxContext) {}
}

module rtd::tx_context {
    struct TxContext has drop {}
}
