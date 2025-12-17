// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Migration mode store - uses explicit watermark files and conditional PUT.
//!
//! In migration mode, we rewrite existing files (e.g., adding new columns to parquet files).
//! This module provides the store implementation and utilities to track existing file ranges.

use std::collections::{BTreeMap, HashMap};
use std::ops::Range;
use std::sync::Arc;
use std::sync::RwLock;

use anyhow::{Context, Result, anyhow, bail};
use object_store::path::Path as ObjectPath;
use object_store::{
    Error as ObjectStoreError, ObjectStore, PutMode, PutOptions, PutPayload, UpdateVersion,
};
use sui_indexer_alt_framework_store_traits::CommitterWatermark;
use sui_storage::object_store::util::find_all_dirs_with_epoch_prefix;
use thiserror::Error;
use tracing::info;

/// Error type for watermark updates.
#[derive(Error, Debug)]
pub enum WatermarkUpdateError {
    /// Precondition failure - concurrent writer detected. This is fatal.
    #[error("Concurrent writer detected on watermark {path}: {message}")]
    ConcurrentWriter { path: String, message: String },

    /// Transient error - can be retried.
    #[error("Transient error updating watermark: {0}")]
    Transient(#[from] anyhow::Error),
}

use crate::handlers::CheckpointRows;

use super::Batch;

/// Version info (etag, version) for conditional PUT operations.
type VersionInfo = (Option<String>, Option<String>);

/// Simple watermark struct for JSON serialization.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct MigrationWatermark {
    pub checkpoint_hi_inclusive: u64,
    /// Epoch of the watermark - used to skip scanning earlier epochs on restart.
    pub epoch_hi_inclusive: u64,
}

/// Migration mode - uses explicit watermark files and conditional PUT.
///
/// Used for rewriting existing files (e.g., adding new columns to parquet files).
/// Tracks progress separately via watermark files and uses conditional PUT to
/// ensure we are overwriting the files we expect.
#[derive(Clone)]
pub struct MigrationStore {
    object_store: Arc<dyn ObjectStore>,
    /// Migration identifier.
    migration_id: String,
    /// Pipeline -> FileRangeIndex (target ranges).
    file_ranges: Arc<RwLock<HashMap<String, FileRangeIndex>>>,
    /// Pipeline -> (etag, version) for conditional PUT on watermark files.
    watermark_versions: Arc<RwLock<HashMap<String, VersionInfo>>>,
    /// Pre-computed per-pipeline adjusted starting checkpoints.
    /// Set during pre-loading in build_analytics_indexer.
    adjusted_start_checkpoints: Arc<RwLock<HashMap<String, u64>>>,
}

/// Entry for a single file range in the index.
#[derive(Debug, Clone)]
pub struct FileRangeEntry {
    /// End checkpoint (exclusive).
    pub end: u64,
    /// Epoch this file belongs to.
    pub epoch: u64,
    /// ETag for conditional PUT operations.
    pub e_tag: Option<String>,
    /// Version for conditional PUT operations.
    pub version: Option<String>,
}

/// Index of existing file ranges for a pipeline.
///
/// In migration mode, this is loaded at startup to track target file ranges.
/// Progress is tracked separately via a watermark file.
#[derive(Debug, Default, Clone)]
pub struct FileRangeIndex {
    /// Map from start_checkpoint -> FileRangeEntry.
    /// Sorted by start checkpoint for efficient lookups.
    ranges: BTreeMap<u64, FileRangeEntry>,
}

