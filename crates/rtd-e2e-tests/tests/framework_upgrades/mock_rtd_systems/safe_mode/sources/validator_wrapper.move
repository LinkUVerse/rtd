// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module rtd_system::validator_wrapper {
    use rtd::versioned::Versioned;

    public struct ValidatorWrapper has store {
        inner: Versioned
    }
}
