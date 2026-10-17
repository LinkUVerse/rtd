// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use rtd_indexer_alt_metrics::MetricsArgs;
use rtd_indexer_alt_reader::consistent_reader::ConsistentReaderArgs;
use rtd_indexer_alt_reader::fullnode_client::FullnodeArgs;
use rtd_indexer_alt_reader::kv_loader::KvArgs;
use rtd_indexer_alt_reader::pg_reader::db::DbArgs;
use rtd_indexer_alt_reader::system_package_task::SystemPackageTaskArgs;
use tonic::transport::Uri;
use url::Url;

use crate::RpcArgs;

/// Arguments for configuring GraphQL streaming subscriptions.
#[derive(clap::Args, Debug, Clone, Default)]
pub struct SubscriptionArgs {
    /// gRPC URL of a configured fullnode to stream checkpoints from. When set, the instance enables GraphQL
    /// subscriptions. When not set, subscriptions are not available.
    #[arg(long)]
    pub checkpoint_stream_url: Option<Uri>,
    /// Fixed publication authority endpoint. HTTPS requires a private CA and client identity.
    #[arg(long, requires = "checkpoint_stream_url")]
    pub publication_guard_url: Option<Url>,
    #[arg(long, requires = "publication_guard_url")]
    pub publication_guard_ca: Option<PathBuf>,
    /// PEM containing the client certificate and private key.
    #[arg(long, requires = "publication_guard_url")]
    pub publication_guard_identity: Option<PathBuf>,
}

#[derive(clap::Parser, Debug, Clone)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[allow(clippy::large_enum_variant)]
#[derive(clap::Subcommand, Debug, Clone)]
pub enum Command {
    /// Run the RPC service.
    Rpc {
        /// The URL of the database to connect to.
        #[clap(
            long,
            default_value = "postgres://postgres:postgrespw@localhost:5432/rtd_indexer_alt"
        )]
        database_url: Url,

        /// Read the database URL from a mounted secret instead of process arguments.
        #[arg(long, conflicts_with = "database_url")]
        database_url_file: Option<PathBuf>,

        #[command(flatten)]
        fullnode_args: FullnodeArgs,

        #[command(flatten)]
        db_args: DbArgs,

        #[command(flatten)]
        kv_args: KvArgs,

        #[command(flatten)]
        consistent_reader_args: ConsistentReaderArgs,

        #[command(flatten)]
        rpc_args: RpcArgs,

        #[command(flatten)]
        system_package_task_args: SystemPackageTaskArgs,

        #[command(flatten)]
        metrics_args: MetricsArgs,

        /// Path to the RPC's configuration TOML file. If one is not provided, the default values for
        /// the configuration will be set.
        #[arg(long)]
        config: Option<PathBuf>,

        /// Path to indexer configuration TOML files (multiple can be supplied). These are used to
        /// identify the pipelines that the RPC will monitor for watermark purposes.
        #[arg(long, action = clap::ArgAction::Append)]
        indexer_config: Vec<PathBuf>,

        #[command(flatten)]
        subscription_args: SubscriptionArgs,
    },

    /// Output the contents of the default configuration to STDOUT.
    GenerateConfig,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::Args;
    use super::Command;

    #[tokio::test]
    async fn database_secret_overrides_default_but_conflicts_with_explicit_url() {
        let args = Args::try_parse_from([
            "graphql",
            "rpc",
            "--database-url-file",
            "/private/database-url",
        ])
        .unwrap();
        let Command::Rpc {
            database_url_file, ..
        } = args.command
        else {
            panic!("expected rpc command")
        };
        assert_eq!(
            database_url_file.unwrap().to_str(),
            Some("/private/database-url")
        );
        let error = Args::try_parse_from([
            "graphql",
            "rpc",
            "--database-url-file",
            "/private/database-url",
            "--database-url",
            "postgres://localhost/indexer",
        ])
        .unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
    }
}
