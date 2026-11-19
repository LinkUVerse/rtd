module rtd::object {
    public struct ID()
    public struct UID()
}
module rtd::transfer {

}
module rtd::tx_context {
    public struct TxContext()
}

module a::m {
    use rtd::object::{Self, ID, UID};
    use rtd::transfer;
    use rtd::tx_context::{Self, TxContext};
}
