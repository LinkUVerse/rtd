// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Analytics store implementation with TransactionalStore support.
//!
//! This store supports two modes:
//!
//! ## Live Mode
//! Derives watermarks from file names via bucket iteration at startup,
//! rather than storing them separately. File uploads inherently update the watermark
//! since file names encode checkpoint ranges.
//!
//! ## Migration Mode
//! When `migration_id` is set, the store operates in migration mode:
//! - Existing file ranges are loaded at startup and updated in-place.
//! - Watermark is stored in a separate file: `_metadata/watermarks/{pipeline}@migration_{id}.json`
//! - Conditional PUT with etag is used to prevent concurrent modification of data files

use std::collections::HashMap;
use std::ops::Range;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::{Result, bail};
use async_trait::async_trait;
use object_store::PutPayload;
use object_store::path::Path as ObjectPath;
use scoped_futures::ScopedBoxFuture;
use sui_indexer_alt_framework::store::{Connection, Store, TransactionalStore};
use sui_indexer_alt_framework_store_traits::{
    CommitterWatermark, PrunerWatermark, ReaderWatermark,
};
use sui_types::base_types::EpochId;
use tokio::sync::mpsc;
use tracing::debug;

use crate::config::FileFormat;
use crate::handlers::CheckpointRows;
use crate::metrics::Metrics;
use uploader::PendingFileUpload;

/// Rows accumulated across commits, waiting to be flushed to a file.
#[derive(Clone, Default)]
pub struct Batch {
    pub(crate) checkpoints_rows: Vec<CheckpointRows>,
    row_count: usize,
}

impl Batch {
    pub(crate) fn first_checkpoint(&self) -> Option<u64> {
        self.checkpoints_rows.first().map(|c| c.checkpoint)
    }

    pub(crate) fn last_checkpoint(&self) -> Option<u64> {
        self.checkpoints_rows.last().map(|c| c.checkpoint)
    }

    pub(crate) fn epoch(&self) -> Option<EpochId> {
        self.checkpoints_rows.last().map(|c| c.epoch)
    }

    pub(crate) fn row_count(&self) -> usize {
        self.row_count
    }

    pub(crate) fn checkpoint_count(&self) -> usize {
        self.checkpoints_rows.len()
    }

    pub(crate) fn checkpoint_range(&self) -> Option<Range<u64>> {
        match (self.first_checkpoint(), self.last_checkpoint()) {
            (Some(first), Some(last)) => Some(first..last + 1),
            _ => None,
        }
    }

    pub(crate) fn add(&mut self, checkpoint_rows: CheckpointRows) {
        self.row_count += checkpoint_rows.len();
        self.checkpoints_rows.push(checkpoint_rows);
    }
}

mod live;
mod migration;
mod uploader;

pub use live::{LiveStore, LiveStoreConfig};
pub use migration::{FileRangeEntry, FileRangeIndex, MigrationStore, WatermarkUpdateError};

/// The operational mode of the analytics store.
#[derive(Clone)]
pub enum StoreMode {
    Live(LiveStore),
    Migration(MigrationStore),
}

use crate::config::PipelineConfig;

/// Analytics store wrapper that delegates to an inner store mode.
#[derive(Clone)]
pub struct AnalyticsStore {
    mode: StoreMode,
    /// Accumulated rows per pipeline, keyed by pipeline name.
    pending_by_pipeline: Arc<RwLock<HashMap<String, Batch>>>,
    /// Per-pipeline configuration (including schema), keyed by pipeline name.
    pipelines: Arc<RwLock<HashMap<String, PipelineConfig>>>,
    /// Shared metrics for all pipelines.
    metrics: Metrics,
    /// Per-pipeline upload senders. Sender is Clone so we can share it.
    uploader_senders: Arc<RwLock<HashMap<String, mpsc::Sender<PendingFileUpload>>>>,
    /// Worker handles for graceful shutdown.
    worker_handles: Arc<tokio::sync::Mutex<Vec<tokio::task::JoinHandle<()>>>>,
    /// Channel capacity for pending file uploads per pipeline.
    upload_channel_capacity: usize,
    /// Minimum interval between watermark writes to object store.
    watermark_update_interval: Duration,
}

