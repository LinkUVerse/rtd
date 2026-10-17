// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
//! Explicit, isolated integration fixture; deterministic public test keys only.
//! This is not a production genesis/deployment entry or an ordinary start path.
use anyhow::{Result, ensure};
use rand::{SeedableRng, rngs::StdRng};
use rtd_config::node::Genesis;
use rtd_swarm_config::{
    genesis_config::GenesisConfig,
    network_config_builder::ConfigBuilder,
    node_config_builder::FullnodeConfigBuilder,
    runtime_inspection::{self, Pin, Request, Role},
};
use serde_json::json;
use std::{
    fs,
    io::Write,
    num::NonZeroUsize,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};
const YAML: &str = "ssfn_config_info: null\nvalidator_config_info: null\nparameters:\n  chain_start_timestamp_ms: 1790173312486\n  protocol_version: 138\n  allow_insertion_of_extra_objects: true\n  epoch_duration_ms: 86400000\n  stake_subsidy_start_epoch: 0\n  stake_subsidy_initial_distribution_amount: 1000000000000000\n  stake_subsidy_period_length: 10\n  stake_subsidy_decrease_rate: 1000\naccounts:\n- address: '0x0000000000000000000000000000000000000000000000000000000000001234'\n  gas_amounts: [5200000000000000]\n";
fn private(path: &Path, bytes: &[u8]) -> Result<Pin> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(Pin {
        path: path.into(),
        sha256: runtime_inspection::hash(bytes),
    })
}
fn directory(path: &Path) -> Result<()> {
    fs::create_dir(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    ensure!(
        matches!(args.len(), 1 | 2),
        "explicit empty private fixture directory and optional validator count required"
    );
    let validators: usize = args
        .get(1)
        .map(|v| -> Result<usize> {
            v.to_str()
                .ok_or_else(|| anyhow::anyhow!("validator count required"))?
                .parse::<usize>()
                .map_err(anyhow::Error::from)
        })
        .transpose()?
        .unwrap_or(1);
    ensure!(
        (1..=8).contains(&validators),
        "fixture validator count must be 1..8"
    );
    let root = PathBuf::from(&args[0]);
    ensure!(
        root.is_absolute()
            && root.canonicalize()? == root
            && fs::metadata(&root)?.permissions().mode() & 0o077 == 0
            && fs::read_dir(&root)?.next().is_none(),
        "empty canonical private fixture root required"
    );
    let network = ConfigBuilder::new(&root)
        .committee_size(NonZeroUsize::new(validators).unwrap())
        .with_genesis_config(serde_yaml::from_str::<GenesisConfig>(YAML)?)
        .rng(StdRng::from_seed([3; 32]))
        .build();
    let fullnode = FullnodeConfigBuilder::new()
        .with_config_directory(root.clone())
        .build(&mut StdRng::from_seed([4; 32]), &network);
    let blob = network.genesis.to_bytes();
    let chain = rtd_swarm_config::genesis_inspection::verify_blob(YAML.as_bytes(), &blob)?.chain_id;
    let input = private(&root.join("genesis-input.yaml"), YAML.as_bytes())?;
    let genesis = private(&root.join("genesis.blob"), &blob)?;
    let mut result =
        json!({"version":1,"root":root,"chainId":chain,"testKeysOnly":true,"started":false});
    let mut nodes: Vec<_> = network
        .validator_configs
        .iter()
        .enumerate()
        .map(|(index, node)| {
            (
                Role::Validator,
                node.clone(),
                if validators == 1 {
                    "validator".into()
                } else {
                    format!("validator-{index}")
                },
            )
        })
        .collect();
    nodes.push((Role::Fullnode, fullnode, "fullnode".into()));
    let mut inventory = Vec::new();
    for (role, mut node, name) in nodes {
        node.genesis = Genesis::new_from_file(genesis.path.clone());
        node.db_path = root.join(format!("{name}-db"));
        directory(&node.db_path)?;
        if let Some(c) = &mut node.consensus_config {
            c.db_path = root.join(format!("{name}-consensus"));
            directory(&c.db_path)?;
        }
        if role == Role::Fullnode {
            let path = root.join("checkpoints");
            directory(&path)?;
            node.checkpoint_executor_config.data_ingestion_dir = Some(path);
        }
        let config = private(
            &root.join(format!("{name}.yaml")),
            &serde_yaml::to_vec(&node)?,
        )?;
        let request = Request {
            version: 1,
            role,
            environment_directory: root.clone(),
            chain_id: chain.clone(),
            configuration: config,
            genesis_input: input.clone(),
            genesis_blob: genesis.clone(),
            extra_inputs: vec![],
            extra_state_directories: vec![],
        };
        let request = private(
            &root.join(format!("{name}-request.json")),
            &serde_json::to_vec(&request)?,
        )?;
        let report = runtime_inspection::inspect(&request.path, &request.sha256)?;
        let record = json!({"request":request,"rpcAddress":report.rpc_address,"protocolPublicKey":report.protocol_public_key,"genesisSha256":report.genesis_sha256});
        result[&name] = record.clone();
        inventory.push(json!({"name":name,"role":role,"details":record}));
    }
    result["nodes"] = json!(inventory);
    private(&root.join("fixture.json"), &serde_json::to_vec(&result)?)?;
    println!(
        "{}",
        json!({"fixture":root.join("fixture.json"),"testKeysOnly":true,"started":false})
    );
    Ok(())
}
