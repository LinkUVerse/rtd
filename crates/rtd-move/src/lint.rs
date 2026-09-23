// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_cli::base::lint;
use move_compiler::editions::Flavor;
use move_package_alt_compilation::build_config::BuildConfig;
use std::path::Path;
use rtd_package_alt::RtdFlavor;

#[derive(Parser)]
#[group(id = "rtd-move-lint")]
pub struct Lint {
    #[clap(flatten)]
    pub lint: lint::Lint,
}

impl Lint {
    pub async fn execute(
        self,
        path: Option<&Path>,
        mut build_config: BuildConfig,
        flavor: RtdFlavor,
    ) -> anyhow::Result<()> {
        // Force the Rtd compiler flavor (as `build` and `test` do) so that the Rtd-specific
        // linters are registered. Without this, `rtd move lint` runs only the generic Move
        // linters and silently skips the Rtd object-model lints.
        if build_config.default_flavor.is_none() {
            build_config.default_flavor = Some(Flavor::Rtd);
        }
        self.lint.execute(path, build_config, flavor).await
    }
}
