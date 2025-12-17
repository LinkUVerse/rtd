// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Configuration types for the analytics indexer.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use sui_indexer_alt_framework::ingestion::IngestionConfig;
use sui_indexer_alt_framework::pipeline::sequential::SequentialConfig;

use crate::pipeline::Pipeline;

fn default_client_metric_host() -> String {
    "127.0.0.1".to_string()
}

fn default_client_metric_port() -> u16 {
    8081
}

fn default_remote_store_url() -> String {
    "https://checkpoints.mainnet.sui.io".to_string()
}

fn default_file_format() -> FileFormat {
    FileFormat::Parquet
}

fn default_request_timeout_secs() -> u64 {
    30
}

fn default_upload_channel_capacity() -> usize {
    10
}

fn default_watermark_update_interval_secs() -> u64 {
    60
}

/// Output file format for analytics data.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileFormat {
    Csv,
    Parquet,
}

/// Object store configuration for analytics output.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum OutputStoreConfig {
    Gcs {
        bucket: String,
        /// Path to service account JSON file
        service_account_path: PathBuf,
        #[serde(default = "default_request_timeout_secs")]
        request_timeout_secs: u64,
    },
    S3 {
        bucket: String,
        region: String,
        access_key_id: Option<String>,
        secret_access_key: Option<String>,
        endpoint: Option<String>,
        #[serde(default = "default_request_timeout_secs")]
        request_timeout_secs: u64,
    },
    Azure {
        container: String,
        account: String,
        access_key: String,
        #[serde(default = "default_request_timeout_secs")]
        request_timeout_secs: u64,
    },
    File {
        path: PathBuf,
    },
    /// Custom object store for testing. Allows sharing a store instance across runs.
    #[serde(skip)]
    Custom(std::sync::Arc<dyn object_store::ObjectStore>),
}

/// Main configuration for an analytics indexer job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerConfig {
    /// The url of the checkpoint client to connect to.
    pub rest_url: String,
    /// The url of the metrics client to connect to.
    #[serde(default = "default_client_metric_host")]
    pub client_metric_host: String,
    /// The port of the metrics client to connect to.
    #[serde(default = "default_client_metric_port")]
    pub client_metric_port: u16,
    /// Output object store configuration
    pub output_store: OutputStoreConfig,
    /// Remote store URL.
    #[serde(default = "default_remote_store_url")]
    pub remote_store_url: String,
    /// Optional streaming URL for real-time indexing
    pub streaming_url: Option<String>,
    /// Optional RPC API URL for request/reply from full node
    pub rpc_api_url: Option<String>,
    /// Optional RPC username
    pub rpc_username: Option<String>,
    /// Optional RPC password
    pub rpc_password: Option<String>,
    /// Optional working directory for temporary files (defaults to system temp dir)
    pub work_dir: Option<PathBuf>,
    /// Optional local ingestion path for reading checkpoints from disk instead of remote
    pub local_ingestion_path: Option<PathBuf>,
    pub sf_account_identifier: Option<String>,
    pub sf_warehouse: Option<String>,
    pub sf_database: Option<String>,
    pub sf_schema: Option<String>,
    pub sf_username: Option<String>,
    pub sf_role: Option<String>,
    pub sf_password_file: Option<String>,

    /// Migration mode identifier. When set, the indexer operates in migration mode:
    /// - Overwrites existing files matching target checkpoint ranges
    /// - Uses conditional PUT with etag to prevent concurrent modification
    /// - Uses per-file metadata to track migration progress separately from main pipeline
    #[serde(default)]
    pub migration_id: Option<String>,

    /// File format for output files (csv or parquet).
    #[serde(default = "default_file_format")]
    pub file_format: FileFormat,

    #[serde(rename = "pipelines")]
    pub pipeline_configs: Vec<PipelineConfig>,

    #[serde(default)]
    pub ingestion: IngestionConfig,

    #[serde(default)]
    pub sequential: SequentialConfig,

    pub first_checkpoint: Option<u64>,
    pub last_checkpoint: Option<u64>,

    /// Channel capacity for pending file uploads per pipeline.
    /// When full, commit() blocks (backpressure).
    /// Kept small since uploads are sequential and there may be many pipelines.
    #[serde(default = "default_upload_channel_capacity")]
    pub upload_channel_capacity: usize,

    /// Minimum interval between watermark writes to object store (seconds).
    /// Watermarks are updated after file uploads; this rate-limits those writes.
    /// Default: 60 seconds.
    #[serde(default = "default_watermark_update_interval_secs")]
    pub watermark_update_interval_secs: u64,
}

impl IndexerConfig {
    pub fn pipeline_configs(&self) -> &[PipelineConfig] {
        &self.pipeline_configs
    }
}

/// Batch size configuration for when to write files.
///
/// Exactly one of these must be set - the indexer will panic at startup if both
/// or neither are configured.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchSizeConfig {
    /// Write a file after accumulating this many checkpoints.
    Checkpoints(usize),
    /// Write a file after accumulating this many rows.
    Rows(usize),
}

/// Configuration for a single analytics task/pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Type of data to write i.e. checkpoint, object, transaction, etc
    pub pipeline: Pipeline,
    /// File format to use (csv or parquet)
    #[serde(default = "default_file_format")]
    pub file_format: FileFormat,
    pub package_id_filter: Option<String>,
    /// Snowflake table to monitor
    pub sf_table_id: Option<String>,
    /// Snowflake column containing checkpoint numbers
    pub sf_checkpoint_col_id: Option<String>,
    /// Whether to report max checkpoint from Snowflake table
    #[serde(default)]
    pub report_sf_max_table_checkpoint: bool,
    /// Batch size configuration - determines when to write files.
    /// Required for live mode (when migration_id is None).
    /// Ignored in migration mode (file boundaries come from existing files).
    #[serde(default)]
    pub batch_size: Option<BatchSizeConfig>,
    /// Migration mode identifier. When set, uses file boundaries from existing files.
    #[serde(default)]
    pub migration_id: Option<String>,
    /// Schema column names for parquet serialization.
    /// Populated programmatically from the row type, not from config files.
    #[serde(skip)]
    pub schema: Vec<String>,
}

impl PipelineConfig {
    /// Validate the configuration.
    ///
    /// Returns an error if batch_size is required but not set.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.migration_id.is_none() && self.batch_size.is_none() {
            anyhow::bail!(
                "batch_size is required for pipeline '{}' (not in migration mode)",
                self.pipeline
            );
        }
        Ok(())
    }
}