impl MigrationStore {
    /// Create a new migration store.
    pub fn new(object_store: Arc<dyn ObjectStore>, migration_id: String) -> Self {
        Self {
            object_store,
            migration_id,
            file_ranges: Arc::new(RwLock::new(HashMap::new())),
            watermark_versions: Arc::new(RwLock::new(HashMap::new())),
            adjusted_start_checkpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load file ranges and find the starting checkpoint for migration.
    ///
    /// This snaps `first_checkpoint` to file boundaries:
    /// - If checkpoint is inside a file → snap to file start
    /// - If checkpoint is in a gap → snap to next file start
    /// - If no files at or after checkpoint → error
    ///
    /// Returns the minimum adjusted checkpoint across all pipelines (for framework ingestion).
    pub async fn find_starting_checkpoint(
        &self,
        pipeline_names: impl Iterator<Item = &str>,
        first_checkpoint: Option<u64>,
    ) -> Result<Option<u64>> {
        let mut file_ranges = HashMap::new();
        let mut adjusted_starts = HashMap::new();
        let mut min_adjusted: Option<u64> = None;

        for pipeline_name in pipeline_names {
            // Load file ranges for this pipeline (all epochs)
            let index =
                FileRangeIndex::load_from_store(&self.object_store, pipeline_name, None).await?;

            // Compute adjusted starting checkpoint if first_checkpoint specified
            if let Some(first_cp) = first_checkpoint {
                let adjusted = index.snap_to_boundary(first_cp).ok_or_else(|| {
                    anyhow!(
                        "No files at or after checkpoint {} for pipeline '{}'. Nothing to migrate.",
                        first_cp,
                        pipeline_name
                    )
                })?;
                adjusted_starts.insert(pipeline_name.to_string(), adjusted);
                min_adjusted = Some(min_adjusted.map_or(adjusted, |m| m.min(adjusted)));

                info!(
                    pipeline = pipeline_name,
                    requested_checkpoint = first_cp,
                    adjusted_checkpoint = adjusted,
                    "Snapped first_checkpoint to file boundary"
                );
            }

            file_ranges.insert(pipeline_name.to_string(), index);
        }

        // Store the loaded data
        *self.file_ranges.write().unwrap() = file_ranges;
        *self.adjusted_start_checkpoints.write().unwrap() = adjusted_starts;

        // Return minimum adjusted checkpoint (or original if no first_checkpoint specified)
        Ok(min_adjusted.or(first_checkpoint))
    }

    pub fn migration_id(&self) -> &str {
        &self.migration_id
    }

    pub fn file_ranges(&self) -> &Arc<RwLock<HashMap<String, FileRangeIndex>>> {
        &self.file_ranges
    }

    /// Read watermark from metadata file and cache its etag/version.
    pub(crate) async fn committer_watermark(
        &self,
        pipeline: &str,
    ) -> anyhow::Result<Option<CommitterWatermark>> {
        let path = migration_watermark_path(pipeline, &self.migration_id);
        match self.object_store.get(&path).await {
            Ok(result) => {
                // Capture etag and version for conditional PUT
                let e_tag = result.meta.e_tag.clone();
                let version = result.meta.version.clone();
                self.watermark_versions
                    .write()
                    .unwrap()
                    .insert(pipeline.to_string(), (e_tag, version));

                let bytes = result.bytes().await?;
                let watermark: MigrationWatermark = serde_json::from_slice(&bytes)
                    .context("Failed to parse migration watermark from object store")?;
                info!(
                    pipeline,
                    migration_id = self.migration_id,
                    epoch = watermark.epoch_hi_inclusive,
                    checkpoint = watermark.checkpoint_hi_inclusive,
                    "Migration mode: found progress from watermark file"
                );
                Ok(Some(CommitterWatermark {
                    epoch_hi_inclusive: watermark.epoch_hi_inclusive,
                    checkpoint_hi_inclusive: watermark.checkpoint_hi_inclusive,
                    tx_hi: 0,
                    timestamp_ms_hi_inclusive: 0,
                }))
            }
            Err(ObjectStoreError::NotFound { .. }) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Initialize migration mode for a pipeline.
    ///
    /// If file ranges were pre-loaded via `set_preloaded_data`, uses those and the
    /// pre-computed adjusted starting checkpoint. Otherwise falls back to loading
    /// file ranges and using `default_next_checkpoint` directly.
    ///
    /// 1. Reads or creates the watermark file
    /// 2. Loads file ranges starting from the watermark epoch (if not pre-loaded)
    pub(crate) async fn init_watermark(
        &self,
        pipeline: &str,
        default_next_checkpoint: u64,
    ) -> anyhow::Result<Option<u64>> {
        // Use pre-computed adjusted checkpoint if available, otherwise use default
        let adjusted_next = self
            .adjusted_start_checkpoints
            .read()
            .unwrap()
            .get(pipeline)
            .copied()
            .unwrap_or(default_next_checkpoint);

        // Check existing watermark first
        let (checkpoint_hi, epoch_hi) = if let Some(watermark) =
            self.committer_watermark(pipeline).await?
        {
            (
                Some(watermark.checkpoint_hi_inclusive),
                Some(watermark.epoch_hi_inclusive),
            )
        } else if let Some(checkpoint_hi_inclusive) = adjusted_next.checked_sub(1) {
            // No existing watermark - initialize if adjusted_next > 0
            // Look up actual epoch from epochs.json
            let epoch =
                lookup_epoch_for_checkpoint(&self.object_store, checkpoint_hi_inclusive).await?;

            let path = migration_watermark_path(pipeline, &self.migration_id);
            let json = serde_json::to_vec(&MigrationWatermark {
                checkpoint_hi_inclusive,
                epoch_hi_inclusive: epoch,
            })?;

            // Use conditional PUT - fail if file already exists (race condition)
            let result = self
                .object_store
                .put_opts(
                    &path,
                    json.into(),
                    PutOptions {
                        mode: PutMode::Create,
                        ..Default::default()
                    },
                )
                .await
                .context("Failed to create initial watermark (already exists?)")?;

            // Cache etag/version for subsequent conditional updates
            self.watermark_versions
                .write()
                .unwrap()
                .insert(pipeline.to_string(), (result.e_tag, result.version));

            info!(
                pipeline,
                migration_id = self.migration_id,
                checkpoint = checkpoint_hi_inclusive,
                epoch,
                "Initialized migration watermark"
            );
            (Some(checkpoint_hi_inclusive), Some(epoch))
        } else {
            (None, None)
        };

        // Load file ranges if not already pre-loaded
        if !self.file_ranges.read().unwrap().contains_key(pipeline) {
            let index =
                FileRangeIndex::load_from_store(&self.object_store, pipeline, epoch_hi).await?;
            self.file_ranges
                .write()
                .unwrap()
                .insert(pipeline.to_string(), index);
        }

        Ok(checkpoint_hi)
    }

    /// Update watermark for a single pipeline after successful file upload.
    ///
    /// Called by the upload worker after each file is successfully uploaded.
    /// This provides incremental progress tracking for crash recovery.
    ///
    /// Returns `WatermarkUpdateError::ConcurrentWriter` on precondition failure (fatal),
    /// or `WatermarkUpdateError::Transient` on other errors (can be retried).
    pub(crate) async fn update_watermark(
        &self,
        pipeline: &str,
        epoch_hi_inclusive: u64,
        checkpoint_hi_inclusive: u64,
    ) -> std::result::Result<(), WatermarkUpdateError> {
        let path = migration_watermark_path(pipeline, &self.migration_id);
        let json = serde_json::to_vec(&MigrationWatermark {
            checkpoint_hi_inclusive,
            epoch_hi_inclusive,
        })
        .map_err(|e| WatermarkUpdateError::Transient(e.into()))?;

        // Look up cached etag/version for conditional PUT
        let (e_tag, version) = self
            .watermark_versions
            .read()
            .unwrap()
            .get(pipeline)
            .cloned()
            .unwrap_or((None, None));

        let mode = if e_tag.is_some() || version.is_some() {
            PutMode::Update(UpdateVersion { e_tag, version })
        } else {
            PutMode::Create
        };

        let result = self
            .object_store
            .put_opts(
                &path,
                json.into(),
                PutOptions {
                    mode,
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| match e {
                ObjectStoreError::Precondition { path, source } => {
                    WatermarkUpdateError::ConcurrentWriter {
                        path: path.to_string(),
                        message: source.to_string(),
                    }
                }
                other => WatermarkUpdateError::Transient(other.into()),
            })?;

        // Update cached etag/version
        self.watermark_versions
            .write()
            .unwrap()
            .insert(pipeline.to_string(), (result.e_tag, result.version));

        tracing::debug!(
            pipeline,
            migration_id = self.migration_id,
            checkpoint = checkpoint_hi_inclusive,
            epoch = epoch_hi_inclusive,
            "Updated migration watermark"
        );

        Ok(())
    }

    /// Split a batch of checkpoints into files based on existing file boundaries.
    ///
    /// In migration mode, we match the boundaries of existing files to ensure
    /// we can use conditional PUT with the correct e_tag/version.
    pub(crate) fn split_framework_batch_into_files(
        &self,
        pipeline: &str,
        batch_from_framework: &[CheckpointRows],
        mut pending_batch: Batch,
    ) -> (Batch, Vec<Batch>) {
        let mut complete_batches: Vec<Batch> = Vec::new();

        let ranges = self
            .file_ranges
            .read()
            .unwrap()
            .get(pipeline)
            .cloned()
            .expect("migration ranges not loaded for pipeline");

        for checkpoint_rows in batch_from_framework {
            // Cut at file boundary (end is exclusive in file names)
            if let Some(first) = pending_batch.first_checkpoint()
                && let Some((_, entry)) = ranges.find_containing(first)
            {
                assert!(
                    checkpoint_rows.checkpoint <= entry.end,
                    "missed file boundary: expected cut at {}, now at {}",
                    entry.end,
                    checkpoint_rows.checkpoint
                );
                if checkpoint_rows.checkpoint == entry.end {
                    complete_batches.push(pending_batch);
                    pending_batch = Batch::default();
                }
            }
            pending_batch.add(checkpoint_rows.clone());
        }

        // Flush if we've exactly filled a file range
        if let Some(first) = pending_batch.first_checkpoint()
            && let Some((_, entry)) = ranges.find_containing(first)
            && pending_batch
                .last_checkpoint()
                .is_some_and(|last| last + 1 == entry.end)
        {
            complete_batches.push(pending_batch);
            pending_batch = Batch::default();
        }

        (pending_batch, complete_batches)
    }

    /// Write a file to the object store with conditional update.
    ///
    /// Looks up the expected etag/version from the file range index and verifies
    /// the checkpoint range matches exactly before performing a conditional PUT.
    pub(crate) async fn write_to_object_store(
        &self,
        pipeline: &str,
        path: &ObjectPath,
        checkpoint_range: &Range<u64>,
        payload: PutPayload,
    ) -> anyhow::Result<()> {
        let (e_tag, version) = {
            let ranges = self.file_ranges.read().unwrap();
            let pipeline_ranges = ranges.get(pipeline).expect("migration ranges not loaded");

            if let Some((start, entry)) = pipeline_ranges.find_containing(checkpoint_range.start) {
                // Verify the range matches exactly
                assert_eq!(
                    start, checkpoint_range.start,
                    "checkpoint range start mismatch: expected {}, got {}",
                    start, checkpoint_range.start
                );
                assert_eq!(
                    entry.end, checkpoint_range.end,
                    "checkpoint range end mismatch: expected {}, got {}",
                    entry.end, checkpoint_range.end
                );
                (entry.e_tag.clone(), entry.version.clone())
            } else {
                (None, None)
            }
        };

        self.put_conditional(path, payload, e_tag.as_deref(), version.as_deref())
            .await
    }

    /// Put a file with conditional update for migration mode.
    ///
    /// Uses `PutMode::Update` with etag/version for atomic replacement to prevent
    /// concurrent modification.
    async fn put_conditional(
        &self,
        path: &ObjectPath,
        payload: PutPayload,
        expected_etag: Option<&str>,
        expected_version: Option<&str>,
    ) -> anyhow::Result<()> {
        let mode = if expected_etag.is_some() || expected_version.is_some() {
            PutMode::Update(UpdateVersion {
                e_tag: expected_etag.map(String::from),
                version: expected_version.map(String::from),
            })
        } else {
            PutMode::Create
        };

        self.object_store
            .put_opts(
                path,
                payload,
                PutOptions {
                    mode,
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| match e {
                ObjectStoreError::Precondition { path, source } => {
                    anyhow!(
                        "Concurrent writer detected - etag mismatch for {}: {}",
                        path,
                        source
                    )
                }
                ObjectStoreError::AlreadyExists { path, source } => {
                    anyhow!(
                        "File already exists (expected for conditional create): {}: {}",
                        path,
                        source
                    )
                }
                _ => e.into(),
            })?;
        Ok(())
    }
}

impl FileRangeIndex {
    /// Create a new empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a file range entry.
    pub fn insert(&mut self, start: u64, entry: FileRangeEntry) {
        self.ranges.insert(start, entry);
    }

    /// Get the number of file ranges.
    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// Find the file range that contains the given checkpoint.
    ///
    /// Returns `Some((start, entry))` if a range contains the checkpoint,
    /// `None` otherwise.
    pub fn find_containing(&self, checkpoint: u64) -> Option<(u64, &FileRangeEntry)> {
        // Find the largest start <= checkpoint
        self.ranges
            .range(..=checkpoint)
            .next_back()
            .filter(|(start, entry)| checkpoint >= **start && checkpoint < entry.end)
            .map(|(start, entry)| (*start, entry))
    }

    /// Find the next file boundary at or after the given checkpoint.
    ///
    /// Returns `Some((start, entry))` for the next file range, or `None` if
    /// there are no more ranges.
    pub fn find_next_boundary(&self, checkpoint: u64) -> Option<(u64, &FileRangeEntry)> {
        self.ranges
            .range(checkpoint..)
            .next()
            .map(|(&start, entry)| (start, entry))
    }

    /// Snap a checkpoint to file boundaries for migration.
    ///
    /// - If checkpoint is inside a file → returns file start
    /// - If checkpoint is in a gap → returns next file start
    /// - If no files at or after checkpoint → returns None (error case)
    pub fn snap_to_boundary(&self, checkpoint: u64) -> Option<u64> {
        // Check if checkpoint is inside a file
        if let Some((start, _)) = self.find_containing(checkpoint) {
            return Some(start);
        }
        // Check for next file after checkpoint
        if let Some((next_start, _)) = self.find_next_boundary(checkpoint) {
            return Some(next_start);
        }
        // No files at or after checkpoint
        None
    }

    /// Get the first checkpoint across all ranges.
    pub fn first_checkpoint(&self) -> Option<u64> {
        self.ranges.keys().next().copied()
    }

    /// Get the last checkpoint across all ranges (exclusive).
    pub fn last_checkpoint_exclusive(&self) -> Option<u64> {
        self.ranges.values().map(|e| e.end).max()
    }

    /// Load file range index from object store.
    ///
    /// This lists all files in the pipeline directory to build the index of
    /// target ranges for migration. Progress is tracked via a separate watermark file.
    ///
    /// If `min_epoch` is provided, only epochs >= min_epoch are scanned, which
    /// reduces startup time when resuming a migration.
    pub async fn load_from_store(
        store: &Arc<dyn ObjectStore>,
        pipeline: &str,
        min_epoch: Option<u64>,
    ) -> Result<Self> {
        let mut index = Self::new();

        // Find all epoch directories under {pipeline}/epoch_*
        let prefix = ObjectPath::from(pipeline);
        let epoch_dirs = find_all_dirs_with_epoch_prefix(store, Some(&prefix)).await?;

        let skipped_epochs = min_epoch
            .map(|min| epoch_dirs.range(..min).count())
            .unwrap_or(0);

        for (epoch, epoch_path) in epoch_dirs {
            // Skip epochs before the watermark
            if let Some(min) = min_epoch
                && epoch < min
            {
                continue;
            }

            // List files in this epoch directory
            let list_result = store.list_with_delimiter(Some(&epoch_path)).await?;

            for obj in list_result.objects {
                // Parse checkpoint range from filename: {start}_{end}.{format}
                let Some(filename) = obj.location.filename() else {
                    continue;
                };
                let Some(range) = super::parse_checkpoint_range(filename) else {
                    continue;
                };

                index.insert(
                    range.start,
                    FileRangeEntry {
                        end: range.end,
                        epoch,
                        e_tag: obj.e_tag,
                        version: obj.version,
                    },
                );
            }
        }

        info!(
            pipeline,
            num_files = index.len(),
            skipped_epochs,
            min_epoch,
            first_checkpoint = ?index.first_checkpoint(),
            last_checkpoint = ?index.last_checkpoint_exclusive(),
            "Loaded existing file ranges"
        );

        Ok(index)
    }
}

/// Construct the path for a migration watermark file.
///
/// Format: `_metadata/watermarks/{pipeline}@migration_{migration_id}.json`
pub(crate) fn migration_watermark_path(pipeline: &str, migration_id: &str) -> ObjectPath {
    ObjectPath::from(format!(
        "_metadata/watermarks/{}@migration_{}.json",
        pipeline, migration_id
    ))
}

/// Look up the epoch for a given checkpoint from epochs.json.
///
/// The epochs.json file contains an array of last checkpoint numbers per epoch:
/// `[last_cp_epoch_0, last_cp_epoch_1, ...]`
/// where epochs[i] is the last checkpoint (inclusive) of epoch i.
///
/// - Epoch 0: checkpoints 0..=epochs[0]
/// - Epoch 1: checkpoints (epochs[0]+1)..=epochs[1]
/// - etc.
async fn lookup_epoch_for_checkpoint(
    object_store: &Arc<dyn ObjectStore>,
    checkpoint: u64,
) -> anyhow::Result<u64> {
    let path = ObjectPath::from("epochs.json");
    let result = object_store
        .get(&path)
        .await
        .context("Failed to download epochs.json")?;
    let bytes = result.bytes().await?;
    let epochs: Vec<u64> = serde_json::from_slice(&bytes).context("Failed to parse epochs.json")?;

    // Linear search for the epoch containing this checkpoint
    // epochs[i] = last checkpoint (inclusive) of epoch i
    for (epoch, &last_checkpoint) in epochs.iter().enumerate() {
        if checkpoint <= last_checkpoint {
            return Ok(epoch as u64);
        }
    }

    // Checkpoint is beyond all known epochs - return error
    bail!(
        "Checkpoint {} is beyond the last known epoch {} (ends at {})",
        checkpoint,
        epochs.len().saturating_sub(1),
        epochs.last().copied().unwrap_or(0)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use object_store::memory::InMemory;

    #[tokio::test]
    async fn test_migration_mode_watermark() {
        let object_store: Arc<dyn ObjectStore> = Arc::new(InMemory::new());

        // Create migration store directly to test watermark updates
        let migration_store = MigrationStore::new(object_store.clone(), "test_migration".into());

        // No watermark file yet
        let watermark = migration_store
            .committer_watermark("test_pipeline")
            .await
            .unwrap();
        assert!(watermark.is_none());

        // Update watermark (simulating what uploader does after upload)
        migration_store
            .update_watermark("test_pipeline", 5, 500)
            .await
            .unwrap();

        // Read it back
        let watermark = migration_store
            .committer_watermark("test_pipeline")
            .await
            .unwrap();
        assert!(watermark.is_some());
        let watermark = watermark.unwrap();
        assert_eq!(watermark.epoch_hi_inclusive, 5);
        assert_eq!(watermark.checkpoint_hi_inclusive, 500);
    }

    #[test]
    fn test_parse_checkpoint_range() {
        use super::super::parse_checkpoint_range;
        assert_eq!(parse_checkpoint_range("0_100.parquet"), Some(0..100));
        assert_eq!(parse_checkpoint_range("100_200.csv"), Some(100..200));
        assert_eq!(
            parse_checkpoint_range("1234_5678.parquet"),
            Some(1234..5678)
        );
        assert_eq!(parse_checkpoint_range("invalid"), None);
        assert_eq!(parse_checkpoint_range("no_extension"), None);
        assert_eq!(parse_checkpoint_range("a_b.parquet"), None);
    }

    #[test]
    fn test_find_containing() {
        let mut index = FileRangeIndex::new();
        index.insert(
            0,
            FileRangeEntry {
                end: 100,
                epoch: 0,
                e_tag: None,
                version: None,
            },
        );
        index.insert(
            100,
            FileRangeEntry {
                end: 200,
                epoch: 0,
                e_tag: None,
                version: None,
            },
        );
        index.insert(
            200,
            FileRangeEntry {
                end: 300,
                epoch: 1,
                e_tag: None,
                version: None,
            },
        );

        // Test checkpoint in first range
        let result = index.find_containing(50);
        assert!(result.is_some());
        let (start, entry) = result.unwrap();
        assert_eq!(start, 0);
        assert_eq!(entry.end, 100);

        // Test checkpoint at boundary (should be in second range)
        let result = index.find_containing(100);
        assert!(result.is_some());
        let (start, entry) = result.unwrap();
        assert_eq!(start, 100);
        assert_eq!(entry.end, 200);

        // Test checkpoint not in any range
        let result = index.find_containing(300);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_next_boundary() {
        let mut index = FileRangeIndex::new();
        index.insert(
            0,
            FileRangeEntry {
                end: 100,
                epoch: 0,
                e_tag: None,
                version: None,
            },
        );
        index.insert(
            100,
            FileRangeEntry {
                end: 200,
                epoch: 0,
                e_tag: None,
                version: None,
            },
        );

        // From checkpoint 0, next boundary is at 0
        let result = index.find_next_boundary(0);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, 0);

        // From checkpoint 50, next boundary is at 100
        let result = index.find_next_boundary(50);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, 100);

        // From checkpoint 200, no more boundaries
        let result = index.find_next_boundary(200);
        assert!(result.is_none());
    }

    #[test]
    fn test_snap_to_boundary() {
        let mut index = FileRangeIndex::new();
        // Files: 0-100, 200-300 (gap at 100-200)
        index.insert(
            0,
            FileRangeEntry {
                end: 100,
                epoch: 0,
                e_tag: None,
                version: None,
            },
        );
        index.insert(
            200,
            FileRangeEntry {
                end: 300,
                epoch: 1,
                e_tag: None,
                version: None,
            },
        );

        // Checkpoint inside first file → snap to file start
        assert_eq!(index.snap_to_boundary(50), Some(0));
        assert_eq!(index.snap_to_boundary(0), Some(0));
        assert_eq!(index.snap_to_boundary(99), Some(0));

        // Checkpoint in gap → snap to next file start
        assert_eq!(index.snap_to_boundary(100), Some(200));
        assert_eq!(index.snap_to_boundary(150), Some(200));
        assert_eq!(index.snap_to_boundary(199), Some(200));

        // Checkpoint inside second file → snap to file start
        assert_eq!(index.snap_to_boundary(200), Some(200));
        assert_eq!(index.snap_to_boundary(250), Some(200));
        assert_eq!(index.snap_to_boundary(299), Some(200));

        // Checkpoint beyond all files → None (error case)
        assert_eq!(index.snap_to_boundary(300), None);
        assert_eq!(index.snap_to_boundary(1000), None);
    }
}