/// Connection to the analytics store.
///
/// Provides access to the underlying object store for file uploads.
pub struct AnalyticsConnection<'a> {
    store: &'a AnalyticsStore,
}

impl StoreMode {
    /// Split a batch of checkpoints into files.
    ///
    /// Delegates to mode-specific splitting logic:
    /// - Live: cuts at epoch boundaries and batch size thresholds
    /// - Migration: cuts at existing file boundaries
    pub(crate) fn split_framework_batch_into_files(
        &self,
        pipeline: &str,
        batch_from_framework: &[CheckpointRows],
        pending_batch: Batch,
    ) -> (Batch, Vec<Batch>) {
        match self {
            StoreMode::Live(store) => store.split_framework_batch_into_files(
                pipeline,
                batch_from_framework,
                pending_batch,
            ),
            StoreMode::Migration(store) => store.split_framework_batch_into_files(
                pipeline,
                batch_from_framework,
                pending_batch,
            ),
        }
    }

    /// Write a file to the object store.
    ///
    /// Delegates to mode-specific logic:
    /// - Live mode: simple `put`
    /// - Migration mode: verifies range matches expected, uses conditional PUT with etag/version
    pub(crate) async fn write_to_object_store(
        &self,
        pipeline: &str,
        path: &ObjectPath,
        checkpoint_range: &Range<u64>,
        payload: PutPayload,
    ) -> anyhow::Result<()> {
        match self {
            StoreMode::Live(store) => store.write_to_object_store(path, payload).await,
            StoreMode::Migration(store) => {
                store
                    .write_to_object_store(pipeline, path, checkpoint_range, payload)
                    .await
            }
        }
    }

    /// Update watermark after a successful file upload.
    ///
    /// In migration mode, writes the watermark to the metadata file.
    /// In live mode, this is a no-op (watermarks are derived from files).
    pub(crate) async fn update_watermark_after_upload(
        &self,
        pipeline: &str,
        epoch: u64,
        checkpoint_hi_inclusive: u64,
    ) -> Result<(), WatermarkUpdateError> {
        match self {
            StoreMode::Live(_) => Ok(()),
            StoreMode::Migration(store) => {
                store
                    .update_watermark(pipeline, epoch, checkpoint_hi_inclusive)
                    .await
            }
        }
    }
}

impl AnalyticsStore {
    /// Create a new live mode store (for streaming ingestion).
    ///
    /// Pipeline configs (including schema) are passed via `LiveStoreConfig`.
    pub fn new(
        object_store: Arc<dyn object_store::ObjectStore>,
        config: LiveStoreConfig,
        metrics: Metrics,
        upload_channel_capacity: usize,
        watermark_update_interval: Duration,
    ) -> Self {
        let pipelines = config.pipelines.clone();
        Self {
            mode: StoreMode::Live(LiveStore::new(object_store, config)),
            pending_by_pipeline: Arc::new(RwLock::new(HashMap::new())),
            pipelines: Arc::new(RwLock::new(pipelines)),
            metrics,
            uploader_senders: Arc::new(RwLock::new(HashMap::new())),
            worker_handles: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            upload_channel_capacity,
            watermark_update_interval,
        }
    }

    /// Create a new migration mode store (for rewriting existing files).
    ///
    /// Pipeline configs (including schema) are passed directly.
    /// After construction, call `load_pipelines()` to load file ranges.
    pub fn new_migration(
        object_store: Arc<dyn object_store::ObjectStore>,
        migration_id: String,
        pipeline_configs: HashMap<String, PipelineConfig>,
        metrics: Metrics,
        upload_channel_capacity: usize,
        watermark_update_interval: Duration,
    ) -> Self {
        Self {
            mode: StoreMode::Migration(MigrationStore::new(object_store, migration_id)),
            pending_by_pipeline: Arc::new(RwLock::new(HashMap::new())),
            pipelines: Arc::new(RwLock::new(pipeline_configs)),
            metrics,
            uploader_senders: Arc::new(RwLock::new(HashMap::new())),
            worker_handles: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            upload_channel_capacity,
            watermark_update_interval,
        }
    }

