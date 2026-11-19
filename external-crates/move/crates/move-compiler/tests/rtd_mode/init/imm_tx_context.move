// TxContext can be immutable, even for init
module a::m {
    use rtd::tx_context;

    fun init(_ctx: &tx_context::TxContext) {
        abort 0
    }
}

module rtd::tx_context {
    struct TxContext has drop {}
}
