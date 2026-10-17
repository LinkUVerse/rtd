// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
//! Read-only provenance for an existing local node configuration. Never chooses
//! a database, generates keys/ports, creates a genesis or opens a node store.
use anyhow::{Result, ensure};
use fastcrypto::{
    encoding::{Encoding, Hex},
    hash::{HashFunction, Sha256},
    traits::{EncodeDecodeBase64, KeyPair},
};
use rtd_config::{
    NodeConfig,
    node::{AuthorityKeyPairWithPath, KeyPairWithPath},
};
use rtd_types::{
    base_types::RtdAddress,
    crypto::{AuthorityKeyPair, RtdKeyPair},
    multiaddr::{Multiaddr, Protocol},
};
use serde::{
    Deserialize, Serialize,
    de::{MapAccess, SeqAccess, Visitor},
};
use serde_yaml::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    net::SocketAddr,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    pub path: PathBuf,
    pub sha256: String,
}
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    Validator,
    Fullnode,
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub version: u8,
    pub role: Role,
    pub environment_directory: PathBuf,
    pub chain_id: String,
    pub configuration: Pin,
    pub genesis_input: Pin,
    pub genesis_blob: Pin,
    pub extra_inputs: Vec<Pin>,
    pub extra_state_directories: Vec<PathBuf>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub path: PathBuf,
    pub sha256: String,
    pub pointer: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
    pub version: u8,
    pub scope: &'static str,
    pub role: Role,
    pub environment_directory: PathBuf,
    pub chain_id: String,
    pub genesis_sha256: String,
    pub configuration: Pin,
    pub protocol_public_key: String,
    pub rpc_address: SocketAddr,
    pub metrics_address: SocketAddr,
    pub admin_port: u16,
    pub network_address: String,
    pub p2p_address: SocketAddr,
    pub listening_addresses: BTreeMap<String, SocketAddr>,
    pub committee_protocol_public_keys: Vec<String>,
    pub state_directories: Vec<PathBuf>,
    pub inputs: BTreeMap<PathBuf, String>,
    pub field_sources: BTreeMap<String, Source>,
    pub native_defaults: Vec<String>,
    pub effective_configuration_sha256: String,
    pub started: bool,
    pub database_opened: bool,
    pub remote_verified: bool,
}
pub fn hash(raw: &[u8]) -> String {
    Hex::encode(Sha256::digest(raw))
}
fn canonical(path: &Path) -> Result<()> {
    ensure!(
        path.is_absolute() && path.canonicalize()? == path,
        "canonical runtime path required"
    );
    Ok(())
}
fn directory(path: &Path) -> Result<()> {
    canonical(path)?;
    let m = std::fs::symlink_metadata(path)?;
    ensure!(
        m.is_dir() && m.uid() == unsafe { libc::geteuid() } && m.mode() & 0o077 == 0,
        "owned private runtime directory required"
    );
    Ok(())
}
fn read(path: &Path, private: bool, max: u64) -> Result<Vec<u8>> {
    canonical(path)?;
    let mut f = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)?;
    let before = f.metadata()?;
    ensure!(
        before.is_file()
            && before.uid() == unsafe { libc::geteuid() }
            && before.len() <= max
            && before.mode() & if private { 0o077 } else { 0o022 } == 0,
        "bounded owned runtime input required"
    );
    let mut raw = Vec::new();
    (&mut f).take(max + 1).read_to_end(&mut raw)?;
    let after = f.metadata()?;
    ensure!(
        raw.len() as u64 <= max
            && before.len() == after.len()
            && before.mtime() == after.mtime()
            && before.mtime_nsec() == after.mtime_nsec()
            && before.ctime() == after.ctime()
            && before.ctime_nsec() == after.ctime_nsec(),
        "runtime input changed"
    );
    Ok(raw)
}
fn pinned(p: &Pin, private: bool, max: u64) -> Result<Vec<u8>> {
    ensure!(
        p.sha256.len() == 64
            && p.sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "runtime input digest required"
    );
    let raw = read(&p.path, private, max)?;
    ensure!(hash(&raw) == p.sha256, "runtime input digest differs");
    Ok(raw)
}
// Node settings contain finite floating-point budgets as well as integers.
// Check nested duplicates before Value or untagged key/genesis loaders see them.
struct Unique(Value);
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Unique;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("unambiguous runtime configuration")
            }
            fn visit_bool<E>(self, v: bool) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Bool(v)))
            }
            fn visit_i64<E>(self, v: i64) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Number(v.into())))
            }
            fn visit_u64<E>(self, v: u64) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Number(v.into())))
            }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> std::result::Result<Unique, E> {
                if !v.is_finite() {
                    return Err(E::custom("finite runtime budget required"));
                }
                serde_yaml::to_value(v).map(Unique).map_err(E::custom)
            }
            fn visit_str<E>(self, v: &str) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::String(v.into())))
            }
            fn visit_string<E>(self, v: String) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::String(v)))
            }
            fn visit_unit<E>(self) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
            fn visit_none<E>(self) -> std::result::Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Unique, A::Error> {
                let mut out = Vec::new();
                while let Some(Unique(v)) = a.next_element()? {
                    out.push(v);
                }
                Ok(Unique(Value::Sequence(out)))
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Unique, A::Error> {
                let mut out = serde_yaml::Mapping::new();
                while let Some(Unique(k)) = a.next_key()? {
                    if !matches!(k, Value::String(_)) || out.contains_key(&k) {
                        return Err(serde::de::Error::custom(
                            "duplicate or non-string runtime key",
                        ));
                    }
                    let Unique(v) = a.next_value()?;
                    out.insert(k, v);
                }
                Ok(Unique(Value::Mapping(out)))
            }
        }
        d.deserialize_any(V)
    }
}
fn required(v: &Value, keys: &[&str]) -> Result<()> {
    ensure!(
        keys.iter().all(|k| v
            .as_mapping()
            .is_some_and(|m| m.contains_key(&Value::String((*k).into())))),
        "explicit node field required"
    );
    Ok(())
}
fn pointer(parent: &str, key: &str) -> String {
    format!("{parent}/{}", key.replace('~', "~0").replace('/', "~1"))
}
fn sources(v: &Value, label: &str, p: &Pin, out: &mut BTreeMap<String, Source>) {
    match v {
        Value::Mapping(m) if !m.is_empty() => {
            for (k, v) in m {
                sources(v, &pointer(label, k.as_str().unwrap()), p, out)
            }
        }
        Value::Sequence(a) if !a.is_empty() => {
            for (i, v) in a.iter().enumerate() {
                sources(v, &format!("{label}/{i}"), p, out)
            }
        }
        _ => {
            out.insert(
                label.into(),
                Source {
                    path: p.path.clone(),
                    sha256: p.sha256.clone(),
                    pointer: label.into(),
                },
            );
        }
    }
}
fn recognized(raw: &Value, effective: &Value) -> Result<()> {
    if let Value::Mapping(m) = raw {
        let typed = effective
            .as_mapping()
            .ok_or_else(|| anyhow::anyhow!("runtime field shape differs"))?;
        for (k, v) in m {
            recognized(
                v,
                typed
                    .get(k)
                    .ok_or_else(|| anyhow::anyhow!("unknown or omitted runtime field"))?,
            )?;
        }
    }
    if let Value::Sequence(a) = raw {
        let b = effective
            .as_sequence()
            .ok_or_else(|| anyhow::anyhow!("runtime field shape differs"))?;
        ensure!(a.len() == b.len(), "runtime array shape differs");
        for (i, item) in a.iter().enumerate() {
            recognized(item, &b[i])?;
        }
    }
    Ok(())
}
fn defaults(raw: &Value, effective: &Value, label: &str, out: &mut Vec<String>) {
    if let Value::Mapping(m) = effective {
        for (k, v) in m {
            let p = pointer(label, k.as_str().unwrap());
            match raw.as_mapping().and_then(|m| m.get(k)) {
                Some(r) => defaults(r, v, &p, out),
                None => out.push(p),
            }
        }
    }
}
fn paths(
    v: &Value,
    key: &str,
    inputs: &BTreeMap<PathBuf, String>,
    states: &[PathBuf],
) -> Result<()> {
    match v {
        Value::Mapping(m) => {
            for (k, v) in m {
                paths(v, k.as_str().unwrap(), inputs, states)?;
            }
        }
        Value::Sequence(a) => {
            for v in a {
                paths(v, key, inputs, states)?;
            }
        }
        Value::String(s) if !s.is_empty() => {
            // Native network multiaddrs are not filesystem paths.
            if s.starts_with('/') && s.parse::<rtd_types::multiaddr::Multiaddr>().is_ok() {
                return Ok(());
            }
            let filesystem = Path::new(s).is_absolute()
                || key == "path"
                || key.ends_with("-path")
                || key.ends_with("_path")
                || key.ends_with("-dir")
                || key.ends_with("_dir")
                || key.ends_with("-directory")
                || key.ends_with("_directory");
            if filesystem {
                let p = Path::new(s);
                canonical(p)?;
                ensure!(
                    inputs.contains_key(p) || states.iter().any(|d| p.starts_with(d)),
                    "unbound runtime file or state path"
                );
            }
        }
        _ => (),
    }
    Ok(())
}
pub fn inspect(path: &Path, expected: &str) -> Result<Binding> {
    let original = pinned(
        &Pin {
            path: path.into(),
            sha256: expected.into(),
        },
        true,
        1_048_576,
    )?;
    let Unique(request_value) = serde_yaml::from_slice(&original)?;
    required(
        &request_value,
        &[
            "version",
            "role",
            "environmentDirectory",
            "chainId",
            "configuration",
            "genesisInput",
            "genesisBlob",
            "extraInputs",
            "extraStateDirectories",
        ],
    )?;
    let r: Request = serde_json::from_slice(&original)?;
    ensure!(
        r.version == 1 && r.extra_inputs.len() <= 128 && r.extra_state_directories.len() <= 32,
        "runtime request version or budget differs"
    );
    directory(&r.environment_directory)?;
    let yaml = pinned(&r.genesis_input, true, 1_048_576)?;
    let blob = pinned(&r.genesis_blob, false, 128 * 1024 * 1024)?;
    let verified = crate::genesis_inspection::verify_blob(&yaml, &blob)?;
    ensure!(
        verified.chain_id == r.chain_id,
        "runtime genesis chain differs"
    );
    let genesis: rtd_config::genesis::Genesis = bcs::from_bytes(&blob)?;
    let raw = pinned(&r.configuration, true, 4 * 1024 * 1024)?;
    let Unique(value) = serde_yaml::from_slice(&raw)?;
    required(
        &value,
        &[
            "protocol-key-pair",
            "worker-key-pair",
            "account-key-pair",
            "network-key-pair",
            "db-path",
            "network-address",
            "json-rpc-address",
            "metrics-address",
            "admin-interface-port",
            "p2p-config",
            "genesis",
            "authority-store-pruning-config",
            "checkpoint-executor-config",
            "grpc-concurrency-limit",
            "grpc-load-shed",
        ],
    )?;
    required(&value["p2p-config"], &["listen-address"])?;
    required(
        &value["authority-store-pruning-config"],
        &[
            "num-latest-epoch-dbs-to-retain",
            "epoch-db-pruning-period-secs",
            "num-epochs-to-retain",
            "max-checkpoints-in-batch",
            "max-transactions-in-batch",
        ],
    )?;
    required(
        &value["checkpoint-executor-config"],
        &[
            "checkpoint-execution-max-concurrency",
            "local-execution-timeout-sec",
        ],
    )?;
    let mut inputs = BTreeMap::new();
    let mut private = BTreeSet::new();
    for p in [&r.configuration, &r.genesis_input, &r.genesis_blob]
        .into_iter()
        .chain(r.extra_inputs.iter())
    {
        let is_private = p.path != r.genesis_blob.path;
        let data = pinned(p, is_private, 128 * 1024 * 1024)?;
        if let Some(old) = inputs.insert(p.path.clone(), hash(&data)) {
            ensure!(old == p.sha256, "runtime input collision");
        }
        if is_private {
            private.insert(p.path.clone());
        }
    }
    inputs.insert(path.into(), expected.into());
    private.insert(path.into());
    let mut protocol_file = None;
    let mut key_files = BTreeMap::new();
    for label in [
        "protocol-key-pair",
        "worker-key-pair",
        "account-key-pair",
        "network-key-pair",
    ] {
        let m = value[label]
            .as_mapping()
            .ok_or_else(|| anyhow::anyhow!("explicit key location required"))?;
        ensure!(m.len() == 1, "ambiguous node key location");
        if let Some(p) = value[label]["path"].as_str() {
            let p = Path::new(p);
            let data = read(p, true, 4096)?;
            let encoded = std::str::from_utf8(&data)?.trim();
            if label == "protocol-key-pair" {
                protocol_file = Some(AuthorityKeyPair::decode_base64(encoded)?);
            } else {
                let kp = RtdKeyPair::decode_base64(encoded)?;
                ensure!(
                    label == "account-key-pair" || matches!(kp, RtdKeyPair::Ed25519(_)),
                    "Ed25519 runtime node key required"
                );
                key_files.insert(label, kp);
            }
            if let Some(old) = inputs.insert(p.into(), hash(&data)) {
                ensure!(old == hash(&data), "runtime key input collision");
            }
            private.insert(p.into());
        } else {
            required(&value[label], &["value"])?;
        }
    }
    ensure!(
        value["genesis"].as_mapping().is_some_and(|m| m.len() == 1)
            && value["genesis"]["genesis-file-location"]
                .as_str()
                .is_some_and(|p| Path::new(p) == r.genesis_blob.path),
        "explicit pinned file genesis required"
    );
    let mut node: NodeConfig = serde_yaml::from_slice(&raw)?;
    let effective = serde_yaml::to_value(&node)?;
    recognized(&value, &effective)?;
    // Use decoded pinned bytes rather than lazy key-file getters, whose legacy
    // failure mode is panic. This changes only the inspection snapshot.
    if let Some(kp) = protocol_file {
        node.protocol_key_pair = AuthorityKeyPairWithPath::new(kp);
    }
    if let Some(kp) = key_files.remove("worker-key-pair") {
        node.worker_key_pair = KeyPairWithPath::new(kp);
    }
    if let Some(kp) = key_files.remove("network-key-pair") {
        node.network_key_pair = KeyPairWithPath::new(kp);
    }
    if let Some(kp) = key_files.remove("account-key-pair") {
        node.account_key_pair = KeyPairWithPath::new(kp);
    }
    for kp in [&node.worker_key_pair, &node.network_key_pair] {
        ensure!(
            matches!(kp.keypair(), RtdKeyPair::Ed25519(_)),
            "Ed25519 node key required"
        );
    }
    ensure!(
        node.genesis.genesis()?.to_bytes() == blob,
        "node genesis differs"
    );
    let authority = node.protocol_public_key();
    let mut listening_addresses = BTreeMap::from([
        ("tcp/rpc".into(), node.json_rpc_address),
        ("tcp/metrics".into(), node.metrics_address),
        (
            "tcp/admin".into(),
            SocketAddr::from(([127, 0, 0, 1], node.admin_interface_port)),
        ),
        (
            "tcp/network".into(),
            node.network_address
                .to_socket_addr()
                .map_err(|_| anyhow::anyhow!("fixed network socket required"))?,
        ),
        ("udp/p2p".into(), node.p2p_config.listen_address),
    ]);
    match r.role {
        Role::Validator => {
            let set = genesis.validator_set_for_tooling();
            let validator = set
                .iter()
                .find(|v| {
                    v.metadata.protocol_pubkey_bytes == node.protocol_key_pair().public().as_ref()
                })
                .ok_or_else(|| anyhow::anyhow!("validator identity missing from genesis"))?;
            ensure!(
                node.consensus_config.is_some()
                    && validator.metadata.network_pubkey_bytes
                        == node.network_key_pair().public().as_ref()
                    && validator.metadata.worker_pubkey_bytes
                        == node.worker_key_pair().public().as_ref()
                    && validator.metadata.rtd_address
                        == RtdAddress::from(&node.account_key_pair.keypair().public())
                    && validator.metadata.net_address == node.network_address.to_string(),
                "validator keys or network differ from genesis"
            );
            let primary: Multiaddr = validator.metadata.primary_address.parse()?;
            let consensus = node.consensus_config.as_ref().expect("checked consensus");
            let listen = consensus.listen_address.clone().unwrap_or_else(|| {
                if primary.is_localhost_ip() {
                    primary
                } else {
                    primary.with_zero_ip()
                }
            });
            // Mysticeti's tonic transport accepts historical UDP-labelled
            // committee addresses but binds a TCP socket at that same port.
            let socket = listen
                .to_socket_addr()
                .ok()
                .or_else(|| {
                    matches!(
                        listen.iter().next(),
                        Some(Protocol::Ip4(_) | Protocol::Ip6(_))
                    )
                    .then(|| listen.udp_multiaddr_to_listen_address())
                    .flatten()
                })
                .ok_or_else(|| anyhow::anyhow!("fixed consensus socket required"))?;
            listening_addresses.insert("tcp/consensus".into(), socket);
        }
        Role::Fullnode => ensure!(
            node.consensus_config.is_none()
                && !genesis
                    .committee()
                    .voting_rights
                    .iter()
                    .any(|(a, _)| a == &authority),
            "fullnode cannot adopt validator identity or consensus"
        ),
    }
    ensure!(
        listening_addresses.values().all(|v| v.port() != 0),
        "fixed nonzero listening socket required"
    );
    let mut states = vec![node.db_path.clone()];
    if let Some(c) = &node.consensus_config {
        states.push(c.db_path.clone());
    }
    if let Some(p) = &node.checkpoint_executor_config.data_ingestion_dir {
        states.push(p.clone());
    }
    states.extend(r.extra_state_directories);
    let state_count = states.len();
    states.sort();
    states.dedup();
    ensure!(
        states.len() == state_count,
        "duplicate runtime state directories"
    );
    for (i, p) in states.iter().enumerate() {
        directory(p)?;
        ensure!(
            p.starts_with(&r.environment_directory)
                && inputs.keys().all(|input| !input.starts_with(p)),
            "runtime state escapes environment or overlaps fixed input"
        );
        for q in &states[..i] {
            ensure!(
                !p.starts_with(q) && !q.starts_with(p),
                "runtime state directories overlap"
            );
        }
    }
    paths(&value, "", &inputs, &states)?;
    ensure!(
        node.network_address
            .to_socket_addr()
            .map_err(|_| anyhow::anyhow!("fixed node network socket required"))?
            .port()
            != 0
            && node.json_rpc_address.port() != 0
            && node.metrics_address.port() != 0
            && node.admin_interface_port != 0
            && node.p2p_config.listen_address.port() != 0
            && node
                .checkpoint_executor_config
                .checkpoint_execution_max_concurrency
                > 0
            && node.checkpoint_executor_config.local_execution_timeout_sec > 0,
        "explicit node endpoint or execution budget required"
    );
    let mut field_sources = BTreeMap::new();
    sources(&value, "", &r.configuration, &mut field_sources);
    let mut native_defaults = Vec::new();
    defaults(&value, &effective, "", &mut native_defaults);
    for (p, sha) in &inputs {
        ensure!(
            hash(&read(p, private.contains(p), 128 * 1024 * 1024)?) == *sha,
            "runtime input changed after resolution"
        );
    }
    Ok(Binding {
        version: 1,
        scope: "persisted-local-node-configuration",
        role: r.role,
        environment_directory: r.environment_directory,
        chain_id: r.chain_id,
        genesis_sha256: r.genesis_blob.sha256,
        configuration: r.configuration,
        protocol_public_key: authority.to_string(),
        rpc_address: node.json_rpc_address,
        metrics_address: node.metrics_address,
        admin_port: node.admin_interface_port,
        network_address: node.network_address.to_string(),
        p2p_address: node.p2p_config.listen_address,
        listening_addresses,
        committee_protocol_public_keys: genesis
            .committee()
            .voting_rights
            .iter()
            .map(|(key, _)| key.to_string())
            .collect(),
        state_directories: states,
        inputs,
        field_sources,
        native_defaults,
        effective_configuration_sha256: hash(&serde_yaml::to_vec(&effective)?),
        started: false,
        database_opened: false,
        remote_verified: false,
    })
}
