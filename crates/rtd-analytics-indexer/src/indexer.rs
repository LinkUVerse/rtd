// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Analytics indexer builder.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use object_store::ClientOptions;
use object_store::aws::AmazonS3Builder;
use object_store::local::LocalFileSystem;
use rtd_config::object_storage_config::validate_self_hosted_endpoint;
use tracing::info;

use rtd_indexer_alt_framework::Indexer;
use rtd_indexer_alt_framework::IndexerArgs;
use rtd_indexer_alt_framework::ingestion::{ClientArgs, IngestionConfig};
use rtd_indexer_alt_framework::pipeline::CommitterConfig;
use rtd_indexer_alt_framework::pipeline::sequential::SequentialConfig;
use rtd_indexer_alt_framework::service::Service;

use rtd_rpc_resolver::package_store::RpcPackageStore;

use crate::config::IndexerConfig;
use crate::config::OutputStoreConfig;
use crate::handlers::system_package_eviction::SYSTEM_PACKAGE_EVICTION_PIPELINE;
use crate::handlers::system_package_eviction::SystemPackageEviction;
use crate::metrics::Metrics;
use crate::store::AnalyticsStore;

/// Build and run an analytics indexer, returning a Service handle.
///
/// The returned Service integrates store shutdown - when the service shuts down
/// gracefully, it will wait for all pending uploads to complete.
pub async fn build_analytics_indexer(
    config: IndexerConfig,
    indexer_args: IndexerArgs,
    client_args: ClientArgs,
    metrics: Metrics,
    registry: prometheus::Registry,
) -> Result<Service> {
    // Validate config (checks for duplicate pipelines, batch_size requirements, etc.)
    config.validate()?;

    let object_store = create_object_store(&config.output_store)?;
    let store = AnalyticsStore::new(object_store.clone(), config.clone(), metrics.clone());

    // Find checkpoint range (snaps to file boundaries in migration mode)
    let (adjusted_first_checkpoint, adjusted_last_checkpoint) = store
        .find_checkpoint_range(indexer_args.first_checkpoint, indexer_args.last_checkpoint)
        .await?;

    let rpc_url = client_args
        .ingestion
        .rpc_api_url
        .as_ref()
        .map(|u| u.to_string())
        .unwrap_or_default();
    let package_cache = Arc::new(RpcPackageStore::new(&rpc_url).with_cache());

    let mut pipeline_filter = indexer_args.pipeline;
    if !pipeline_filter.is_empty() {
        pipeline_filter.push(SYSTEM_PACKAGE_EVICTION_PIPELINE.to_string());
    }

    let adjusted_indexer_args = IndexerArgs {
        first_checkpoint: adjusted_first_checkpoint,
        last_checkpoint: adjusted_last_checkpoint,
        pipeline: pipeline_filter,
        task: indexer_args.task,
    };

    let ingestion_config = config.ingestion.clone().finish(IngestionConfig::default());

    let mut indexer = Indexer::new(
        store.clone(),
        adjusted_indexer_args,
        client_args,
        ingestion_config,
        None,
        &registry,
    )
    .await?;

    let base_committer = config.committer.clone().finish(CommitterConfig::default());
    let base_sequential = SequentialConfig {
        committer: base_committer,
        ..Default::default()
    };

    for pipeline_config in config.pipeline_configs() {
        info!("Registering pipeline: {}", pipeline_config.pipeline);
        let sequential = pipeline_config
            .sequential
            .clone()
            .finish(base_sequential.clone());
        pipeline_config
            .pipeline
            .register(
                &mut indexer,
                pipeline_config,
                package_cache.clone(),
                &rpc_url,
                metrics.clone(),
                sequential,
            )
            .await?;
    }

    indexer
        .sequential_pipeline(
            SystemPackageEviction::new(package_cache.clone()),
            base_sequential.clone(),
        )
        .await?;

    // Run the indexer and register shutdown signals
    let service = indexer.run().await?;
    Ok(service.with_shutdown_signal(async move {
        store.shutdown().await;
    }))
}

fn create_object_store(config: &OutputStoreConfig) -> Result<Arc<dyn object_store::ObjectStore>> {
    match config {
        OutputStoreConfig::Gcs { .. } | OutputStoreConfig::Azure { .. } => Err(anyhow::anyhow!(
            "RTD analytics only permits local or self-hosted S3 storage"
        )),
        OutputStoreConfig::S3 {
            bucket,
            region,
            access_key_id,
            secret_access_key,
            endpoint,
            request_timeout_secs,
        } => {
            let client_options =
                ClientOptions::default().with_timeout(Duration::from_secs(*request_timeout_secs));
            let endpoint = endpoint
                .as_deref()
                .context("RTD analytics requires a self-hosted S3 endpoint")?;
            validate_self_hosted_endpoint(endpoint)?;
            let access_key = access_key_id
                .as_deref()
                .filter(|value| !value.is_empty())
                .context("RTD analytics requires an explicit self-hosted S3 access key")?;
            let secret_key = secret_access_key
                .as_deref()
                .filter(|value| !value.is_empty())
                .context("RTD analytics requires an explicit self-hosted S3 secret")?;
            anyhow::ensure!(
                !region.is_empty(),
                "RTD analytics requires a self-hosted S3 region"
            );
            let builder = AmazonS3Builder::new()
                .with_client_options(client_options)
                .with_bucket_name(bucket)
                .with_region(region)
                .with_access_key_id(access_key)
                .with_secret_access_key(secret_key)
                .with_endpoint(endpoint);
            builder
                .build()
                .map(|s| Arc::new(s) as Arc<dyn object_store::ObjectStore>)
                .context("Failed to create S3 store")
        }
        OutputStoreConfig::File { path } => LocalFileSystem::new_with_prefix(path)
            .map(|s| Arc::new(s) as Arc<dyn object_store::ObjectStore>)
            .context("Failed to create file store"),
        OutputStoreConfig::Custom(store) => Ok(store.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::create_object_store;
    use crate::config::OutputStoreConfig;
    use std::path::PathBuf;

    #[test]
    fn analytics_rejects_cloud_store_and_implicit_s3() {
        let gcs = OutputStoreConfig::Gcs {
            bucket: "unused".into(),
            service_account_path: PathBuf::from("/unused"),
            custom_headers: None,
            request_timeout_secs: 1,
        };
        assert!(create_object_store(&gcs).is_err());

        let s3 = OutputStoreConfig::S3 {
            bucket: "unused".into(),
            region: "us-east-1".into(),
            access_key_id: Some("unused".into()),
            secret_access_key: Some("unused".into()),
            endpoint: None,
            request_timeout_secs: 1,
        };
        assert!(create_object_store(&s3).is_err());
    }
}
