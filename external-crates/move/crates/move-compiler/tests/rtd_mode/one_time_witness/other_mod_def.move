// invalid, one-time witness type candidate used in a different module

module a::n {
    use rtd::rtd;
    use rtd::tx_context;

    fun init(_otw: rtd::RTD, _ctx: &mut tx_context::TxContext) {}
}

module rtd::tx_context {
    struct TxContext has drop {}
}

module rtd::rtd {
    struct RTD has drop {}
}
