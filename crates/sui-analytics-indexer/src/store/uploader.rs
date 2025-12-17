// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Async upload worker for analytics files.
//!
//! This module provides a background worker that handles file serialization and upload
//! asynchronously. Files are uploaded in the order they were scheduled (using sequence numbers)
//! to maintain checkpoint ordering guarantees.

use std::collections::BTreeMap;
use std::ops::Range;
use std::time::Duration;

use anyhow::Result;
use bytes::Bytes;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use object_store::PutPayload;
use object_store::path::Path as ObjectPath;
use sui_types::base_types::EpochId;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use crate::config::FileFormat;
use crate::handlers::CheckpointRows;
use crate::handlers::record_file_metrics;
use crate::metrics::Metrics;
use crate::writers::{CsvWriter, ParquetWriter};

use super::{StoreMode, WatermarkUpdateError, construct_object_store_path};

/// Initial backoff delay for retries.
const INITIAL_BACKOFF: Duration = Duration::from_millis(100);
/// Maximum backoff delay for retries.
const MAX_BACKOFF: Duration = Duration::from_secs(60 * 5);

/// Helper for exponential backoff with jitter.
struct Backoff {
    current: Duration,
}

impl Backoff {
    fn new() -> Self {
        Self {
            current: INITIAL_BACKOFF,
        }
    }

    /// Sleep for the current backoff duration, then increase it.
    async fn sleep_and_advance(&mut self) {
        tokio::time::sleep(self.current).await;
        self.current = (self.current * 2).min(MAX_BACKOFF);
    }

    /// Get the current backoff duration (for logging).
    fn current_ms(&self) -> u128 {
        self.current.as_millis()
    }
}

/// The rows for a checkpoint, ready to be serialized and uploaded.
pub struct PendingFileUpload {
    pub epoch: EpochId,
    pub checkpoint_range: Range<u64>,
    pub file_format: FileFormat,
    pub checkpoints_rows: Vec<CheckpointRows>,
    pub schema: Vec<String>,
}

/// A serialized file ready for upload.
struct SerializedFile {
    epoch: EpochId,
    checkpoint_range: Range<u64>,
    file_format: FileFormat,
    bytes: Bytes,
}

/// Spawn an upload worker for a pipeline.
///
/// Returns the sender for queueing files and the worker's JoinHandle for shutdown.
pub fn spawn_uploader(
    pipeline: String,
    mode: StoreMode,
    metrics: Metrics,
    channel_capacity: usize,
    watermark_update_interval: Duration,
) -> (mpsc::Sender<PendingFileUpload>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel(channel_capacity);

    let worker = UploadWorker::new(rx, pipeline, mode, metrics, watermark_update_interval);
    let worker_handle = tokio::spawn(worker.run());

    (tx, worker_handle)
}

/// Background worker that serializes and uploads files in order.
struct UploadWorker {
    rx: mpsc::Receiver<PendingFileUpload>,
    /// Next sequence number to assign to incoming files
    next_seq: u64,
    /// Serialization tasks in flight, returns (seq, file)
    serializing: FuturesUnordered<JoinHandle<Result<(u64, SerializedFile)>>>,
    /// Serialized files waiting for upload, keyed by sequence number
    pending_upload: BTreeMap<u64, SerializedFile>,
    /// Next sequence number to upload (upload in order: 0, 1, 2, ...)
    next_upload_seq: u64,
    /// Store mode for upload logic
    mode: StoreMode,
    /// Pipeline name
    pipeline: String,
    /// Metrics
    metrics: Metrics,
    /// Last time we wrote watermark to object store (for rate limiting)
    last_watermark_update: Option<std::time::Instant>,
    /// Minimum interval between watermark writes
    watermark_update_interval: Duration,
    /// Latest uploaded watermark (epoch, checkpoint_hi_inclusive).
    /// Updated after every successful upload, written to object store on interval + shutdown.
    latest_watermark: Option<(EpochId, u64)>,
}

impl UploadWorker {
    fn new(
        rx: mpsc::Receiver<PendingFileUpload>,
        pipeline: String,
        mode: StoreMode,
        metrics: Metrics,
        watermark_update_interval: Duration,
    ) -> Self {
        Self {
            rx,
            next_seq: 0,
            serializing: FuturesUnordered::new(),
            pending_upload: BTreeMap::new(),
            next_upload_seq: 0,
            mode,
            pipeline,
            metrics,
            last_watermark_update: None,
            watermark_update_interval,
            latest_watermark: None,
        }
    }

    async fn run(mut self) {
        debug!(pipeline = %self.pipeline, "Upload worker starting");

        loop {
            tokio::select! {
                // Receive new batch to serialize
                Some(batch) = self.rx.recv() => {
                    self.spawn_serialization(batch);
                }

                // Serialization task completed
                Some(result) = self.serializing.next(), if !self.serializing.is_empty() => {
                    match result {
                        Ok(Ok((seq, file))) => {
                            self.pending_upload.insert(seq, file);
                            self.drain_and_upload().await;
                        }
                        Ok(Err(e)) => {
                            // Serialization error - fatal, return to close channel and stop pipeline
                            error!(pipeline = %self.pipeline, error = %e, "Serialization failed, stopping upload worker");
                            return;
                        }
                        Err(e) => {
                            // Task panicked - fatal, return to close channel and stop pipeline
                            error!(pipeline = %self.pipeline, error = %e, "Serialization task panicked, stopping upload worker");
                            return;
                        }
                    }
                }

                // Channel closed and no more serialization work
                else => {
                    if self.serializing.is_empty() {
                        break;
                    }
                    // Wait for remaining serialization tasks
                }
            }
        }

        // Drain remaining serialization tasks
        while let Some(result) = self.serializing.next().await {
            if let Ok(Ok((seq, file))) = result {
                self.pending_upload.insert(seq, file);
            }
        }
        self.drain_and_upload().await;

        // Final watermark flush on shutdown
        if let Some((epoch, checkpoint_hi)) = self.latest_watermark {
            self.update_watermark_with_retry(epoch, checkpoint_hi).await;
            info!(
                pipeline = %self.pipeline,
                epoch,
                checkpoint_hi,
                "Flushed watermark on shutdown"
            );
        }

        debug!(pipeline = %self.pipeline, "Upload worker finished");
    }

