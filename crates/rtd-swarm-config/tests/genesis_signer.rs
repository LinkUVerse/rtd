// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use fastcrypto::traits::{EncodeDecodeBase64, KeyPair};
use rtd_swarm_config::genesis_signer::verify;
use rtd_types::{base_types::RtdAddress, crypto::RtdKeyPair};
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt, symlink},
    path::PathBuf,
    process::Command,
};

struct Fixture {
    _directory: tempfile::TempDir,
    client: PathBuf,
    key: PathBuf,
    yaml: String,
    address: RtdAddress,
}
impl Fixture {
    fn new(key: RtdKeyPair) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let path = root.join("keys.json");
        fs::write(
            &path,
            serde_json::to_vec(&vec![key.encode_base64()]).unwrap(),
        )
        .unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let address = RtdAddress::from(&key.public());
        let client = root.join("client.yaml");
        fs::write(
            &client,
            format!(
                "keystore:\n  File: '{}'\nactive_address: '{address}'\n",
                path.display()
            ),
        )
        .unwrap();
        fs::set_permissions(&client, fs::Permissions::from_mode(0o600)).unwrap();
        let yaml = format!(
            "ssfn_config_info: null\nvalidator_config_info: null\nparameters:\n  chain_start_timestamp_ms: 1790173312486\n  protocol_version: 138\n  allow_insertion_of_extra_objects: true\n  epoch_duration_ms: 86400000\n  stake_subsidy_start_epoch: 0\n  stake_subsidy_initial_distribution_amount: 1000000000000000\n  stake_subsidy_period_length: 10\n  stake_subsidy_decrease_rate: 1000\naccounts:\n- address: '{address}'\n  gas_amounts: [7, 11]\n- address: '{address}'\n  gas_amounts: [13]\n"
        );
        Self {
            _directory: directory,
            client,
            key: path,
            yaml,
            address,
        }
    }
    fn ed25519() -> Self {
        Self::new(fastcrypto::ed25519::Ed25519KeyPair::generate(&mut rand::thread_rng()).into())
    }
}

#[tokio::test]
async fn all_native_account_schemes_use_yaml_funds_without_mutation() {
    let keys: Vec<RtdKeyPair> = vec![
        fastcrypto::ed25519::Ed25519KeyPair::generate(&mut rand::thread_rng()).into(),
        fastcrypto::secp256k1::Secp256k1KeyPair::generate(&mut rand::thread_rng()).into(),
        fastcrypto::secp256r1::Secp256r1KeyPair::generate(&mut rand::thread_rng()).into(),
    ];
    for key in keys {
        let f = Fixture::new(key);
        let before = fs::metadata(&f.key).unwrap();
        let bytes = fs::read(&f.key).unwrap();
        let report = verify(f.yaml.as_bytes(), &f.client, f.address).unwrap();
        assert_eq!(report.configured_total_mist, "31");
        assert_eq!(report.configured_coin_amounts, [7, 11, 13]);
        assert!(!report.signing_performed && !report.validator_funds_included);
        assert_eq!(fs::read(&f.key).unwrap(), bytes);
        let after = fs::metadata(&f.key).unwrap();
        assert_eq!(
            (
                before.ino(),
                before.mtime(),
                before.mtime_nsec(),
                before.mode()
            ),
            (after.ino(), after.mtime(), after.mtime_nsec(), after.mode())
        );
    }
}

