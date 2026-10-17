// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{io::Read, os::unix::fs::OpenOptionsExt, path::Path};
fn read(path: &Path, max: u64) -> anyhow::Result<Vec<u8>> {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    anyhow::ensure!(
        file.metadata()?.is_file() && file.metadata()?.len() <= max,
        "bounded regular input required"
    );
    let mut bytes = Vec::new();
    file.take(max + 1).read_to_end(&mut bytes)?;
    anyhow::ensure!(bytes.len() as u64 <= max, "input exceeds budget");
    Ok(bytes)
}
fn run() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() == 3 && args[0] == "--inspect-node" {
        let sha = args[2]
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("runtime digest required"))?;
        let report = rtd_swarm_config::runtime_inspection::inspect(Path::new(&args[1]), sha)?;
        println!("{}", serde_json::to_string(&report)?);
        return Ok(());
    }
    anyhow::ensure!(
        args.len() == 1
            || (args.len() == 3 && (args[1] == "--verify-blob" || args[1] == "--verify-signer"))
            || (args.len() == 5 && args[1] == "--verify-signer" && args[3] == "--deploy-address"),
        "usage: rtd-genesis-inspect GENESIS_YAML [--verify-blob GENESIS_BLOB | --verify-signer CLIENT_YAML [--deploy-address ADDRESS]]"
    );
    let bytes = read(Path::new(&args[0]), 1024 * 1024)?;
    let result = if args.len() == 5 {
        let address = args[4]
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("invalid address"))?
            .parse()?;
        serde_json::to_value(rtd_swarm_config::genesis_signer::verify(
            &bytes,
            Path::new(&args[2]),
            address,
        )?)?
    } else if args.len() == 3 && args[1] == "--verify-signer" {
        serde_json::to_value(rtd_swarm_config::genesis_signer::verify_active(
            &bytes,
            Path::new(&args[2]),
        )?)?
    } else if args.len() == 3 {
        let blob = read(Path::new(&args[2]), 128 * 1024 * 1024)?;
        serde_json::to_value(rtd_swarm_config::genesis_inspection::verify_blob(
            &bytes, &blob,
        )?)?
    } else {
        serde_json::to_value(rtd_swarm_config::genesis_inspection::inspect(&bytes)?)?
    };
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}
fn main() {
    if run().is_err() {
        eprintln!(
            "genesis inspection rejected input; require supported explicit YAML, matching native blob or funded active local signer; input values withheld"
        );
        std::process::exit(1);
    }
}
