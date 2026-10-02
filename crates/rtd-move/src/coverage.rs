// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_cli::base::coverage;
use move_package_alt_compilation::build_config::BuildConfig;
use rtd_package_alt::RtdFlavor;
use std::path::Path;

#[derive(Parser)]
#[group(id = "rtd-move-coverage")]
pub struct Coverage {
    #[clap(flatten)]
    pub coverage: coverage::Coverage,
}

impl Coverage {
    pub async fn execute(
        self,
        path: Option<&Path>,
        build_config: BuildConfig,
        flavor: RtdFlavor,
    ) -> anyhow::Result<()> {
        self.coverage.execute(path, build_config, flavor).await?;
        Ok(())
    }
}