    /// Find the starting checkpoint for ingestion.
    ///
    /// In migration mode, loads file ranges and snaps `first_checkpoint` to file
    /// boundaries, returning the minimum adjusted checkpoint across all pipelines.
    ///
    /// In live mode, returns `first_checkpoint` unchanged.
    pub async fn find_starting_checkpoint(
        &self,
        first_checkpoint: Option<u64>,
    ) -> Result<Option<u64>> {
        match &self.mode {
            StoreMode::Live(_) => Ok(first_checkpoint),
            StoreMode::Migration(store) => {
                let pipeline_names: Vec<_> =
                    self.pipelines.read().unwrap().keys().cloned().collect();
                store
                    .find_starting_checkpoint(
                        pipeline_names.iter().map(|s| s.as_str()),
                        first_checkpoint,
                    )
                    .await
            }
        }
    }

    /// Get the store mode.
    pub fn mode(&self) -> &StoreMode {
        &self.mode
    }

    /// Get the pending rows for all pipelines.
    pub fn pending_by_pipeline(&self) -> &Arc<RwLock<HashMap<String, Batch>>> {
        &self.pending_by_pipeline
    }

    /// Get or create an uploader for a pipeline.
    ///
    /// Lazily spawns a background worker on first access.
    fn get_or_create_uploader(&self, pipeline: &str) -> mpsc::Sender<PendingFileUpload> {
        // Check if uploader already exists
        {
            let uploaders = self.uploader_senders.read().unwrap();
            if let Some(tx) = uploaders.get(pipeline) {
                return tx.clone();
            }
        }

        // Create new uploader
        let mut uploaders = self.uploader_senders.write().unwrap();
        // Double-check in case another thread created it
        if let Some(tx) = uploaders.get(pipeline) {
            return tx.clone();
        }

        let (tx, handle) = uploader::spawn_uploader(
            pipeline.to_string(),
            self.mode.clone(),
            self.metrics.clone(),
            self.upload_channel_capacity,
            self.watermark_update_interval,
        );
        uploaders.insert(pipeline.to_string(), tx.clone());

        // Track the handle for shutdown
        // Note: We can't block here, so we spawn a task to add the handle
        let handles = self.worker_handles.clone();
        tokio::spawn(async move {
            handles.lock().await.push(handle);
        });

        tx
    }

    /// Shutdown all upload workers, waiting for pending uploads to complete.
    pub async fn shutdown(&self) {
        // Clear senders to signal workers to stop
        self.uploader_senders.write().unwrap().clear();

        // Wait for all workers to finish
        let mut handles = self.worker_handles.lock().await;
        for handle in handles.drain(..) {
            let _ = handle.await;
        }
    }
}

impl<'a> AnalyticsConnection<'a> {
    /// Get the store mode for split_batch operations.
    pub fn mode(&self) -> &StoreMode {
        &self.store.mode
    }

    /// Get a clone of the pending rows for a pipeline.
    /// Returns default FileRows if pipeline has no pending rows.
    pub fn get_pending_batch(&self, pipeline: &str) -> Batch {
        self.store
            .pending_by_pipeline
            .read()
            .unwrap()
            .get(pipeline)
            .cloned()
            .unwrap_or_default()
    }

    /// Set the pending rows for a pipeline after successful upload.
    pub fn set_pending_batch(&self, pipeline: &str, rows: Batch) {
        self.store
            .pending_by_pipeline
            .write()
            .unwrap()
            .insert(pipeline.to_string(), rows);
    }

    /// Get the pipeline config for a pipeline.
    fn pipeline_config(&self, pipeline: &str) -> Result<PipelineConfig> {
        self.store
            .pipelines
            .read()
            .unwrap()
            .get(pipeline)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Pipeline '{}' not configured", pipeline))
    }

    /// Write a file to the object store.
    ///
    /// Constructs the path from the provided parameters and delegates to the store mode.
    pub async fn write_to_object_store(
        &self,
        pipeline: &str,
        epoch: EpochId,
        checkpoint_range: Range<u64>,
        file_format: FileFormat,
        payload: PutPayload,
    ) -> anyhow::Result<()> {
        let path = construct_object_store_path(pipeline, epoch, &checkpoint_range, file_format);
        self.store
            .mode
            .write_to_object_store(pipeline, &path, &checkpoint_range, payload)
            .await
    }

