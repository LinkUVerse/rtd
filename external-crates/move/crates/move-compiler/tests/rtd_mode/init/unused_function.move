// init is unused but does not error because we are in Rtd mode
module a::m {
    fun init(_: &mut rtd::tx_context::TxContext) {}
}

module rtd::tx_context {
    struct TxContext has drop {}
}
