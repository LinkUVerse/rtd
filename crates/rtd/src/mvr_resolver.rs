// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Error;
use rtd_protocol_config::Chain;
use rtd_rpc_api::Client;
use rtd_types::base_types::ObjectID;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Serialize)]
pub struct MvrResolver {
    pub names: BTreeSet<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolvedNames {
    pub resolution: BTreeMap<String, PackageId>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageId {
    pub package_id: ObjectID,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NamesRequest {
    pub names: BTreeSet<String>,
}

impl MvrResolver {
    /// Call this function before calling resolve_names to avoid making a call to the resolver if
    /// not needed.
    pub fn should_resolve(&self) -> bool {
        !self.names.is_empty()
    }

    /// Given a set of MVR names, resolve them to their corresponding package IDs. Note that this
    /// API will error if the resolved list length does not match with the given input.
    pub async fn resolve_names(&self, client: &Client) -> Result<ResolvedNames, Error> {
        if self.names.is_empty() {
            return Ok(ResolvedNames {
                resolution: BTreeMap::new(),
            });
        }

        let request = reqwest::Client::new();
        let (url, chain) = mvr_req_url(client).await?;
        let json_body = json!(NamesRequest {
            names: self.names.clone()
        });
        let response = request
            .post(format!("{url}/v1/resolution/bulk"))
            .header("Content-Type", "application/json")
            .json(&json_body)
            .send()
            .await?;

        let resolved_addresses: ResolvedNames = response.json().await?;

        anyhow::ensure!(
            resolved_addresses.resolution.len() == self.names.len(),
            "Could not find package id for {} for {chain} enviroment",
            self.names
                .difference(
                    &resolved_addresses
                        .resolution
                        .keys()
                        .cloned()
                        .collect::<BTreeSet<_>>()
                )
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );

        Ok(resolved_addresses)
    }
}

/// Based on the chain id of the current set environment, return the correct MVR URL to use for
/// resolution.
async fn mvr_req_url(client: &Client) -> Result<(String, &'static str), Error> {
    let chain = client.get_chain_identifier().await?;
    let chain = chain.chain();
    let (variable, name) = match chain {
        Chain::Mainnet => ("RTD_MVR_MAINNET_URL", "mainnet"),
        Chain::Testnet => ("RTD_MVR_TESTNET_URL", "testnet"),
        Chain::Unknown => anyhow::bail!("MVR is unavailable for an unrecognized RTD chain"),
    };
    let url = std::env::var(variable)
        .map_err(|_| anyhow::anyhow!("Set {variable} to a resolver deployed for RTD {name}"))?;
    reqwest::Url::parse(&url)?;
    Ok((url, name))
}
