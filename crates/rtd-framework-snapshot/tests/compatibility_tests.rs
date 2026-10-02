// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

mod compatibility_tests {
    use move_package_alt::PackageLoader;
    use rtd_framework::{BuiltInFramework, compare_system_package};
    use rtd_framework_snapshot::{
        FIRST_RTD_SNAPSHOT_PROTOCOL_VERSION, load_bytecode_snapshot,
        load_bytecode_snapshot_manifest,
    };
    use rtd_package_alt::{RtdFlavor, local_test_environment};
    use rtd_protocol_config::{Chain, ProtocolConfig, ProtocolVersion};
    use std::collections::BTreeMap;
    use std::path::Path;

    /// The number of bytecode snapshots to backtest against the current framework.
    /// This should be set to a reasonable number to ensure that we are testing against any
    /// possible running version of the framework, but not too large to avoid weird config
    /// artifacts.
    const SNAPSHOT_BACKTEST_WINDOW: usize = 10;

    #[tokio::test]
    async fn test_framework_compatibility() {
        // Only snapshots created for this chain can be compared with the current RTD framework.
        let manifest = load_bytecode_snapshot_manifest();
        assert!(
            manifest.contains_key(&ProtocolVersion::MAX.as_u64()),
            "Generate the current RTD framework snapshot before running compatibility tests"
        );
        for (version, _snapshots) in manifest
            .iter()
            .rev()
            .filter(|(version, _)| **version >= FIRST_RTD_SNAPSHOT_PROTOCOL_VERSION)
            .take(SNAPSHOT_BACKTEST_WINDOW)
        {
            let config =
                ProtocolConfig::get_for_version(ProtocolVersion::new(*version), Chain::Unknown);
            let binary_config = config.binary_config(None);
            let framework = load_bytecode_snapshot(*version).unwrap();
            let old_framework_store: BTreeMap<_, _> = framework
                .into_iter()
                .map(|package| (package.id, package.genesis_object()))
                .collect();
            for cur_package in BuiltInFramework::iter_system_packages() {
                if compare_system_package(
                    &old_framework_store,
                    &cur_package.id,
                    &cur_package.modules(),
                    cur_package.dependencies.to_vec(),
                    &binary_config,
                )
                .await
                .is_none()
                {
                    panic!(
                        "The current Rtd framework {:?} is not compatible with version {:?}",
                        cur_package.id, version
                    );
                }
            }
        }
    }

    #[test]
    fn check_framework_change_with_protocol_upgrade() {
        // A new RTD genesis must have bytecode for its exact protocol version.
        let snapshots = load_bytecode_snapshot_manifest();
        let version = ProtocolVersion::MAX.as_u64();
        assert!(
            snapshots.contains_key(&version),
            "Generate the current RTD framework snapshot before creating genesis"
        );
        let latest_snapshot = load_bytecode_snapshot(version).unwrap();
        // Turn them into BTreeMap for deterministic comparison.
        let latest_snapshot_ref: BTreeMap<_, _> =
            latest_snapshot.iter().map(|p| (&p.id, p)).collect();
        let current_framework: BTreeMap<_, _> = BuiltInFramework::iter_system_packages()
            .map(|p| (&p.id, p))
            .collect();
        assert_eq!(
            latest_snapshot_ref, current_framework,
            "The current framework differs the latest bytecode snapshot. Did you forget to upgrade protocol version?"
        );
    }

    /// This test checks that the `SinglePackage` entries in `manifest.json` match the metadata
    /// in the `Move.toml` files in the repo.
    ///
    /// Note that this test currently assumes that no framework packages will be removed or moved
    /// within the repo; we check the historical metadata against the current repository. If
    /// needed, we could be more precise by first checking out the revision of the package listed
    /// in the manifest (this should actually be fairly cheap since the git history is present).
    #[tokio::test]
    async fn check_manifest_against_tomls() {
        let manifest = load_bytecode_snapshot_manifest();
        assert!(manifest.contains_key(&ProtocolVersion::MAX.as_u64()));
        for entry in manifest
            .range(FIRST_RTD_SNAPSHOT_PROTOCOL_VERSION..)
            .map(|(_, entry)| entry)
        {
            for package in entry.packages.iter() {
                // parse package.path/Move.toml
                let package_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../..")
                    .join(&package.path);

                let root_pkg = PackageLoader::new(
                    &package_path,
                    local_test_environment(),
                    RtdFlavor::for_testing(),
                )
                .load()
                .await
                .expect("can load system packages");

                assert_eq!(root_pkg.package_info().display_name(), package.name);
                assert_eq!(
                    root_pkg
                        .package_info()
                        .published()
                        .expect("system packages are published")
                        .published_at
                        .0,
                    *package.id
                );
            }
        }
    }

    #[test]
    fn check_no_dirty_manifest_commit() {
        let snapshots = load_bytecode_snapshot_manifest();
        for snapshot in snapshots
            .range(FIRST_RTD_SNAPSHOT_PROTOCOL_VERSION..)
            .map(|(_, snapshot)| snapshot)
        {
            assert!(
                !snapshot.git_revision.contains("dirty"),
                "If you are trying to regenerate the bytecode snapshot after cherry-picking, please do so in a standalone PR after the cherry-pick is merged on the release branch.",
            );
        }
    }

    #[test]
    fn legacy_upstream_snapshots_are_rejected_even_before_cleanup() {
        let error = load_bytecode_snapshot(FIRST_RTD_SNAPSHOT_PROTOCOL_VERSION - 1).unwrap_err();
        assert!(error.to_string().contains("predates RTD genesis"));
    }
}
