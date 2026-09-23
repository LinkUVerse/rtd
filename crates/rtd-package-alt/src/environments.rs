// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use move_package_alt::schema::Environment;
use rtd_sdk::types::{
    digests::{get_mainnet_chain_identifier, get_testnet_chain_identifier},
    supported_protocol_versions::Chain,
};

pub fn testnet_environment() -> Option<Environment> {
    get_testnet_chain_identifier().map(|id| Environment {
        name: Chain::Testnet.as_str().to_string(),
        id: id.to_string(),
    })
}

pub fn mainnet_environment() -> Option<Environment> {
    get_mainnet_chain_identifier().map(|id| Environment {
        name: Chain::Mainnet.as_str().to_string(),
        id: id.to_string(),
    })
}

/// An opaque environment for source-only compilation in tests. It never
/// identifies a public RTD network or a real genesis checkpoint.
pub fn local_test_environment() -> Environment {
    Environment::new("rtd-local-test".into(), "rtd-local-test".into())
}
