// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Network node configuration for Rtd data stores.
//!
//! Defines the [`Node`] enum for specifying which Rtd network to connect to
//! (mainnet, testnet, or custom) and provides URL resolution for both
//! GraphQL and JSON-RPC endpoints.

use rtd_types::supported_protocol_versions::Chain;
use std::str::FromStr;

/// Represents a Rtd network node configuration.
///
/// Used to specify which network the data store should connect to.
#[derive(Clone, Debug)]
pub enum Node {
    /// Rtd mainnet
    Mainnet,
    /// Rtd testnet
    Testnet,
    /// Custom network with a user-provided URL
    Custom(String),
}

impl Node {
    /// Returns the [`Chain`] identifier for this node.
    pub fn chain(&self) -> Chain {
        match self {
            Node::Mainnet => Chain::Mainnet,
            Node::Testnet => Chain::Testnet,
            Node::Custom(_) => Chain::Unknown,
        }
    }

    /// Returns a human-readable network name.
    pub fn network_name(&self) -> String {
        match self {
            Node::Mainnet => "mainnet".to_string(),
            Node::Testnet => "testnet".to_string(),
            Node::Custom(url) => url.clone(),
        }
    }

    /// Returns the GraphQL endpoint URL, or an error for an undeployed public network.
    pub fn gql_url(&self) -> Result<&str, String> {
        match self {
            Node::Mainnet | Node::Testnet => Err(format!(
                "No public RTD {} GraphQL endpoint is configured; pass a custom URL",
                self.network_name()
            )),
            // For custom, assume it's already a GraphQL URL
            Node::Custom(url) => Ok(url.as_str()),
        }
    }

    /// Returns the JSON-RPC endpoint URL, or an error for an undeployed public network.
    pub fn node_url(&self) -> Result<&str, String> {
        match self {
            Node::Mainnet | Node::Testnet => Err(format!(
                "No public RTD {} RPC endpoint is configured; pass a custom URL",
                self.network_name()
            )),
            // For custom, assume it's already an RPC URL
            Node::Custom(url) => Ok(url.as_str()),
        }
    }
}

impl FromStr for Node {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mainnet" | "testnet" => Err(format!(
                "No public RTD {s} endpoint is configured; pass an explicit node URL"
            )),
            _ => Ok(Node::Custom(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_gql_url_returns_provided_url() {
        let url = "https://graphql.devnet.example.com/graphql";
        let node = Node::Custom(url.to_string());
        assert_eq!(node.gql_url().unwrap(), url);
        assert_eq!(node.node_url().unwrap(), url);
    }

    #[test]
    fn from_str_unknown_becomes_custom() {
        let url = "https://graphql.devnet.example.com/graphql";
        match Node::from_str(url).unwrap() {
            Node::Custom(s) => assert_eq!(s, url),
            other => panic!("expected Custom, got {other:?}"),
        }
    }

    #[test]
    fn undeployed_networks_cannot_resolve_to_remote_urls() {
        for network in ["mainnet", "testnet"] {
            assert!(Node::from_str(network).is_err());
        }
        for node in [Node::Mainnet, Node::Testnet] {
            assert!(node.gql_url().is_err());
            assert!(node.node_url().is_err());
        }
    }
}
