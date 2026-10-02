// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use rtd_framework::{SystemPackage, SystemPackageMetadata};
use rtd_protocol_config::ProtocolVersion;
use rtd_types::base_types::ObjectID;
use rtd_types::{
    BRIDGE_PACKAGE_ID, DEEPBOOK_PACKAGE_ID, MOVE_STDLIB_PACKAGE_ID, RTD_FRAMEWORK_PACKAGE_ID,
    RTD_SYSTEM_PACKAGE_ID,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub type SnapshotManifest = BTreeMap<u64, Snapshot>;

/// Earlier snapshots belong to the upstream chain and cannot be used for RTD genesis.
pub const FIRST_RTD_SNAPSHOT_PROTOCOL_VERSION: u64 = 138;

/// Encapsulation of an entry in the manifest file corresponding to a single version of the system
/// packages.
///
// Note: the [Snapshot] and [SnapshotPackage] types are similar to the
// [rtd_framework::{SystemPackageMetadata, SystemPackage}] types,
// and also to the [rtd::framework_versions::{FrameworkVersion, FrameworkPackage}] types.
// They are sort of a stepping stone from one to the other - the [rtd_framework] types contain
// additional information about the compiled bytecode of the package, while the
// [framework_versions] types do not contain information about the object IDs of the packages.
//
// These types serve as a kind of stepping stone; they are constructed from the [rtd_framework]
// types and serialized in the manifest, and then the build script for the [rtd] crate reads them
// from the manifest file and encodes them in the `rtd` binary. A little information is dropped in
// each of these steps.
#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    /// Git revision that this snapshot is taken on.
    pub git_revision: String,

    /// List of system packages in this version
    pub packages: Vec<SnapshotPackage>,
}

/// Entry in the manifest file corresponding to a specific version of a specific system package
#[derive(Serialize, Deserialize)]
pub struct SnapshotPackage {
    /// Name of the package (e.g. "MoveStdLib")
    pub name: String,
    /// Path to the package in the monorepo (e.g. "crates/rtd-framework/packages/move-stdlib")
    pub path: String,
    /// Object ID of the published package
    pub id: ObjectID,
}

impl Snapshot {
    pub fn package_ids(&self) -> impl Iterator<Item = ObjectID> + '_ {
        self.packages.iter().map(|p| p.id)
    }
}

impl SnapshotPackage {
    pub fn from_system_package_metadata(value: &SystemPackageMetadata) -> Self {
        Self {
            name: value.name.clone(),
            path: value.path.clone(),
            id: value.compiled.id,
        }
    }
}

const SYSTEM_PACKAGE_PUBLISH_ORDER: &[ObjectID] = &[
    MOVE_STDLIB_PACKAGE_ID,
    RTD_FRAMEWORK_PACKAGE_ID,
    RTD_SYSTEM_PACKAGE_ID,
    DEEPBOOK_PACKAGE_ID,
    BRIDGE_PACKAGE_ID,
];

pub fn load_bytecode_snapshot_manifest() -> SnapshotManifest {
    let Ok(bytes) = fs::read(manifest_path()) else {
        return SnapshotManifest::default();
    };
    serde_json::from_slice::<SnapshotManifest>(&bytes)
        .expect("Could not deserialize SnapshotManifest")
}

pub fn update_bytecode_snapshot_manifest(
    git_revision: &str,
    version: u64,
    files: Vec<SnapshotPackage>,
) {
    let mut snapshot = load_bytecode_snapshot_manifest();

    snapshot.insert(
        version,
        Snapshot {
            git_revision: git_revision.to_string(),
            packages: files,
        },
    );

    let json =
        serde_json::to_string_pretty(&snapshot).expect("Could not serialize SnapshotManifest");
    fs::write(manifest_path(), json).expect("Could not update manifest file");
}

