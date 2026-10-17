// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{env, fs, path::PathBuf};

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let snapshots = root.join("bytecode_snapshot");
    println!("cargo:rerun-if-changed=manifest.json");
    println!("cargo:rerun-if-changed=bytecode_snapshot");
    let mut versions: Vec<_> = fs::read_dir(&snapshots)
        .unwrap()
        .map(Result::unwrap)
        .collect();
    versions.sort_by_key(|entry| entry.file_name());
    let mut generated = format!(
        "pub const MANIFEST: &[u8] = include_bytes!({:?});\n\
         pub fn packages(version: u64) -> Option<&'static [(&'static str, &'static [u8])]> {{\nmatch version {{\n",
        root.join("manifest.json")
    );
    for version in versions {
        assert!(
            version.file_type().unwrap().is_dir(),
            "snapshot versions must be regular directories"
        );
        let name = version.file_name().into_string().unwrap();
        let number: u64 = name.parse().expect("snapshot version must be numeric");
        assert_eq!(
            name,
            number.to_string(),
            "snapshot version must be canonical"
        );
        generated.push_str(&format!("{number} => Some(&[\n"));
        let mut packages: Vec<_> = fs::read_dir(version.path())
            .unwrap()
            .map(Result::unwrap)
            .collect();
        packages.sort_by_key(|entry| entry.file_name());
        for package in packages {
            assert!(
                package.file_type().unwrap().is_file(),
                "snapshot package must be a regular file"
            );
            generated.push_str(&format!(
                "({:?}, include_bytes!({:?}) as &[u8]),\n",
                package.file_name().into_string().unwrap(),
                package.path()
            ));
        }
        generated.push_str("]),\n");
    }
    generated.push_str("_ => None,\n}\n}\n");
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("embedded_snapshots.rs"),
        generated,
    )
    .unwrap();
}
