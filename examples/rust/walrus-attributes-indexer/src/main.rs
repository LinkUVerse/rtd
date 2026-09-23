// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::bail;
use clap::Parser;
use rtd_indexer_alt_framework::{
    cluster::{self, IndexerClusterBuilder},
    pipeline::sequential::SequentialConfig,
    service::Error,
    Result,
};
use url::Url;
use walrus_attributes_indexer::{handlers::BlogPostPipeline, MIGRATIONS};

#[derive(clap::Parser, Debug)]
struct WalrusIndexerArgs {
    /// StructTag of Metadata in a Walrus deployment on this RTD network.
    #[clap(long, env = "RTD_WALRUS_METADATA_TYPE")]
    metadata_dynamic_field_type: String,

    #[clap(
        long,
        default_value = "postgres://postgres:postgrespw@localhost:5432/walrus_attributes"
    )]
    database_url: Url,

    #[clap(flatten)]
    cluster_args: cluster::Args,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = WalrusIndexerArgs::parse();

    // The `IndexerClusterBuilder` offers a convenient way to quickly set up an `IndexerCluster`,
    // which consists of the base indexer, metrics service, and a cancellation token.
    let mut indexer = IndexerClusterBuilder::new()
        .with_database_url(args.database_url)
        .with_args(args.cluster_args)
        .with_migrations(&MIGRATIONS)
        .build()
        .await?;

    let blog_post_pipeline = BlogPostPipeline::new(&args.metadata_dynamic_field_type)?;

    // Other pipelines can be easily added with `.sequential_pipeline()` or
    // `.concurrent_pipeline()`.
    indexer
        .sequential_pipeline(blog_post_pipeline, SequentialConfig::default())
        .await?;

    match indexer.run().await?.main().await {
        Ok(()) | Err(Error::Terminated) => Ok(()),
        Err(Error::Aborted) => {
            bail!("Indexer aborted due to an unexpected error")
        }
        Err(Error::Task(e)) => {
            bail!(e)
        }
    }
}