    fn spawn_serialization(&mut self, batch: PendingFileUpload) {
        let seq = self.next_seq;
        self.next_seq += 1;

        let pipeline = self.pipeline.clone();

        let handle = tokio::task::spawn_blocking(move || {
            let bytes = serialize_rows(&batch.checkpoints_rows, &batch.schema, batch.file_format)?;
            debug!(
                pipeline = %pipeline,
                seq = seq,
                checkpoint_range = ?batch.checkpoint_range,
                bytes = bytes.len(),
                "Serialized file"
            );
            Ok((
                seq,
                SerializedFile {
                    epoch: batch.epoch,
                    checkpoint_range: batch.checkpoint_range,
                    file_format: batch.file_format,
                    bytes,
                },
            ))
        });
        self.serializing.push(handle);
    }

    async fn drain_and_upload(&mut self) {
        // Upload files in sequence order (0, 1, 2, ...)
        while let Some(file) = self.pending_upload.remove(&self.next_upload_seq) {
            self.upload_with_retry(&file).await;
            self.next_upload_seq += 1;
        }
    }

    async fn upload_with_retry(&mut self, file: &SerializedFile) {
        let mut backoff = Backoff::new();

        let path = construct_object_store_path(
            &self.pipeline,
            file.epoch,
            &file.checkpoint_range,
            file.file_format,
        );

        loop {
            match self
                .do_upload(&path, &file.checkpoint_range, file.bytes.clone())
                .await
            {
                Ok(()) => {
                    record_file_metrics(&self.metrics, &self.pipeline, file.bytes.len());
                    info!(
                        pipeline = %self.pipeline,
                        epoch = file.epoch,
                        checkpoint_range = ?file.checkpoint_range,
                        bytes = file.bytes.len(),
                        "Uploaded file"
                    );

                    // Always track latest watermark in memory (for shutdown flush)
                    let checkpoint_hi = file.checkpoint_range.end - 1;
                    self.latest_watermark = Some((file.epoch, checkpoint_hi));

                    // Rate-limit writes to object store
                    let should_update = self
                        .last_watermark_update
                        .map(|last| last.elapsed() >= self.watermark_update_interval)
                        .unwrap_or(true);

                    if should_update {
                        self.update_watermark_with_retry(file.epoch, checkpoint_hi)
                            .await;
                        self.last_watermark_update = Some(std::time::Instant::now());
                    }

                    return;
                }
                Err(e) => {
                    warn!(
                        pipeline = %self.pipeline,
                        checkpoint_range = ?file.checkpoint_range,
                        error = %e,
                        backoff_ms = backoff.current_ms(),
                        "Upload failed, retrying"
                    );
                    backoff.sleep_and_advance().await;
                }
            }
        }
    }

    async fn do_upload(
        &self,
        path: &ObjectPath,
        checkpoint_range: &Range<u64>,
        bytes: Bytes,
    ) -> Result<()> {
        self.mode
            .write_to_object_store(
                &self.pipeline,
                path,
                checkpoint_range,
                PutPayload::from(bytes),
            )
            .await
    }

    /// Update watermark with retry on transient errors, panic on concurrent writer.
    ///
    /// This blocks the uploader until the watermark is written successfully.
    /// Concurrent writer detection (precondition failure) causes a panic since
    /// it indicates a serious configuration error (multiple instances writing
    /// to the same migration).
    async fn update_watermark_with_retry(&self, epoch: EpochId, checkpoint_hi: u64) {
        let mut backoff = Backoff::new();

        loop {
            match self
                .mode
                .update_watermark_after_upload(&self.pipeline, epoch, checkpoint_hi)
                .await
            {
                Ok(()) => return,
                Err(WatermarkUpdateError::ConcurrentWriter { path, message }) => {
                    // Fatal: concurrent writer detected, this is a configuration error
                    panic!(
                        "Concurrent writer detected on watermark {}: {}. \
                         Only one migration instance should run at a time per pipeline.",
                        path, message
                    );
                }
                Err(WatermarkUpdateError::Transient(e)) => {
                    warn!(
                        pipeline = %self.pipeline,
                        epoch,
                        checkpoint_hi,
                        error = %e,
                        backoff_ms = backoff.current_ms(),
                        "Transient error updating watermark, retrying"
                    );
                    backoff.sleep_and_advance().await;
                }
            }
        }
    }
}

/// Serialize rows grouped by checkpoint to the appropriate file format.
fn serialize_rows(
    checkpoints: &[CheckpointRows],
    schema: &[String],
    format: FileFormat,
) -> Result<Bytes> {
    match format {
        FileFormat::Csv => {
            let mut writer = CsvWriter::new()?;
            for checkpoint in checkpoints {
                writer.write(checkpoint)?;
            }
            writer
                .flush()
                .map(|opt| opt.unwrap_or_default())
                .map(Bytes::from)
        }
        FileFormat::Parquet => {
            let mut writer = ParquetWriter::new()?;
            for checkpoint in checkpoints {
                writer.write(checkpoint)?;
            }
            writer
                .flush(schema)
                .map(|opt| opt.unwrap_or_default())
                .map(Bytes::from)
        }
    }
}
