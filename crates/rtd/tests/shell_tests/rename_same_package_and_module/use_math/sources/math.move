// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module use_math::both;

public fun foo() {
    let _ = math_a::math::a();
    let _ = math_b::math::a();
}
