// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Live mode store - derives watermarks from file names.

use std::collections::HashMap;
use std::sync::Arc;

use object_store::ObjectStore;
use object_store::path::Path as ObjectPath;
use sui_indexer_alt_framework_store_traits::CommitterWatermark;
use sui_storage::object_store::util::{
    find_all_dirs_with_epoch_prefix, find_all_files_with_epoch_prefix,
};
use tracing::info;

use crate::config::{BatchSizeConfig, PipelineConfig};
use crate::handlers::CheckpointRows;

use super::Batch;

/// Configuration for live mode store.
#[derive(Clone, Default)]
pub struct LiveStoreConfig {
    /// Pipeline configurations, keyed by pipeline name.
    pub pipelines: HashMap<String, PipelineConfig>,
}

/// Live mode - derives watermarks from file names.
///
/// Used for normal streaming ingestion where files are written with checkpoint
/// ranges in their names, and watermarks are derived from those file names.
#[derive(Clone)]
pub struct LiveStore {
    object_store: Arc<dyn ObjectStore>,
    config: LiveStoreConfig,
}

impl LiveStore {
    /// Create a new live store with the given configuration.
    pub fn new(object_store: Arc<dyn ObjectStore>, config: LiveStoreConfig) -> Self {
        Self {
            object_store,
            config,
        }
    }

    /// Determine the watermark by scanning file names in the object store.
    ///
    /// 1. Find epoch directories under `{pipeline}/epoch_*`
    /// 2. Get the latest epoch (max epoch number)
    /// 3. Find files in that epoch and extract checkpoint ranges from file names
    /// 4. Return the maximum `end` value as the watermark
    pub(crate) async fn committer_watermark(
        &self,
        pipeline: &str,
    ) -> anyhow::Result<Option<CommitterWatermark>> {
        let prefix = ObjectPath::from(pipeline);
        let epoch_dirs = find_all_dirs_with_epoch_prefix(&self.object_store, Some(&prefix)).await?;

        // Get latest epoch
        let Some((&epoch, epoch_path)) = epoch_dirs.last_key_value() else {
            return Ok(None); // No data yet
        };

        // Find files in latest epoch: {pipeline}/epoch_N/{start}_{end}.{format}
        let checkpoint_ranges =
            find_all_files_with_epoch_prefix(&self.object_store, Some(epoch_path)).await?;

        // Watermark = max(end) across all files
        let checkpoint_hi = checkpoint_ranges.iter().map(|r| r.end).max().unwrap_or(0);

        // Need checkpoint_hi - 1 since ranges are exclusive and watermark is inclusive
        if checkpoint_hi == 0 {
            return Ok(None);
        }

        info!(
            pipeline,
            epoch,
            checkpoint = checkpoint_hi - 1,
            "Determined watermark from bucket iteration"
        );

        Ok(Some(CommitterWatermark {
            epoch_hi_inclusive: epoch,
            checkpoint_hi_inclusive: checkpoint_hi - 1, // Convert exclusive end to inclusive
            tx_hi: 0,                                   // Not stored in filenames
            timestamp_ms_hi_inclusive: 0,               // Not stored in filenames
        }))
    }

    /// Write a file to the object store.
    pub(crate) async fn write_to_object_store(
        &self,
        path: &ObjectPath,
        payload: object_store::PutPayload,
    ) -> anyhow::Result<()> {
        self.object_store.put(path, payload).await?;
        Ok(())
    }

    /// Split a batch of checkpoints into files based on epoch boundaries and batch size.
    ///
    /// Cuts at:
    /// - Epoch boundaries (files don't span epochs)
    /// - Batch size thresholds (rows or checkpoint count)
    pub(crate) fn split_framework_batch_into_files(
        &self,
        pipeline: &str,
        rows_by_checkpoint: &[CheckpointRows],
        mut pending_batch: Batch,
    ) -> (Batch, Vec<Batch>) {
        let config = self
            .config
            .pipelines
            .get(pipeline)
            .expect("pipeline not configured");
        let batch_size = config
            .batch_size
            .as_ref()
            .expect("batch_size not configured for pipeline");

        let mut complete_batches: Vec<Batch> = Vec::new();

        for checkpoint_rows in rows_by_checkpoint {
            // Cut at epoch boundary
            if pending_batch
                .epoch()
                .is_some_and(|e| e != checkpoint_rows.epoch)
            {
                assert!(
                    !pending_batch.checkpoints_rows.is_empty(),
                    "invalid state: epoch set but rows empty"
                );
                complete_batches.push(pending_batch);
                pending_batch = Batch::default();
            }

            match *batch_size {
                BatchSizeConfig::Rows(n) => {
                    // Flush BEFORE adding so checkpoint rows stay together
                    if pending_batch.row_count() >= n && !pending_batch.checkpoints_rows.is_empty()
                    {
                        complete_batches.push(pending_batch);
                        pending_batch = Batch::default();
                    }
                    pending_batch.add(checkpoint_rows.clone());
                }
                BatchSizeConfig::Checkpoints(n) => {
                    pending_batch.add(checkpoint_rows.clone());
                    // Flush AFTER adding
                    if pending_batch.checkpoint_count() == n {
                        complete_batches.push(pending_batch);
                        pending_batch = Batch::default();
                    }
                }
            }
        }

        (pending_batch, complete_batches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use object_store::PutPayload;
    use object_store::memory::InMemory;
    use std::time::Duration;

    use crate::metrics::Metrics;
    use crate::store::AnalyticsStore;
    use sui_indexer_alt_framework::store::{Connection, Store};

    fn test_metrics() -> Metrics {
        Metrics::new(&prometheus::Registry::new())
    }

    async fn create_test_file(
        store: &Arc<dyn object_store::ObjectStore>,
        pipeline: &str,
        epoch: u64,
        start: u64,
        end: u64,
    ) {
        let path = ObjectPath::from(format!(
            "{}/epoch_{}/{}_{}.parquet",
            pipeline, epoch, start, end
        ));
        let payload: PutPayload = Bytes::from("test data").into();
        store.put(&path, payload).await.unwrap();
    }

    #[tokio::test]
    async fn test_committer_watermark_multiple_epochs() {
        let object_store: Arc<dyn object_store::ObjectStore> = Arc::new(InMemory::new());
        // Epoch 0
        create_test_file(&object_store, "test_pipeline", 0, 0, 100).await;
        create_test_file(&object_store, "test_pipeline", 0, 100, 200).await;
        // Epoch 1 (latest)
        create_test_file(&object_store, "test_pipeline", 1, 200, 300).await;
        create_test_file(&object_store, "test_pipeline", 1, 300, 400).await;

        let store = AnalyticsStore::new(
            object_store,
            LiveStoreConfig::default(),
            test_metrics(),
            10,
            Duration::from_secs(60),
        );
        let mut conn = store.connect().await.unwrap();

        let watermark = conn.committer_watermark("test_pipeline").await.unwrap();
        assert!(watermark.is_some());
        let watermark = watermark.unwrap();
        // Should use latest epoch (1) and max checkpoint from that epoch
        assert_eq!(watermark.epoch_hi_inclusive, 1);
        assert_eq!(watermark.checkpoint_hi_inclusive, 399); // max(300, 400) - 1
    }
}
