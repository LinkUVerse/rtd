// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use bytes::Bytes;
use object_store::Error as ObjectStoreError;
use object_store::ObjectStoreExt as _;
use object_store::path::Path as ObjectPath;
use rtd_indexer_alt_framework::pipeline::Processor;
use rtd_indexer_alt_framework::pipeline::concurrent::BatchStatus;
use rtd_indexer_alt_framework::pipeline::concurrent::Handler;
use rtd_indexer_alt_framework::store::Store;
use rtd_indexer_alt_object_store::ObjectStore;
use rtd_types::full_checkpoint_content::Checkpoint;

pub struct EpochsPipeline;

pub struct EpochCheckpoint {
    pub checkpoint_number: u64,
}

#[async_trait::async_trait]
impl Processor for EpochsPipeline {
    const NAME: &'static str = "epochs";
    type Value = EpochCheckpoint;

    async fn process(&self, checkpoint: &Arc<Checkpoint>) -> anyhow::Result<Vec<Self::Value>> {
        if checkpoint.summary.is_last_checkpoint_of_epoch() {
            Ok(vec![EpochCheckpoint {
                checkpoint_number: checkpoint.summary.sequence_number,
            }])
        } else {
            Ok(vec![])
        }
    }
}

#[async_trait::async_trait]
impl Handler for EpochsPipeline {
    type Store = ObjectStore;
    type Batch = Option<Self::Value>;

    fn batch(
        &self,
        batch: &mut Self::Batch,
        values: &mut std::vec::IntoIter<Self::Value>,
    ) -> BatchStatus {
        if batch.is_none() && values.len() > 0 {
            *batch = values.next();
            BatchStatus::Ready
        } else {
            BatchStatus::Pending
        }
    }

    async fn commit<'a>(
        &self,
        batch: &Self::Batch,
        conn: &mut <Self::Store as Store>::Connection<'a>,
    ) -> anyhow::Result<usize> {
        let Some(epoch_checkpoint) = batch else {
            return Ok(0);
        };

        let checkpoint_num = epoch_checkpoint.checkpoint_number;
        let marker_path = ObjectPath::from(format!("_metadata/epochs/{checkpoint_num:020}.json"));
        let epochs_path = ObjectPath::from("epochs.json");
        let store = conn.object_store();

        let mut changed = false;
        let mut epochs: Vec<u64> = match store.get(&epochs_path).await {
            Ok(result) => {
                let bytes = result.bytes().await?;
                serde_json::from_slice(&bytes).map_err(|error| {
                    anyhow::anyhow!("invalid epochs.json in checkpoint store: {error}")
                })?
            }
            Err(ObjectStoreError::NotFound { .. }) => Vec::new(),
            Err(error) => return Err(error.into()),
        };
        match epochs.binary_search(&checkpoint_num) {
            Ok(_) => {}
            Err(position) => {
                epochs.insert(position, checkpoint_num);
                let body = serde_json::to_vec(&epochs)?;
                store.put(&epochs_path, Bytes::from(body).into()).await?;
                changed = true;
            }
        }

        match store.head(&marker_path).await {
            Ok(_) => {}
            Err(ObjectStoreError::NotFound { .. }) => {
                let body = format!("{{\"checkpoint\":{checkpoint_num}}}");
                store.put(&marker_path, Bytes::from(body).into()).await?;
                changed = true;
            }
            Err(error) => return Err(error.into()),
        }

        Ok(usize::from(changed))
    }
}
