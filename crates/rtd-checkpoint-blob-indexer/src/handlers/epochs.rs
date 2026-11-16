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
        let path = ObjectPath::from(format!("_metadata/epochs/{checkpoint_num:020}.json"));
        let store = conn.object_store();
        match store.head(&path).await {
            Ok(_) => return Ok(0),
            Err(ObjectStoreError::NotFound { .. }) => {}
            Err(error) => return Err(error.into()),
        }
        let body = format!("{{\"checkpoint\":{checkpoint_num}}}");
        store.put(&path, Bytes::from(body).into()).await?;
        Ok(1)
    }
}