    /// Commit a batch of rows to the object store.
    ///
    /// # Background
    ///
    /// The indexer framework has limitations that require us to handle batching
    /// and serialization in the store layer:
    ///
    /// 1. **No minimum batch size**: The framework supports max batch size but not
    ///    min batch size, so there's no way to defer commits until a batch reaches
    ///    a certain size or to control which checkpoints end up in which output files.
    ///
    /// 2. **No fan-out/fan-in for batch processing**: The framework provides no way
    ///    to fan out processing of a completed batch (e.g., CPU-intensive serialization)
    ///    before committing it sequentially.
    ///
    /// To work around these limitations, this store accumulates rows across
    /// checkpoint commits and manages its own batching logic (by checkpoint count
    /// or row count). Serialization is offloaded to background workers via
    /// `spawn_blocking`, allowing multiple batches to serialize in parallel while
    /// maintaining strict checkpoint ordering for uploads.
    ///
    /// # Commit Lifecycle
    ///
    /// 1. Accumulates rows in pending buffer
    /// 2. When batch threshold is reached, sends to background upload worker
    /// 3. Worker serializes (parallel) and uploads (sequential by checkpoint order)
    ///
    /// Backpressure: If the upload channel is full, this method blocks.
    ///
    /// # Error Handling
    ///
    /// The framework assumes commit_batch is atomic - if an error is returned,
    /// the transaction is rolled back and retried. This method _never_ returns
    /// an error; object store write failures are retried internally. The
    /// implementation is idempotent, so framework retries would be safe anyway.
    pub async fn commit_batch(
        &mut self,
        pipeline: &str,
        batch_from_framework: &[CheckpointRows],
    ) -> Result<usize> {
        let pipeline_config = self.pipeline_config(pipeline)?;
        let file_format = pipeline_config.file_format;
        let schema = pipeline_config.schema;

        // Split batch from framework into batches that we can upload as files
        let (pending_batch, complete_batches) = {
            let pending_batch = self.get_pending_batch(pipeline);
            self.store.mode.split_framework_batch_into_files(
                pipeline,
                batch_from_framework,
                pending_batch,
            )
        };

        debug!(
            pipeline = pipeline,
            files_to_upload = complete_batches.len(),
            pending_checkpoints = pending_batch.checkpoint_count(),
            "Commit starting"
        );

        // Get the uploader for this pipeline (lazily created)
        let tx = self.store.get_or_create_uploader(pipeline);

        let mut total_rows = 0;
        for batch in complete_batches {
            total_rows += batch.row_count();

            let pending_upload = PendingFileUpload {
                epoch: batch.epoch().unwrap(),
                checkpoint_range: batch.checkpoint_range().unwrap(),
                file_format,
                checkpoints_rows: batch.checkpoints_rows,
                schema: schema.clone(),
            };

            // Send to worker - BLOCKS IF CHANNEL FULL (backpressure)
            tx.send(pending_upload)
                .await
                .unwrap_or_else(|e| panic!("Upload channel closed: {}", e));
        }

        self.set_pending_batch(pipeline, pending_batch);

        debug!(
            pipeline = pipeline,
            total_rows = total_rows,
            "Commit complete, files queued for upload"
        );

        Ok(total_rows)
    }
}

#[async_trait]
impl Store for AnalyticsStore {
    type Connection<'c> = AnalyticsConnection<'c>;

    async fn connect<'c>(&'c self) -> anyhow::Result<Self::Connection<'c>> {
        Ok(AnalyticsConnection { store: self })
    }
}

#[async_trait]
impl TransactionalStore for AnalyticsStore {
    async fn transaction<'a, R, F>(&self, f: F) -> anyhow::Result<R>
    where
        R: Send + 'a,
        F: Send + 'a,
        F: for<'r> FnOnce(
            &'r mut Self::Connection<'_>,
        ) -> ScopedBoxFuture<'a, 'r, anyhow::Result<R>>,
    {
        let mut conn = self.connect().await?;
        f(&mut conn).await
    }
}

