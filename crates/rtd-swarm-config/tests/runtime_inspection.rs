// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
use fastcrypto::traits::EncodeDecodeBase64;
use rand::{SeedableRng, rngs::StdRng};
use rtd_config::{NodeConfig, node::Genesis};
use rtd_swarm_config::{
    genesis_config::GenesisConfig,
    network_config_builder::ConfigBuilder,
    node_config_builder::FullnodeConfigBuilder,
    runtime_inspection::{self, Pin, Role},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    num::NonZeroUsize,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};
const YAML: &str = "ssfn_config_info: null\nvalidator_config_info: null\nparameters:\n  chain_start_timestamp_ms: 1790173312486\n  protocol_version: 138\n  allow_insertion_of_extra_objects: true\n  epoch_duration_ms: 86400000\n  stake_subsidy_start_epoch: 0\n  stake_subsidy_initial_distribution_amount: 1000000000000000\n  stake_subsidy_period_length: 10\n  stake_subsidy_decrease_rate: 1000\naccounts:\n- address: '0x0000000000000000000000000000000000000000000000000000000000001234'\n  gas_amounts: [5200000000000000]\n";
#[tokio::test]
async fn group_report_exposes_native_listener_transports_and_exact_genesis_committee() {
    for role in [Role::Validator, Role::Fullnode] {
        let f = Fixture::new(role);
        let before = f.files();
        let report = f.inspect().unwrap();
        let sockets = report["listeningAddresses"].as_object().unwrap();
        assert_eq!(sockets.len(), if role == Role::Validator { 6 } else { 5 });
        assert_eq!(sockets["tcp/rpc"], f.node.json_rpc_address.to_string());
        assert_eq!(sockets["tcp/metrics"], f.node.metrics_address.to_string());
        assert_eq!(
            sockets["tcp/admin"],
            format!("127.0.0.1:{}", f.node.admin_interface_port)
        );
        assert_eq!(
            sockets["udp/p2p"],
            f.node.p2p_config.listen_address.to_string()
        );
        assert_eq!(
            report["committeeProtocolPublicKeys"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        let member = report["committeeProtocolPublicKeys"]
            .as_array()
            .unwrap()
            .contains(&report["protocolPublicKey"]);
        assert_eq!(member, role == Role::Validator);
        assert_eq!(before, f.files());
    }
}
#[tokio::test]
async fn consensus_listener_override_is_derived_natively_and_zero_port_is_rejected() {
    let mut f = Fixture::new(Role::Validator);
    let mut value = serde_yaml::to_value(&f.node).unwrap();
    value["consensus-config"]["listen-address"] =
        serde_yaml::Value::String("/ip4/127.0.0.1/tcp/12345".into());
    f.node(&value);
    assert_eq!(
        f.inspect().unwrap()["listeningAddresses"]["tcp/consensus"],
        "127.0.0.1:12345"
    );
    value["consensus-config"]["listen-address"] =
        serde_yaml::Value::String("/ip4/127.0.0.1/udp/12345".into());
    f.node(&value);
    assert_eq!(
        f.inspect().unwrap()["listeningAddresses"]["tcp/consensus"],
        "127.0.0.1:12345"
    );
    value["consensus-config"]["listen-address"] =
        serde_yaml::Value::String("/ip4/127.0.0.1/tcp/0".into());
    f.node(&value);
    assert!(f.inspect().is_err());
}
fn private(p: &Path, raw: impl AsRef<[u8]>) {
    fs::write(p, raw).unwrap();
    fs::set_permissions(p, fs::Permissions::from_mode(0o600)).unwrap();
}
fn directory(p: &Path) {
    fs::create_dir_all(p).unwrap();
    fs::set_permissions(p, fs::Permissions::from_mode(0o700)).unwrap();
}
fn pin(p: &Path) -> Pin {
    Pin {
        path: p.into(),
        sha256: runtime_inspection::hash(&fs::read(p).unwrap()),
    }
}
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    node: NodeConfig,
    path: PathBuf,
    request: Value,
    sha: String,
}
impl Fixture {
    fn new(role: Role) -> Self {
        static BASE: OnceLock<(Vec<u8>, Vec<u8>, Vec<u8>)> = OnceLock::new();
        let base = BASE.get_or_init(|| {
            // Explicit fixture genesis creation precedes inspection. These keys
            // and ports never originate from ordinary prepare/start defaults.
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().canonicalize().unwrap();
            let network = ConfigBuilder::new(&root)
                .committee_size(NonZeroUsize::new(1).unwrap())
                .with_genesis_config(serde_yaml::from_str::<GenesisConfig>(YAML).unwrap())
                .rng(StdRng::from_seed([3; 32]))
                .build();
            let fullnode = FullnodeConfigBuilder::new()
                .with_config_directory(root)
                .build(&mut StdRng::from_seed([4; 32]), &network);
            (
                serde_yaml::to_vec(&network.validator_configs[0]).unwrap(),
                serde_yaml::to_vec(&fullnode).unwrap(),
                network.genesis.to_bytes(),
            )
        });
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        private(&root.join("genesis-input.yaml"), YAML);
        private(&root.join("genesis.blob"), &base.2);
        let mut node: NodeConfig = serde_yaml::from_slice(if role == Role::Validator {
            &base.0
        } else {
            &base.1
        })
        .unwrap();
        node.genesis = Genesis::new_from_file(root.join("genesis.blob"));
        node.db_path = root.join("db");
        directory(&node.db_path);
        if let Some(c) = &mut node.consensus_config {
            c.db_path = root.join("consensus");
            directory(&c.db_path);
        }
        if role == Role::Fullnode {
            let p = root.join("checkpoints");
            directory(&p);
            node.checkpoint_executor_config.data_ingestion_dir = Some(p);
        }
        private(&root.join("node.yaml"), serde_yaml::to_vec(&node).unwrap());
        let chain = rtd_swarm_config::genesis_inspection::verify_blob(YAML.as_bytes(), &base.2)
            .unwrap()
            .chain_id;
        let request = json!({"version":1,"role":role,"environmentDirectory":root,"chainId":chain,"configuration":pin(&root.join("node.yaml")),"genesisInput":pin(&root.join("genesis-input.yaml")),"genesisBlob":pin(&root.join("genesis.blob")),"extraInputs":[],"extraStateDirectories":[]});
        let path = root.join("request.json");
        private(&path, serde_json::to_vec(&request).unwrap());
        let sha = pin(&path).sha256;
        Self {
            _temp: temp,
            root,
            node,
            path,
            request,
            sha,
        }
    }
    fn save(&mut self) {
        private(&self.path, serde_json::to_vec(&self.request).unwrap());
        self.sha = pin(&self.path).sha256;
    }
    fn node(&mut self, value: &serde_yaml::Value) {
        private(
            &self.root.join("node.yaml"),
            serde_yaml::to_vec(value).unwrap(),
        );
        self.request["configuration"] =
            serde_json::to_value(pin(&self.root.join("node.yaml"))).unwrap();
        self.save();
    }
    fn inspect(&self) -> anyhow::Result<Value> {
        serde_json::to_value(runtime_inspection::inspect(&self.path, &self.sha)?)
            .map_err(Into::into)
    }
    fn files(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        fn walk(p: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for e in fs::read_dir(p).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    walk(&p, out)
                } else {
                    out.insert(p.clone(), fs::read(p).unwrap());
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(&self.root, &mut out);
        out
    }
    fn cli(&self) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_rtd-genesis-inspect"))
            .env_clear()
            .env("PATH", "/no-tools")
            .env("RTD_CONFIG_DIR", "/should-not-create")
            .env("RTD_PROTOCOL_CONFIG_OVERRIDE_ENABLE", "1")
            .env("RTD_PROTOCOL_CONFIG_OVERRIDE_max_tx_size_bytes", "1")
            .args(["--inspect-node", self.path.to_str().unwrap(), &self.sha])
            .output()
            .unwrap()
    }
}
#[tokio::test]
async fn validator_and_fullnode_bind_real_genesis_and_original_fields_without_state_effects() {
    for role in [Role::Validator, Role::Fullnode] {
        let f = Fixture::new(role);
        let before = f.files();
        let a = f.inspect().unwrap();
        assert_eq!(a, f.inspect().unwrap());
        assert_eq!(a["chainId"], f.request["chainId"]);
        assert_eq!(a["configuration"], f.request["configuration"]);
        assert_eq!(a["started"], false);
        assert_eq!(a["databaseOpened"], false);
        assert_eq!(a["remoteVerified"], false);
        assert_eq!(a["inputs"].as_object().unwrap().len(), 4);
        assert!(
            !a.to_string()
                .contains(&f.node.worker_key_pair.keypair().encode_base64())
        );
        let source: serde_json::Value =
            serde_yaml::from_slice(&fs::read(f.root.join("node.yaml")).unwrap()).unwrap();
        for s in a["fieldSources"].as_object().unwrap().values() {
            assert_eq!(s["path"], f.request["configuration"]["path"]);
            assert_eq!(s["sha256"], f.request["configuration"]["sha256"]);
            assert!(source.pointer(s["pointer"].as_str().unwrap()).is_some());
        }
        assert_eq!(before, f.files());
        assert_eq!(fs::read_dir(f.root.join("db")).unwrap().count(), 0);
    }
}
#[tokio::test]
async fn missing_identity_endpoints_budgets_unknowns_and_duplicate_keys_never_trigger_defaults() {
    let mut f = Fixture::new(Role::Validator);
    let original: serde_yaml::Value = serde_yaml::to_value(&f.node).unwrap();
    for field in [
        "protocol-key-pair",
        "worker-key-pair",
        "account-key-pair",
        "network-key-pair",
        "network-address",
        "metrics-address",
        "admin-interface-port",
        "grpc-concurrency-limit",
        "grpc-load-shed",
        "p2p-config",
        "authority-store-pruning-config",
        "checkpoint-executor-config",
    ] {
        let mut value = original.clone();
        value
            .as_mapping_mut()
            .unwrap()
            .remove(&serde_yaml::Value::String(field.into()));
        f.node(&value);
        let before = f.files();
        assert!(f.inspect().is_err(), "{field}");
        assert_eq!(before, f.files());
    }
    let mut value = original.clone();
    value["unexpected-private-field"] = serde_yaml::Value::String("secret-value-never-log".into());
    f.node(&value);
    assert!(f.inspect().is_err());
    let out = f.cli();
    assert!(!out.status.success());
    assert!(!String::from_utf8_lossy(&out.stderr).contains("secret-value-never-log"));
    let raw = serde_yaml::to_string(&original).unwrap();
    private(
        &f.root.join("node.yaml"),
        format!("{raw}\nmetrics-address: '127.0.0.1:12345'\n"),
    );
    f.request["configuration"] = serde_json::to_value(pin(&f.root.join("node.yaml"))).unwrap();
    f.save();
    assert!(f.inspect().is_err());
}
#[tokio::test]
async fn pinned_file_keys_are_decoded_without_lazy_panics_and_drift_is_refused() {
    let mut f = Fixture::new(Role::Validator);
    let key = f.root.join("worker.key");
    private(&key, f.node.worker_key_pair.keypair().encode_base64());
    let mut value = serde_yaml::to_value(&f.node).unwrap();
    value["worker-key-pair"] = serde_yaml::from_str(&format!("path: {}", key.display())).unwrap();
    f.node(&value);
    let before = f.files();
    let b = f.inspect().unwrap();
    assert_eq!(b["inputs"][key.to_str().unwrap()], pin(&key).sha256);
    assert_eq!(before, f.files());
    private(&key, "secret-invalid-node-key");
    let before = f.files();
    assert!(f.inspect().is_err());
    assert_eq!(before, f.files());
    let out = f.cli();
    assert!(!out.status.success());
    assert!(!String::from_utf8_lossy(&out.stderr).contains("secret-invalid-node-key"));
}
#[tokio::test]
async fn account_keys_preserve_all_native_schemes_for_inline_and_pinned_files() {
    use rtd_types::crypto::{RtdKeyPair, get_key_pair_from_rng};
    let mut rng = StdRng::from_seed([8; 32]);
    for key in [
        RtdKeyPair::Ed25519(get_key_pair_from_rng(&mut rng).1),
        RtdKeyPair::Secp256k1(get_key_pair_from_rng(&mut rng).1),
        RtdKeyPair::Secp256r1(get_key_pair_from_rng(&mut rng).1),
    ] {
        let mut f = Fixture::new(Role::Fullnode);
        let mut value = serde_yaml::to_value(&f.node).unwrap();
        value["account-key-pair"] =
            serde_yaml::from_str(&format!("value: {}", key.encode_base64())).unwrap();
        f.node(&value);
        assert!(f.inspect().is_ok());
        let path = f.root.join("account.key");
        private(&path, key.encode_base64());
        value["account-key-pair"] =
            serde_yaml::from_str(&format!("path: {}", path.display())).unwrap();
        f.node(&value);
        let before = f.files();
        assert!(f.inspect().is_ok());
        assert_eq!(before, f.files());
    }
}
#[tokio::test]
async fn chain_validator_identity_role_and_genesis_authority_must_match() {
    let mut f = Fixture::new(Role::Validator);
    let chain = f.request["chainId"].clone();
    f.request["chainId"] = json!("wrong-chain");
    f.save();
    assert!(f.inspect().is_err());
    f.request["chainId"] = chain;
    f.request["role"] = json!("fullnode");
    f.save();
    assert!(f.inspect().is_err());
    let mut f = Fixture::new(Role::Fullnode);
    f.request["role"] = json!("validator");
    f.save();
    assert!(f.inspect().is_err());
    let mut f = Fixture::new(Role::Validator);
    let mut value = serde_yaml::to_value(&f.node).unwrap();
    value["worker-key-pair"] = value["network-key-pair"].clone();
    f.node(&value);
    assert!(f.inspect().is_err());
    let mut f = Fixture::new(Role::Validator);
    private(
        &f.root.join("genesis-input.yaml"),
        YAML.replace("epoch_duration_ms: 86400000", "epoch_duration_ms: 300000"),
    );
    f.request["genesisInput"] =
        serde_json::to_value(pin(&f.root.join("genesis-input.yaml"))).unwrap();
    f.save();
    assert!(f.inspect().is_err());
}
#[tokio::test]
async fn private_state_paths_and_indirect_inputs_are_never_created_repaired_or_selected_implicitly()
{
    for case in 0..5 {
        let mut f = Fixture::new(Role::Fullnode);
        let mut value = serde_yaml::to_value(&f.node).unwrap();
        match case {
            0 => value["db-path"] = serde_yaml::to_value(f.root.join("missing-db")).unwrap(),
            1 => value["db-path"] = serde_yaml::to_value(&f.root).unwrap(),
            2 => {
                let link = f.root.join("db-link");
                symlink(f.root.join("db"), &link).unwrap();
                value["db-path"] = serde_yaml::to_value(link).unwrap();
            }
            3 => f.request["extraStateDirectories"] = json!([f.root.join("db")]),
            _ => {
                fs::set_permissions(f.root.join("db"), fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
        f.node(&value);
        let before = f.files();
        assert!(f.inspect().is_err(), "case {case}");
        assert_eq!(before, f.files());
        assert!(!f.root.join("missing-db").exists());
    }
    let mut f = Fixture::new(Role::Fullnode);
    let mut value = serde_yaml::to_value(&f.node).unwrap();
    value["genesis"]["genesis-file-location"] = serde_yaml::to_value("genesis.blob").unwrap();
    f.node(&value);
    assert!(f.inspect().is_err());
}
#[tokio::test]
async fn real_inspector_cli_ignores_ambient_node_configuration_and_keeps_legacy_genesis_mode() {
    let f = Fixture::new(Role::Fullnode);
    let before = f.files();
    let out = f.cli();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&out.stdout).unwrap(),
        f.inspect().unwrap()
    );
    assert_eq!(before, f.files());
    let out = Command::new(env!("CARGO_BIN_EXE_rtd-genesis-inspect"))
        .env_clear()
        .env("PATH", "/no-tools")
        .arg(f.root.join("genesis-input.yaml"))
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&out.stdout).unwrap()["generatedGenesis"],
        false
    );
}
