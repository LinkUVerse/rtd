// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Read-only local deployment signer admission. No wallet initialization,
//! aliases, key export, signing, network access or implicit account selection.
use anyhow::{Result, ensure};
use fastcrypto::traits::EncodeDecodeBase64;
use rtd_types::{base_types::RtdAddress, crypto::RtdKeyPair};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerAdmission {
    pub version: u8,
    pub source_sha256: String,
    pub deploy_address: RtdAddress,
    pub configured_coin_amounts: Vec<u64>,
    pub configured_total_mist: String,
    pub validator_allocation_policy: &'static str,
    pub validator_funds_included: bool,
    pub local_key_present: bool,
    pub signing_performed: bool,
}

fn read_owned(path: &Path, private: bool) -> Result<Vec<u8>> {
    ensure!(
        path.is_absolute() && path.canonicalize()? == path,
        "canonical absolute input required"
    );
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    let before = file.metadata()?;
    ensure!(
        before.is_file()
            && before.len() <= 1024 * 1024
            && before.uid() == unsafe { libc::geteuid() }
            && before.mode() & if private { 0o077 } else { 0o022 } == 0,
        "bounded owned input with safe permissions required"
    );
    let mut bytes = Vec::new();
    (&mut file).take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
    let after = file.metadata()?;
    ensure!(
        bytes.len() <= 1024 * 1024
            && before.len() == after.len()
            && before.mtime() == after.mtime()
            && before.mtime_nsec() == after.mtime_nsec()
            && before.ctime() == after.ctime()
            && before.ctime_nsec() == after.ctime_nsec(),
        "input changed during inspection"
    );
    Ok(bytes)
}

pub fn verify_active(input: &[u8], client: &Path) -> Result<SignerAdmission> {
    let bytes = read_owned(client, false)?;
    let crate::genesis_inspection::Unique(value) = serde_yaml::from_slice(&bytes)
        .map_err(|_| anyhow::anyhow!("invalid or duplicate client YAML"))?;
    let address = value["active_address"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("explicit active address required"))?
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid active address"))?;
    verify(input, client, address)
}

pub fn verify(input: &[u8], client: &Path, address: RtdAddress) -> Result<SignerAdmission> {
    let inspected = crate::genesis_inspection::inspect(input)?;
    let amounts: Vec<_> = inspected
        .accounts
        .iter()
        .filter(|a| a.address == Some(address))
        .flat_map(|a| a.gas_amounts.iter().copied())
        .collect();
    let total: u128 = amounts.iter().map(|v| *v as u128).sum();
    ensure!(
        total > 0,
        "deployment address requires positive explicit YAML funding"
    );
    let bytes = read_owned(client, false)?;
    let crate::genesis_inspection::Unique(value) = serde_yaml::from_slice(&bytes)
        .map_err(|_| anyhow::anyhow!("invalid or duplicate client YAML"))?;
    let active: RtdAddress = value["active_address"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("explicit active address required"))?
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid active address"))?;
    ensure!(
        active == address,
        "active address differs from selected deployment address"
    );
    let store = value["keystore"]
        .as_mapping()
        .ok_or_else(|| anyhow::anyhow!("explicit local file keystore required"))?;
    ensure!(store.len() == 1, "ambiguous keystore configuration");
    let path = PathBuf::from(value["keystore"]["File"].as_str().ok_or_else(|| {
        anyhow::anyhow!("local File keystore required; external signers need separate admission")
    })?);
    let key_bytes = read_owned(&path, true)?;
    let encoded: Vec<String> = serde_json::from_slice(&key_bytes)
        .map_err(|_| anyhow::anyhow!("invalid local keystore"))?;
    ensure!(
        !encoded.is_empty() && encoded.len() <= 1024,
        "invalid keystore key count"
    );
    let mut addresses = BTreeSet::new();
    for item in &encoded {
        let key = RtdKeyPair::decode_base64(item)
            .map_err(|_| anyhow::anyhow!("invalid native account key"))?;
        ensure!(
            addresses.insert(RtdAddress::from(&key.public())),
            "duplicate keystore account"
        );
    }
    ensure!(
        addresses.contains(&address),
        "deployment private key absent from local keystore"
    );
    Ok(SignerAdmission {
        version: 1,
        source_sha256: inspected.source_sha256,
        deploy_address: address,
        configured_coin_amounts: amounts,
        configured_total_mist: total.to_string(),
        validator_allocation_policy: inspected.validator_allocation_policy,
        validator_funds_included: false,
        local_key_present: true,
        signing_performed: false,
    })
}
