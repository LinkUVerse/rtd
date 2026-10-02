// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Network node configuration for Rtd data stores.
//!
//! Defines the [`Node`] enum for specifying which Rtd network to connect to (mainnet, testnet,
//! devnet, or custom) and provides URL resolution for both GraphQL and gRPC endpoints.

use std::str::FromStr;

use rtd_types::supported_protocol_versions::Chain;

/// Represents a Rtd network node configuration.
///
/// Used to specify which network the data store should connect to.
#[derive(Clone, Debug)]
pub enum Node {
    /// Rtd mainnet
    Mainnet,
    /// Rtd testnet
    Testnet,
    /// Rtd devnet
    Devnet,
    /// Custom network with a user-provided URL
    Custom(String),
}

impl Node {
    /// Returns the [`Chain`] identifier for this node.
    pub fn chain(&self) -> Chain {
        match self {
            Node::Mainnet => Chain::Mainnet,
            Node::Testnet => Chain::Testnet,
            Node::Devnet => Chain::Unknown,
            Node::Custom(_) => Chain::Unknown,
        }
    }

    /// Returns a human-readable network name.
    pub fn network_name(&self) -> String {
        match self {
            Node::Mainnet => "mainnet".to_string(),
            Node::Testnet => "testnet".to_string(),
            Node::Devnet => "devnet".to_string(),
            Node::Custom(url) => url.clone(),
        }
    }

    /// Returns the GraphQL endpoint URL, or an error for an undeployed public network.
    pub(crate) fn gql_url(&self) -> Result<&str, String> {
        match self {
            Node::Mainnet | Node::Testnet | Node::Devnet => Err(format!(
                "No public RTD {} GraphQL endpoint is configured; pass a custom URL",
                self.network_name()
            )),
            Node::Custom(url) => Ok(url.as_str()),
        }
    }
}

impl FromStr for Node {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mainnet" | "testnet" | "devnet" => Err(format!(
                "No public RTD {s} endpoint is configured; pass a custom GraphQL URL"
            )),
            _ => Ok(Node::Custom(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undeployed_networks_require_an_explicit_url() {
        for network in ["mainnet", "testnet", "devnet"] {
            assert!(Node::from_str(network).is_err());
        }
        for node in [Node::Mainnet, Node::Testnet, Node::Devnet] {
            assert!(node.gql_url().is_err());
        }
        assert_eq!(
            Node::Custom("http://localhost:9000/graphql".into())
                .gql_url()
                .unwrap(),
            "http://localhost:9000/graphql"
        );
    }
}
