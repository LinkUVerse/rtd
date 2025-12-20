module a::beep {
    struct BEEP has drop {
        f0: u64,
        f1: bool,
    }

    fun init(_: BEEP, _ctx: &mut rtd::tx_context::TxContext) {}
}

module rtd::tx_context {
    struct TxContext has drop {}
}