#[async_trait]
impl Connection for AnalyticsConnection<'_> {
    /// Initialize watermark.
    ///
    /// In live mode: Watermarks are derived from file names, so just delegates to `committer_watermark`.
    /// In migration mode: If no watermark exists and `default_next_checkpoint > 0`, initializes
    /// the watermark to `default_next_checkpoint - 1` so migration starts from the configured
    /// `first_checkpoint`.
    async fn init_watermark(
        &mut self,
        pipeline_task: &str,
        default_next_checkpoint: u64,
    ) -> anyhow::Result<Option<u64>> {
        match &self.store.mode {
            StoreMode::Live(_) => {
                // Live mode: derive from file names
                Ok(self
                    .committer_watermark(pipeline_task)
                    .await?
                    .map(|w| w.checkpoint_hi_inclusive))
            }
            StoreMode::Migration(store) => {
                store
                    .init_watermark(pipeline_task, default_next_checkpoint)
                    .await
            }
        }
    }

    /// Determine the watermark.
    ///
    /// In live mode: scans file names in the object store.
    /// In migration mode: reads from watermark metadata file.
    async fn committer_watermark(
        &mut self,
        pipeline: &str,
    ) -> anyhow::Result<Option<CommitterWatermark>> {
        match &self.store.mode {
            StoreMode::Live(store) => store.committer_watermark(pipeline).await,
            StoreMode::Migration(store) => store.committer_watermark(pipeline).await,
        }
    }

    async fn reader_watermark(
        &mut self,
        _pipeline: &'static str,
    ) -> anyhow::Result<Option<ReaderWatermark>> {
        // Reader watermark not supported - no pruning in analytics indexer
        Ok(None)
    }

    async fn pruner_watermark(
        &mut self,
        _pipeline: &'static str,
        _delay: Duration,
    ) -> anyhow::Result<Option<PrunerWatermark>> {
        // Pruning not supported in analytics indexer
        Ok(None)
    }

    /// No-op: watermarks are managed by the upload worker, not the framework.
    ///
    /// The framework calls this after processing checkpoints, but those checkpoints
    /// may not be uploaded yet. The upload worker updates watermarks after each
    /// successful file upload to ensure we only record progress for uploaded data.
    async fn set_committer_watermark(
        &mut self,
        _pipeline_task: &str,
        _watermark: CommitterWatermark,
    ) -> anyhow::Result<bool> {
        Ok(true)
    }

    async fn set_reader_watermark(
        &mut self,
        _pipeline: &'static str,
        _reader_lo: u64,
    ) -> anyhow::Result<bool> {
        bail!("Pruning not supported by analytics store");
    }

    async fn set_pruner_watermark(
        &mut self,
        _pipeline: &'static str,
        _pruner_hi: u64,
    ) -> anyhow::Result<bool> {
        bail!("Pruning not supported by analytics store");
    }
}

/// Construct the object store path for an analytics file.
/// Path format: {pipeline}/epoch_{epoch}/{start}_{end}.{ext}
pub(crate) fn construct_object_store_path(
    pipeline: &str,
    epoch: EpochId,
    checkpoint_range: &Range<u64>,
    file_format: FileFormat,
) -> ObjectPath {
    let extension = match file_format {
        FileFormat::Csv => "csv",
        FileFormat::Parquet => "parquet",
    };
    ObjectPath::from(format!(
        "{}/epoch_{}/{}_{}.{}",
        pipeline, epoch, checkpoint_range.start, checkpoint_range.end, extension
    ))
}

/// Parse checkpoint range from filename.
/// Expected format: `{start}_{end}.{format}` (e.g., `0_100.parquet`)
pub(crate) fn parse_checkpoint_range(filename: &str) -> Option<Range<u64>> {
    let base = filename.split('.').next()?;
    let (start_str, end_str) = base.split_once('_')?;
    let start: u64 = start_str.parse().ok()?;
    let end: u64 = end_str.parse().ok()?;
    Some(start..end)
}
