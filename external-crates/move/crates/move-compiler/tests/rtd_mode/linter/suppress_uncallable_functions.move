// invalid cases. Clock and Random must be passed by immutable reference.
// TxContext can appear multiple times but uniquely if mutable. And cannot be owned.

module a::m {
    const ERR: u64 = 0;

    #[allow(lint(uncallable_function))]
    fun no_clock_mut(_: &mut rtd::clock::Clock) {
        abort ERR
    }
    #[allow(lint(uncallable_function))]
    fun no_clock_val(_: rtd::clock::Clock) {
        abort ERR
    }
    #[allow(lint(uncallable_function))]
     fun no_random_mut(_: &mut rtd::random::Random) {
        abort ERR
    }
    #[test_only]
    fun no_random_val(_: rtd::random::Random) {
        abort ERR
    }
    #[allow(lint(uncallable_function))]
    public fun ret_ctx_mut(
        ctx: &mut rtd::tx_context::TxContext,
    ): &mut rtd::tx_context::TxContext {
        ctx
    }
    #[test_only]
    public fun test_ret_ctx(ctx: &mut rtd::tx_context::TxContext): &mut rtd::tx_context::TxContext {
        ctx
    }
}

#[test_only]
module a::t {
    const ERR: u64 = 0;

    fun no_clock_mut(_: &mut rtd::clock::Clock) {
        abort ERR
    }
    fun no_clock_val(_: rtd::clock::Clock) {
        abort ERR
    }
     fun no_random_mut(_: &mut rtd::random::Random) {
        abort ERR
    }
    fun no_random_mut(_: &mut rtd::random::Random) {
        abort ERR
    }
    fun no_random_val(_: rtd::random::Random) {
        abort ERR
    }
}

#[allow(lint(uncallable_function))]
module a::n {
    const ERR: u64 = 0;

    use rtd::tx_context::TxContext;
    fun two_mut_ctx(_: &mut TxContext, _: &mut TxContext) {
        abort ERR
    }
    fun mut_and_imm_ctx(_: &mut TxContext, _: &TxContext) {
        abort ERR
    }
    fun owned_ctx(_: TxContext, _: &mut TxContext, _: &mut TxContext) {
        abort ERR
    }

}

module rtd::clock {
    const ERR: u64 = 0;

    struct Clock has key {
        id: rtd::object::UID,
    }

    // no warning
    fun test_clock_mut(_: &mut Clock) {}
    // no warning
    fun test_clock_val(_: Clock) { abort ERR }
}

module rtd::random {
    const ERR: u64 = 0;

    struct Random has key {
        id: rtd::object::UID,
    }

    // no warning
    fun test_random_mut(_: &mut Random) {}
    // no warning
    fun test_random_val(_: Random) { abort ERR }
}


module rtd::object {
    struct UID has store {
        id: address,
    }
}

module rtd::tx_context {
    struct TxContext has drop {}
}