pub fn load_bytecode_snapshot(protocol_version: u64) -> anyhow::Result<Vec<SystemPackage>> {
    let snapshot_path = snapshot_path_for_version(protocol_version)?;
    let manifest = load_bytecode_snapshot_manifest();
    let entry = manifest.get(&protocol_version).ok_or_else(|| {
        anyhow::anyhow!(
            "No RTD snapshot manifest entry found for protocol version {protocol_version}"
        )
    })?;
    let expected_ids: BTreeSet<_> = entry.package_ids().collect();
    anyhow::ensure!(
        !expected_ids.is_empty() && expected_ids.len() == entry.packages.len(),
        "RTD snapshot manifest for protocol version {protocol_version} has no packages or duplicate package IDs"
    );

    let mut snapshots = BTreeMap::<ObjectID, SystemPackage>::new();
    for directory_entry in fs::read_dir(&snapshot_path)? {
        let directory_entry = directory_entry?;
        let path = directory_entry.path();
        anyhow::ensure!(
            directory_entry.file_type()?.is_file(),
            "Unexpected non-file entry in RTD snapshot: {}",
            path.display()
        );
        let bytes = fs::read(&path)
            .with_context(|| format!("Cannot read RTD snapshot package {}", path.display()))?;
        let package: SystemPackage = bcs::from_bytes(&bytes)
            .with_context(|| format!("Cannot decode RTD snapshot package {}", path.display()))?;
        let expected_name = package.id.to_string();
        anyhow::ensure!(
            path.file_name().and_then(|name| name.to_str()) == Some(expected_name.as_str()),
            "RTD snapshot package {} has the wrong object ID filename; expected {expected_name}",
            path.display()
        );
        anyhow::ensure!(
            snapshots.insert(package.id, package).is_none(),
            "RTD snapshot for protocol version {protocol_version} contains duplicate package IDs"
        );
    }
    let loaded_ids: BTreeSet<_> = snapshots.keys().copied().collect();
    anyhow::ensure!(
        loaded_ids == expected_ids,
        "RTD snapshot files for protocol version {protocol_version} do not match manifest package IDs"
    );
    if protocol_version == FIRST_RTD_SNAPSHOT_PROTOCOL_VERSION {
        let required_ids: BTreeSet<_> = SYSTEM_PACKAGE_PUBLISH_ORDER.iter().copied().collect();
        anyhow::ensure!(
            loaded_ids == required_ids,
            "RTD genesis snapshot for protocol version {protocol_version} must contain every system package"
        );
    }

    // system packages need to be restored in a specific order
    let mut snapshot_objects = Vec::new();
    for package_id in SYSTEM_PACKAGE_PUBLISH_ORDER {
        if let Some(object) = snapshots.remove(package_id) {
            snapshot_objects.push(object);
        }
    }
    anyhow::ensure!(
        snapshots.is_empty(),
        "RTD snapshot for protocol version {protocol_version} contains an unsupported system package"
    );
    Ok(snapshot_objects)
}

pub fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("manifest.json")
}

fn snapshot_path_for_version(version: u64) -> anyhow::Result<PathBuf> {
    let snapshot_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bytecode_snapshot");
    snapshot_path_for_version_in(&snapshot_dir, version)
}

fn snapshot_path_for_version_in(snapshot_dir: &Path, version: u64) -> anyhow::Result<PathBuf> {
    anyhow::ensure!(
        version >= FIRST_RTD_SNAPSHOT_PROTOCOL_VERSION,
        "Protocol version {version} predates RTD genesis; upstream chain snapshots cannot be loaded"
    );
    anyhow::ensure!(
        version <= ProtocolVersion::MAX.as_u64(),
        "Protocol version {version} exceeds the current RTD maximum {}",
        ProtocolVersion::MAX.as_u64()
    );
    let snapshot_path = snapshot_dir.join(version.to_string());
    anyhow::ensure!(
        snapshot_path.is_dir(),
        "No RTD snapshot found for protocol version {version} at {}",
        snapshot_path.display()
    );
    Ok(snapshot_path)
}

#[cfg(test)]
mod tests {
    use super::{FIRST_RTD_SNAPSHOT_PROTOCOL_VERSION, snapshot_path_for_version_in};
    use rtd_protocol_config::ProtocolVersion;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

    struct TestSnapshotDir(PathBuf);

    impl TestSnapshotDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "rtd-snapshot-policy-{}-{}",
                std::process::id(),
                NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestSnapshotDir {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn rejects_upstream_snapshot_even_if_its_directory_exists() {
        let root = TestSnapshotDir::new();
        let old_version = FIRST_RTD_SNAPSHOT_PROTOCOL_VERSION - 1;
        std::fs::create_dir(root.0.join(old_version.to_string())).unwrap();

        let error = snapshot_path_for_version_in(&root.0, old_version).unwrap_err();
        assert!(error.to_string().contains("predates RTD genesis"));
    }

    #[test]
    fn requires_exact_current_snapshot_without_falling_back_to_upstream() {
        let root = TestSnapshotDir::new();
        let current_version = ProtocolVersion::MAX.as_u64();
        std::fs::create_dir(root.0.join((current_version - 1).to_string())).unwrap();

        let error = snapshot_path_for_version_in(&root.0, current_version).unwrap_err();
        assert!(error.to_string().contains("No RTD snapshot found"));

        let current_path = root.0.join(current_version.to_string());
        std::fs::create_dir(&current_path).unwrap();
        assert_eq!(
            snapshot_path_for_version_in(&root.0, current_version).unwrap(),
            current_path
        );
    }
}
