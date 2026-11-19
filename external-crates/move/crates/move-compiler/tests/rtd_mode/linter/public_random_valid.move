// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module a::test {
    use rtd::random::{Random, RandomGenerator};

    friend a::test2;

    entry fun basic_random(_r: &Random) {}

    #[allow(lint(public_random))]
    public fun allow_public_random(_r: &Random, _rg: &RandomGenerator) {}

    public(friend) fun public_friend_fn(_r: &Random, _rg: &RandomGenerator) {}

    fun private_fn(_r: &Random, _rg: &RandomGenerator) {}

    #[test_only]
    public fun test_fn(_r: &Random, _rg: &RandomGenerator) {}
}

module a::test2 {

}

#[test_only]
module a::test3 {
    use rtd::random::{Random, RandomGenerator};

    public fun test_fn(_r: &Random, _rg: &RandomGenerator) {}
}

module rtd::object {
    struct UID has store {
        id: address,
    }
}

module rtd::random {
    use rtd::object::UID;

    struct Random has key { id: UID }
    struct RandomGenerator has drop {}

    public fun should_work(_r: &Random, _rg: &RandomGenerator) {}
}
