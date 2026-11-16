// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

pub mod config;
mod handlers;

pub use config::CommitterLayer;
pub use config::ConcurrentLayer;
pub use config::IndexerConfig;
pub use config::IngestionConfig;
pub use config::PipelineLayer;
pub use handlers::BcsCheckpoint;
pub use handlers::CheckpointBcsPipeline;
pub use handlers::CheckpointBlob;
pub use handlers::CheckpointBlobPipeline;
pub use handlers::EpochCheckpoint;
pub use handlers::EpochsPipeline;

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use object_store::ObjectStoreExt as _;
    use object_store::memory::InMemory;
    use object_store::path::Path as ObjectPath;
    use rtd_indexer_alt_framework::pipeline::concurrent::Handler;
    use rtd_indexer_alt_framework_store_traits::Store;
    use rtd_indexer_alt_object_store::ObjectStore;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_checkpoint_blob_handler_uncompressed() {
        let store = ObjectStore::new(Arc::new(InMemory::new()));
        let mut conn = store.connect().await.unwrap();

        let blob = CheckpointBlob {
            sequence_number: 100,
            proto_bytes: Bytes::from(vec![1, 2, 3, 4, 5]),
        };

        let pipeline = CheckpointBlobPipeline {
            compression_level: None,
        };
        let count = pipeline.commit(&Some(blob), &mut conn).await.unwrap();
        assert_eq!(count, 1);

        let path = ObjectPath::from("100.binpb");
        let result = conn.object_store().get(&path).await.unwrap();
        let bytes = result.bytes().await.unwrap();
        assert_eq!(bytes.as_ref(), &[1, 2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn test_checkpoint_blob_handler_compressed() {
        let store = ObjectStore::new(Arc::new(InMemory::new()));
        let mut conn = store.connect().await.unwrap();

        let test_data = vec![0u8; 1000];
        let blob = CheckpointBlob {
            sequence_number: 200,
            proto_bytes: Bytes::from(test_data.clone()),
        };

        let pipeline = CheckpointBlobPipeline {
            compression_level: Some(3),
        };
        let count = pipeline.commit(&Some(blob), &mut conn).await.unwrap();
        assert_eq!(count, 1);

        let path = ObjectPath::from("200.binpb.zst");
        let result = conn.object_store().get(&path).await.unwrap();
        let compressed_bytes = result.bytes().await.unwrap();

        let decompressed = zstd::decode_all(&compressed_bytes[..]).unwrap();
        assert_eq!(decompressed, test_data);
        assert!(compressed_bytes.len() < test_data.len());
    }

    #[tokio::test]
    async fn test_epochs_handler() {
        let store = ObjectStore::new(Arc::new(InMemory::new()));
        let marker = |n: u64| ObjectPath::from(format!("_metadata/epochs/{n:020}.json"));

        let mut conn = store.connect().await.unwrap();
        let pipeline = EpochsPipeline;

        for n in [100_u64, 200, 150] {
            let epoch = EpochCheckpoint {
                checkpoint_number: n,
            };
            let count = pipeline.commit(&Some(epoch), &mut conn).await.unwrap();
            assert_eq!(count, 1);
            let result = conn.object_store().get(&marker(n)).await.unwrap();
            let bytes = result.bytes().await.unwrap();
            let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(value["checkpoint"], n);
        }

        // Duplicate marker is idempotent and does not create another version.
        let epoch = EpochCheckpoint {
            checkpoint_number: 100,
        };
        let count = pipeline.commit(&Some(epoch), &mut conn).await.unwrap();
        assert_eq!(count, 0);
    }
}
