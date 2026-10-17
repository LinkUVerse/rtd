// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use clap::Subcommand;
use rtd_indexer_alt_framework::ingestion::ClientArgs;
use rtd_indexer_alt_framework::postgres::DbArgs;
use rtd_indexer_alt_metrics::MetricsArgs;
use url::Url;

use crate::IndexerArgs;
#[cfg(feature = "benchmark")]
use crate::benchmark::BenchmarkArgs;

#[derive(clap::Parser, Debug, Clone)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Clone, Debug)]
pub enum Command {
    /// Explicitly prepare, plan, supply or verify one fixed Alt database identity.
    PostgresIdentity {
        #[arg(long)]
        profile: String,
        #[arg(long)]
        action: String,
        #[arg(long)]
        config: PathBuf,
        #[arg(long)]
        expected_sha256: String,
        #[arg(long)]
        state_directory: PathBuf,
        #[arg(long)]
        identity_sha256: Option<String>,
        #[arg(long)]
        expected_plan_sha256: Option<String>,
        /// Test-only numeric-loopback plaintext; omitted by deployment scripts.
        #[arg(long)]
        allow_loopback_development: bool,
    },
    /// Run the indexer.
    Indexer {
        /// The URL of the database to connect to.
        #[clap(
            long,
            default_value = "postgres://postgres:postgrespw@localhost:5432/rtd_indexer_alt"
        )]
        database_url: Url,

        /// Read a database URL from an owned, owner-only regular file instead of process arguments.
        #[arg(long, conflicts_with = "database_url")]
        database_url_file: Option<PathBuf>,

        #[command(flatten)]
        db_args: DbArgs,

        #[command(flatten)]
        client_args: ClientArgs,

        #[command(flatten)]
        indexer_args: IndexerArgs,

        #[command(flatten)]
        metrics_args: MetricsArgs,

        /// Path to the indexer's configuration TOML file.
        #[arg(long)]
        config: PathBuf,

        /// Refuse another Alt writer in the same PostgreSQL database and exit if the lock session dies.
        #[arg(long, default_value_t = false)]
        require_exclusive_writer: bool,
    },

    /// Inspect installed SQL state, or explicitly initialize/upgrade it with --apply.
    InitializeSchema {
        /// Private file containing the administrative database URL.
        #[arg(long)]
        database_url_file: PathBuf,
        #[command(flatten)]
        db_args: DbArgs,
        /// Run embedded migrations and install writer guards; never called by indexer startup.
        #[arg(long)]
        apply: bool,
        /// Explicitly revoke PUBLIC database CREATE/TEMP and public schema CREATE.
        #[arg(long, requires = "apply")]
        restrict_runtime_access: bool,
    },

    /// Output the contents of the default configuration to STDOUT.
    GenerateConfig,

    /// Combine the configuration held across multiple files into one and output it to STDOUT. When
    /// two configurations set the same field, the last write wins.
    MergeConfigs {
        /// Path to a TOML file to be merged
        #[arg(long, required = true, action = clap::ArgAction::Append)]
        config: Vec<PathBuf>,
    },

    /// Wipe the database of its contents
    ResetDatabase {
        /// The URL of the database to connect to.
        #[clap(
            long,
            default_value = "postgres://postgres:postgrespw@localhost:5432/rtd_indexer_alt"
        )]
        database_url: Url,

        #[command(flatten)]
        db_args: DbArgs,

        /// If true, only drop all tables but do not run the migrations.
        /// That is, no tables will exist in the DB after the reset.
        #[clap(long, default_value_t = false)]
        skip_migrations: bool,
    },

    /// Run the benchmark. It will load ingestion data from the given path and run the pipelines.
    /// The first and last checkpoint will be determined automatically based on the ingestion data.
    /// Note that the indexer will not be bootstrapped from genesis, and hence will
    /// skip any pipelines that rely on genesis data.
    #[cfg(feature = "benchmark")]
    Benchmark {
        /// The URL of the database to connect to.
        #[clap(
            long,
            default_value = "postgres://postgres:postgrespw@localhost:5432/rtd_indexer_alt"
        )]
        database_url: Url,

        #[command(flatten)]
        db_args: DbArgs,

        #[command(flatten)]
        benchmark_args: BenchmarkArgs,

        /// Path to the indexer's configuration TOML file.
        #[arg(long)]
        config: PathBuf,
    },
}