#[tokio::test]
async fn rejects_unfunded_mismatch_ambiguous_and_invalid_keys() {
    let f = Fixture::ed25519();
    let original_client = fs::read_to_string(&f.client).unwrap();
    let original_key = fs::read(&f.key).unwrap();
    let other: RtdAddress = "0x0000000000000000000000000000000000000000000000000000000000001234"
        .parse()
        .unwrap();
    assert!(verify(f.yaml.as_bytes(), &f.client, other).is_err());
    let zero = f.yaml.replace("[7, 11]", "[0]").replace("[13]", "[0]");
    assert!(verify(zero.as_bytes(), &f.client, f.address).is_err());
    // Validator redirection alone is not an explicit deployer allocation.
    let redirected = format!("{zero}validator_token_recipient: '{}'\n", f.address);
    assert!(verify(redirected.as_bytes(), &f.client, f.address).is_err());
    for invalid in [
        original_client.replace(&f.address.to_string(), &other.to_string()),
        format!("{original_client}active_address: '{}'\n", f.address),
        original_client.replace("File:", "External:"),
        original_client.replace(&f.key.display().to_string(), "keys.json"),
        original_client.replace("keystore:", "keystore:\n  External: other"),
    ] {
        fs::write(&f.client, invalid).unwrap();
        assert!(verify(f.yaml.as_bytes(), &f.client, f.address).is_err());
    }
    fs::write(&f.client, original_client).unwrap();
    let mut duplicate: Vec<String> = serde_json::from_slice(&original_key).unwrap();
    duplicate.push(duplicate[0].clone());
    for invalid in [
        b"[]".to_vec(),
        b"[\"secret-bad-key\"]".to_vec(),
        serde_json::to_vec(&duplicate).unwrap(),
        b"{}".to_vec(),
    ] {
        fs::write(&f.key, invalid).unwrap();
        assert!(verify(f.yaml.as_bytes(), &f.client, f.address).is_err());
    }
    let second = Fixture::ed25519();
    fs::write(&f.key, fs::read(second.key).unwrap()).unwrap();
    assert!(verify(f.yaml.as_bytes(), &f.client, f.address).is_err());
}

#[tokio::test]
async fn missing_unsafe_or_linked_inputs_never_initialize_a_wallet() {
    let f = Fixture::ed25519();
    let saved = fs::read(&f.key).unwrap();
    fs::remove_file(&f.key).unwrap();
    assert!(verify(f.yaml.as_bytes(), &f.client, f.address).is_err());
    assert!(!f.key.exists());
    fs::write(&f.key, &saved).unwrap();
    fs::set_permissions(&f.key, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(verify(f.yaml.as_bytes(), &f.client, f.address).is_err());
    fs::set_permissions(&f.key, fs::Permissions::from_mode(0o600)).unwrap();
    fs::set_permissions(&f.client, fs::Permissions::from_mode(0o666)).unwrap();
    assert!(verify(f.yaml.as_bytes(), &f.client, f.address).is_err());
    fs::set_permissions(&f.client, fs::Permissions::from_mode(0o600)).unwrap();
    let stored = f.key.with_extension("saved");
    fs::rename(&f.key, &stored).unwrap();
    symlink(&stored, &f.key).unwrap();
    assert!(verify(f.yaml.as_bytes(), &f.client, f.address).is_err());
    assert_eq!(fs::read(stored).unwrap(), saved);
}

#[tokio::test]
async fn cli_needs_no_tools_ignores_overrides_and_redacts_private_input() {
    let f = Fixture::ed25519();
    let input = f.client.with_extension("genesis.yaml");
    fs::write(&input, &f.yaml).unwrap();
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_rtd-genesis-inspect"))
            .env_clear()
            .env("PATH", "/no-tools")
            .env("REQUIRED_GENESIS_GAS", "999")
            .env("REQUIRED_GENESIS_ADDRESS", "0xdead")
            .arg(&input)
            .arg("--verify-signer")
            .arg(&f.client)
            .arg("--deploy-address")
            .arg(f.address.to_string())
            .output()
            .unwrap()
    };
    let result = run();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["configuredTotalMist"], "31");
    let encoded: Vec<String> = serde_json::from_slice(&fs::read(&f.key).unwrap()).unwrap();
    assert!(!String::from_utf8_lossy(&result.stdout).contains(&encoded[0]));
    fs::write(&f.key, "[\"secret-bad-key\"]").unwrap();
    let bad = run();
    assert!(!bad.status.success() && bad.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&bad.stderr).contains("secret-bad-key"));
}
