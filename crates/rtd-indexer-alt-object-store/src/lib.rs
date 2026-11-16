// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::Context;
use anyhow::bail;
use async_trait::async_trait;
use bytes::Bytes;
use futures::TryStreamExt;
use object_store::ObjectStoreExt as _;
use object_store::path::Path as ObjectPath;
use rtd_indexer_alt_framework_store_traits::CommitterWatermark;
use rtd_indexer_alt_framework_store_traits::ConcurrentConnection;
use rtd_indexer_alt_framework_store_traits::ConcurrentStore;
use rtd_indexer_alt_framework_store_traits::Connection;
use rtd_indexer_alt_framework_store_traits::InitWatermark;
use rtd_indexer_alt_framework_store_traits::PrunerWatermark;
use rtd_indexer_alt_framework_store_traits::ReaderWatermark;
use rtd_indexer_alt_framework_store_traits::Store;
use serde::Deserialize;
use serde::Serialize;

/// State is append-only: every watermark update writes a new marker object.
/// Readers list the pipeline prefix and merge fields with max, so concurrent
/// or stale writers cannot erase a higher watermark. No conditional writes.
#[derive(Clone)]
pub struct ObjectStore {
    object_store: Arc<dyn object_store::ObjectStore>,
    watermark_cache: Arc<Mutex<Option<ObjectStoreWatermark>>>,
}

pub struct ObjectStoreConnection {
    object_store: Arc<dyn object_store::ObjectStore>,
    watermark_cache: Arc<Mutex<Option<ObjectStoreWatermark>>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ObjectStoreWatermark {
    epoch_hi_inclusive: u64,
    checkpoint_hi_inclusive: Option<u64>,
    tx_hi: u64,
    timestamp_ms_hi_inclusive: u64,
    reader_lo: u64,
    pruner_hi: u64,
    pruner_timestamp_ms: u64,
}

#[derive(Clone, Copy, Debug)]
struct MarkerFields {
    checkpoint_hi: u64,
    epoch_hi: u64,
    tx_hi: u64,
    reader_lo: u64,
    pruner_hi: u64,
    pruner_timestamp_ms: u64,
}

impl ObjectStore {
    pub fn new(object_store: Arc<dyn object_store::ObjectStore>) -> Self {
        Self {
            object_store,
            watermark_cache: Arc::new(Mutex::new(None)),
        }
    }
}

impl ObjectStoreConnection {
    pub fn object_store(&self) -> Arc<dyn object_store::ObjectStore> {
        self.object_store.clone()
    }

    fn cached(&self) -> Option<ObjectStoreWatermark> {
        self.watermark_cache
            .lock()
            .expect("watermark cache mutex poisoned")
            .clone()
    }

    fn cache(&self, watermark: &ObjectStoreWatermark) {
        *self
            .watermark_cache
            .lock()
            .expect("watermark cache mutex poisoned") = Some(watermark.clone());
    }

    async fn merge_markers(&self, pipeline: &str) -> anyhow::Result<Option<ObjectStoreWatermark>> {
        let prefix = watermark_prefix(pipeline);
        let mut stream = self.object_store.list(Some(&prefix));
        let mut merged: Option<ObjectStoreWatermark> = None;
        while let Some(meta) = stream.try_next().await? {
            let fields = parse_marker_key(&meta.location, pipeline)?;
            let bytes = self.object_store.get(&meta.location).await?.bytes().await?;
            let marker: ObjectStoreWatermark = serde_json::from_slice(&bytes)
                .with_context(|| format!("Failed to parse watermark marker {}", meta.location))?;
            validate_marker(&marker, fields).with_context(|| {
                format!("Watermark marker fields differ from key {}", meta.location)
            })?;
            merged = Some(merge_watermark(merged, marker));
        }
        Ok(merged)
    }

    async fn current_watermark(&self, pipeline: &str) -> anyhow::Result<ObjectStoreWatermark> {
        if let Some(watermark) = self.cached() {
            return Ok(watermark);
        }
        let watermark = self
            .merge_markers(pipeline)
            .await?
            .unwrap_or_else(default_watermark_none);
        self.cache(&watermark);
        Ok(watermark)
    }

    async fn get_watermark_for_read(
        &self,
        pipeline: &str,
    ) -> anyhow::Result<Option<(ObjectStoreWatermark, u64)>> {
        let watermark = self.current_watermark(pipeline).await?;
        // Hide watermarks where `checkpoint_hi_inclusive < reader_lo`.
        let Some(checkpoint_hi_inclusive) = watermark
            .checkpoint_hi_inclusive
            .filter(|&cp| watermark.reader_lo <= cp)
        else {
            return Ok(None);
        };
        Ok(Some((watermark, checkpoint_hi_inclusive)))
    }

    async fn refresh_snapshot(
        &self,
        pipeline: &str,
        watermark: &ObjectStoreWatermark,
    ) -> anyhow::Result<()> {
        let json_bytes = serde_json::to_vec(watermark)?;
        self.object_store
            .put(
                &watermark_snapshot_path(pipeline),
                Bytes::from(json_bytes).into(),
            )
            .await?;
        Ok(())
    }

    async fn set_watermark(
        &self,
        pipeline: &str,
        watermark: ObjectStoreWatermark,
    ) -> anyhow::Result<()> {
        let object_path = watermark_marker_path(pipeline, &watermark);
        let json_bytes = serde_json::to_vec(&watermark)?;
        self.object_store
            .put(&object_path, Bytes::from(json_bytes.clone()).into())
            .await?;
        self.object_store
            .put(
                &watermark_snapshot_path(pipeline),
                Bytes::from(json_bytes).into(),
            )
            .await?;
        self.cache(&watermark);
        Ok(())
    }
}

#[async_trait]
impl ConcurrentStore for ObjectStore {
    type ConcurrentConnection<'c> = ObjectStoreConnection;
}

#[async_trait]
impl Store for ObjectStore {
    type Connection<'c> = ObjectStoreConnection;

    async fn connect<'c>(&'c self) -> anyhow::Result<Self::Connection<'c>> {
        Ok(ObjectStoreConnection {
            object_store: self.object_store.clone(),
            watermark_cache: self.watermark_cache.clone(),
        })
    }
}

#[async_trait]
impl Connection for ObjectStoreConnection {
    async fn init_watermark(
        &mut self,
        pipeline_task: &str,
        checkpoint_hi_inclusive: Option<u64>,
    ) -> anyhow::Result<Option<InitWatermark>> {
        if let Some(watermark) = self.merge_markers(pipeline_task).await? {
            self.cache(&watermark);
            self.refresh_snapshot(pipeline_task, &watermark).await?;
            return Ok(Some(InitWatermark {
                checkpoint_hi_inclusive: watermark.checkpoint_hi_inclusive,
                reader_lo: Some(watermark.reader_lo),
            }));
        }
        // Fresh pipeline: keep the initial watermark in memory only. The first
        // real update writes a marker; no create-if-absent is needed.
        let reader_lo = checkpoint_hi_inclusive.map_or(0, |cp| cp + 1);
        let watermark = default_watermark(checkpoint_hi_inclusive, reader_lo);
        self.set_watermark(pipeline_task, watermark.clone()).await?;
        Ok(Some(InitWatermark {
            checkpoint_hi_inclusive,
            reader_lo: Some(reader_lo),
        }))
    }

    async fn accepts_chain_id(
        &mut self,
        pipeline_task: &str,
        chain_id: [u8; 32],
    ) -> anyhow::Result<bool> {
        crate::accepts_chain_id(self.object_store.as_ref(), pipeline_task, chain_id).await
    }

    async fn committer_watermark(
        &mut self,
        pipeline_task: &str,
    ) -> anyhow::Result<Option<CommitterWatermark>> {
        Ok(self
            .get_watermark_for_read(pipeline_task)
            .await?
            .map(|(w, checkpoint_hi_inclusive)| CommitterWatermark {
                epoch_hi_inclusive: w.epoch_hi_inclusive,
                checkpoint_hi_inclusive,
                tx_hi: w.tx_hi,
                timestamp_ms_hi_inclusive: w.timestamp_ms_hi_inclusive,
            }))
    }

    async fn set_committer_watermark(
        &mut self,
        pipeline_task: &str,
        watermark: CommitterWatermark,
    ) -> anyhow::Result<bool> {
        let current_watermark = self.current_watermark(pipeline_task).await?;
        if current_watermark
            .checkpoint_hi_inclusive
            .is_some_and(|cp| cp >= watermark.checkpoint_hi_inclusive)
        {
            return Ok(false);
        }
        let new_watermark = ObjectStoreWatermark {
            epoch_hi_inclusive: watermark.epoch_hi_inclusive,
            checkpoint_hi_inclusive: Some(watermark.checkpoint_hi_inclusive),
            tx_hi: watermark.tx_hi,
            timestamp_ms_hi_inclusive: watermark.timestamp_ms_hi_inclusive,
            ..current_watermark
        };
        self.set_watermark(pipeline_task, new_watermark).await?;
        Ok(true)
    }
}

#[async_trait]
impl ConcurrentConnection for ObjectStoreConnection {
    async fn reader_watermark(
        &mut self,
        pipeline: &str,
    ) -> anyhow::Result<Option<ReaderWatermark>> {
        Ok(self
            .get_watermark_for_read(pipeline)
            .await?
            .map(|(w, checkpoint_hi_inclusive)| ReaderWatermark {
                checkpoint_hi_inclusive,
                reader_lo: w.reader_lo,
            }))
    }

    async fn pruner_watermark(
        &mut self,
        pipeline: &'static str,
        delay: Duration,
    ) -> anyhow::Result<Option<PrunerWatermark>> {
        let Some((watermark, _)) = self.get_watermark_for_read(pipeline).await? else {
            return Ok(None);
        };
        // Compute max(0, (pruner_timestamp + delay) - now). Use u128 to avoid overflow
        // when summing the two operands, and saturating_sub so we never underflow when
        // the wait period has already elapsed. saturating_sub is safe because the caller treats
        // anything < 1 the same.
        let pruner_ready_ms = (watermark.pruner_timestamp_ms as u128) + delay.as_millis();
        let now_ms = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let wait_for_ms = i64::try_from(pruner_ready_ms.saturating_sub(now_ms))?;
        Ok(Some(PrunerWatermark {
            wait_for_ms,
            reader_lo: watermark.reader_lo,
            pruner_hi: watermark.pruner_hi,
        }))
    }

    async fn set_reader_watermark(
        &mut self,
        pipeline: &'static str,
        reader_lo: u64,
    ) -> anyhow::Result<bool> {
        let current_watermark = self.current_watermark(pipeline).await?;
        if reader_lo <= current_watermark.reader_lo {
            return Ok(false);
        }
        let new_watermark = ObjectStoreWatermark {
            reader_lo,
            ..current_watermark
        };
        self.set_watermark(pipeline, new_watermark).await?;
        Ok(true)
    }

    async fn set_pruner_watermark(
        &mut self,
        pipeline: &'static str,
        pruner_hi: u64,
    ) -> anyhow::Result<bool> {
        let current_watermark = self.current_watermark(pipeline).await?;
        if pruner_hi <= current_watermark.pruner_hi {
            return Ok(false);
        }
        let new_watermark = ObjectStoreWatermark {
            pruner_hi,
            ..current_watermark
        };
        self.set_watermark(pipeline, new_watermark).await?;
        Ok(true)
    }
}

fn watermark_prefix(pipeline: &str) -> ObjectPath {
    ObjectPath::from(format!("_metadata/watermarks/{pipeline}/"))
}

/// Derived read cache for the publisher. Markers remain the source of truth;
/// this object is written after every marker so existing readers keep a single
/// fixed-version snapshot without any conditional write.
fn watermark_snapshot_path(pipeline: &str) -> ObjectPath {
    ObjectPath::from(format!("_metadata/watermarks/{pipeline}.json"))
}

fn watermark_marker_path(pipeline: &str, watermark: &ObjectStoreWatermark) -> ObjectPath {
    let checkpoint_hi = watermark.checkpoint_hi_inclusive.unwrap_or(0);
    ObjectPath::from(format!(
        "_metadata/watermarks/{pipeline}/{checkpoint_hi:020}.{:020}.{:020}.{:020}.{:020}.{:020}.{}.json",
        watermark.epoch_hi_inclusive,
        watermark.tx_hi,
        watermark.reader_lo,
        watermark.pruner_hi,
        watermark.pruner_timestamp_ms,
        marker_suffix(),
    ))
}

fn parse_marker_key(path: &ObjectPath, pipeline: &str) -> anyhow::Result<MarkerFields> {
    let prefix = format!("_metadata/watermarks/{pipeline}/");
    let raw = path.to_string();
    let name = raw
        .strip_prefix(&prefix)
        .context("watermark marker path prefix differs")?
        .strip_suffix(".json")
        .context("watermark marker lacks .json suffix")?;
    let parts: Vec<&str> = name.split('.').collect();
    if parts.len() != 7 {
        bail!(
            "watermark marker key has {} fields, expected 7",
            parts.len()
        );
    }
    let parse = |value: &str| -> anyhow::Result<u64> {
        value
            .parse::<u64>()
            .with_context(|| format!("invalid watermark marker field {value}"))
    };
    Ok(MarkerFields {
        checkpoint_hi: parse(parts[0])?,
        epoch_hi: parse(parts[1])?,
        tx_hi: parse(parts[2])?,
        reader_lo: parse(parts[3])?,
        pruner_hi: parse(parts[4])?,
        pruner_timestamp_ms: parse(parts[5])?,
    })
}

fn validate_marker(marker: &ObjectStoreWatermark, fields: MarkerFields) -> anyhow::Result<()> {
    if marker.checkpoint_hi_inclusive.unwrap_or(0) != fields.checkpoint_hi
        || marker.epoch_hi_inclusive != fields.epoch_hi
        || marker.tx_hi != fields.tx_hi
        || marker.reader_lo != fields.reader_lo
        || marker.pruner_hi != fields.pruner_hi
        || marker.pruner_timestamp_ms != fields.pruner_timestamp_ms
    {
        bail!("watermark marker content does not match its key");
    }
    Ok(())
}

fn merge_watermark(
    current: Option<ObjectStoreWatermark>,
    next: ObjectStoreWatermark,
) -> ObjectStoreWatermark {
    let Some(mut merged) = current else {
        return next;
    };
    let current_cp = merged
        .checkpoint_hi_inclusive
        .map(|value| value as i128)
        .unwrap_or(-1);
    let next_cp = next
        .checkpoint_hi_inclusive
        .map(|value| value as i128)
        .unwrap_or(-1);
    if next_cp > current_cp {
        merged.checkpoint_hi_inclusive = next.checkpoint_hi_inclusive;
        merged.tx_hi = next.tx_hi;
        merged.timestamp_ms_hi_inclusive = next.timestamp_ms_hi_inclusive;
    } else if next_cp == current_cp {
        merged.tx_hi = merged.tx_hi.max(next.tx_hi);
        merged.timestamp_ms_hi_inclusive = merged
            .timestamp_ms_hi_inclusive
            .max(next.timestamp_ms_hi_inclusive);
    }
    merged.epoch_hi_inclusive = merged.epoch_hi_inclusive.max(next.epoch_hi_inclusive);
    merged.reader_lo = merged.reader_lo.max(next.reader_lo);
    merged.pruner_hi = merged.pruner_hi.max(next.pruner_hi);
    merged.pruner_timestamp_ms = merged.pruner_timestamp_ms.max(next.pruner_timestamp_ms);
    merged
}

fn default_watermark(checkpoint_hi_inclusive: Option<u64>, reader_lo: u64) -> ObjectStoreWatermark {
    ObjectStoreWatermark {
        epoch_hi_inclusive: 0,
        checkpoint_hi_inclusive,
        tx_hi: 0,
        timestamp_ms_hi_inclusive: 0,
        reader_lo,
        pruner_hi: reader_lo,
        pruner_timestamp_ms: 0,
    }
}

fn default_watermark_none() -> ObjectStoreWatermark {
    default_watermark(None, 0)
}

static MARKER_SEQ: AtomicU64 = AtomicU64::new(0);

fn marker_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let seq = MARKER_SEQ.fetch_add(1, Ordering::Relaxed);
    format!("{:x}{:x}{:x}", std::process::id(), nanos, seq)
}

fn chain_id_prefix(pipeline_task: &str) -> ObjectPath {
    ObjectPath::from(format!("_metadata/chain_id/{pipeline_task}/"))
}

fn hex_chain_id(chain_id: &[u8; 32]) -> String {
    let mut value = String::with_capacity(64);
    for byte in chain_id {
        use std::fmt::Write;
        write!(value, "{byte:02x}").expect("write to String");
    }
    value
}

/// Reusable implementation of [`Connection::accepts_chain_id`] for object-store-backed
/// connections. Every chain id is an immutable marker under the pipeline prefix; a
/// different chain leaves a second marker and is rejected on the next list. No
/// create-if-absent is needed.
pub async fn accepts_chain_id(
    object_store: &dyn object_store::ObjectStore,
    pipeline_task: &str,
    chain_id: [u8; 32],
) -> anyhow::Result<bool> {
    let prefix = chain_id_prefix(pipeline_task);
    let expected = hex_chain_id(&chain_id);
    let prefix_text = prefix.to_string();
    let mut stream = object_store.list(Some(&prefix));
    let mut found = false;
    while let Some(meta) = stream.try_next().await? {
        found = true;
        let raw = meta.location.to_string();
        let name = raw
            .strip_prefix(&prefix_text)
            .unwrap_or_default()
            .trim_start_matches('/');
        if name != expected {
            return Ok(false);
        }
        let bytes = object_store.get(&meta.location).await?.bytes().await?;
        if bytes.as_ref() != chain_id.as_slice() {
            return Ok(false);
        }
    }
    if !found {
        let path = ObjectPath::from(format!("{prefix_text}/{expected}"));
        object_store
            .put(&path, Bytes::from(chain_id.to_vec()).into())
            .await?;
    }
    // Derived canonical object for readers that still check one key. Keep its
    // version stable across restarts: write only when absent or different.
    let canonical = ObjectPath::from(format!("_metadata/chain_id/{pipeline_task}"));
    let canonical_matches = match object_store.get(&canonical).await {
        Ok(result) => result.bytes().await?.as_ref() == chain_id.as_slice(),
        Err(object_store::Error::NotFound { .. }) => false,
        Err(error) => return Err(error.into()),
    };
    if !canonical_matches {
        object_store
            .put(&canonical, Bytes::from(chain_id.to_vec()).into())
            .await?;
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use object_store::memory::InMemory;
    use rtd_indexer_alt_framework_store_traits::concurrent_connection_tests;
    use rtd_indexer_alt_framework_store_traits::connection_tests;
    use rtd_indexer_alt_framework_store_traits::testing::Harness;

    use super::*;

    const PIPELINE: &str = "pipeline";

    /// One hour in milliseconds. Used in pruner watermark tests as both the expected
    /// `wait_for_ms` value and the offset for `delay_for_one_hour_wait`.
    const ONE_HOUR_MS: u64 = 3_600_000;

    // Canonical "non-default" watermark field values reused across tests so each one
    // doesn't need to invent its own. Distinct values let assertions catch field mix-ups.
    const EPOCH_HI: u64 = 7;
    const CHECKPOINT_HI: u64 = 200;
    const TX_HI: u64 = 42;
    const TIMESTAMP_MS_HI: u64 = 99;
    const READER_LO: u64 = 123;
    const PRUNER_HI: u64 = 77;
    const PRUNER_TIMESTAMP_MS: u64 = 555;

    struct ObjectStoreHarness {
        store: ObjectStore,
    }

    #[async_trait::async_trait(?Send)]
    impl Harness for ObjectStoreHarness {
        type Store = ObjectStore;

        async fn new() -> Self {
            Self {
                store: ObjectStore::new(Arc::new(InMemory::new())),
            }
        }

        fn store(&self) -> &Self::Store {
            &self.store
        }
    }

    async fn store_conn() -> ObjectStoreConnection {
        let harness = ObjectStoreHarness::new().await;
        harness.store.connect().await.unwrap()
    }

    async fn bootstrap(conn: &mut ObjectStoreConnection, checkpoint_hi_inclusive: u64) {
        conn.init_watermark(PIPELINE, None).await.unwrap();
        conn.set_committer_watermark(
            PIPELINE,
            CommitterWatermark {
                epoch_hi_inclusive: 0,
                checkpoint_hi_inclusive,
                tx_hi: 0,
                timestamp_ms_hi_inclusive: 0,
            },
        )
        .await
        .unwrap();
    }

    /// Build a delay such that `pruner_watermark` returns `wait_for_ms ~= 1h` for a
    /// watermark with `pruner_timestamp_ms = 0` (which is what `bootstrap` produces).
    /// `pruner_watermark` computes `(pruner_timestamp + delay) - now` saturating to 0,
    /// so the delay must put `pruner_timestamp + delay` past `now` for the wait to be
    /// non-zero; i.e. delay must be at least `now`.
    fn delay_for_one_hour_wait() -> Duration {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        Duration::from_millis(now_ms + ONE_HOUR_MS)
    }

    connection_tests!(ObjectStoreHarness);
    concurrent_connection_tests!(ObjectStoreHarness);

    #[tokio::test]
    async fn test_init_watermark_fresh_without_checkpoint_reader_lo() {
        let mut conn = store_conn().await;
        let watermark = conn.init_watermark(PIPELINE, None).await.unwrap().unwrap();
        assert_eq!(watermark.reader_lo, Some(0));
    }

    #[tokio::test]
    async fn test_init_watermark_fresh_with_checkpoint_reader_lo() {
        let mut conn = store_conn().await;
        let watermark = conn
            .init_watermark(PIPELINE, Some(CHECKPOINT_HI))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(watermark.reader_lo, Some(CHECKPOINT_HI + 1));
    }

    #[tokio::test]
    async fn test_init_watermark_returns_existing_reader_lo() {
        let mut conn = store_conn().await;
        conn.init_watermark(PIPELINE, None).await.unwrap();
        conn.set_reader_watermark(PIPELINE, READER_LO)
            .await
            .unwrap();

        let watermark = conn
            .init_watermark(PIPELINE, Some(0))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(watermark.reader_lo, Some(READER_LO));
    }

    async fn put_watermark_marker(
        conn: &ObjectStoreConnection,
        pipeline: &str,
        watermark: &ObjectStoreWatermark,
    ) {
        let path = watermark_marker_path(pipeline, watermark);
        let bytes = serde_json::to_vec(watermark).unwrap();
        conn.object_store
            .put(&path, Bytes::from(bytes).into())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_init_watermark_merges_markers() {
        let mut conn = store_conn().await;
        put_watermark_marker(
            &conn,
            PIPELINE,
            &ObjectStoreWatermark {
                epoch_hi_inclusive: 1,
                checkpoint_hi_inclusive: Some(100),
                tx_hi: 10,
                timestamp_ms_hi_inclusive: 20,
                reader_lo: 50,
                pruner_hi: 40,
                pruner_timestamp_ms: 30,
            },
        )
        .await;
        put_watermark_marker(
            &conn,
            PIPELINE,
            &ObjectStoreWatermark {
                epoch_hi_inclusive: EPOCH_HI,
                checkpoint_hi_inclusive: Some(CHECKPOINT_HI),
                tx_hi: TX_HI,
                timestamp_ms_hi_inclusive: TIMESTAMP_MS_HI,
                reader_lo: READER_LO,
                pruner_hi: PRUNER_HI,
                pruner_timestamp_ms: PRUNER_TIMESTAMP_MS,
            },
        )
        .await;

        let watermark = conn
            .init_watermark(PIPELINE, Some(0))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(watermark.checkpoint_hi_inclusive, Some(CHECKPOINT_HI));
        assert_eq!(watermark.reader_lo, Some(READER_LO));
    }

    #[tokio::test]
    async fn test_lower_marker_cannot_regress() {
        let mut conn = store_conn().await;
        put_watermark_marker(
            &conn,
            PIPELINE,
            &ObjectStoreWatermark {
                epoch_hi_inclusive: EPOCH_HI,
                checkpoint_hi_inclusive: Some(CHECKPOINT_HI),
                tx_hi: TX_HI,
                timestamp_ms_hi_inclusive: TIMESTAMP_MS_HI,
                reader_lo: READER_LO,
                pruner_hi: PRUNER_HI,
                pruner_timestamp_ms: PRUNER_TIMESTAMP_MS,
            },
        )
        .await;
        put_watermark_marker(
            &conn,
            PIPELINE,
            &ObjectStoreWatermark {
                epoch_hi_inclusive: 1,
                checkpoint_hi_inclusive: Some(1),
                tx_hi: 1,
                timestamp_ms_hi_inclusive: 1,
                reader_lo: 1,
                pruner_hi: 1,
                pruner_timestamp_ms: 1,
            },
        )
        .await;

        let watermark = conn
            .init_watermark(PIPELINE, Some(0))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(watermark.checkpoint_hi_inclusive, Some(CHECKPOINT_HI));
    }

    #[tokio::test]
    async fn test_marker_key_content_mismatch_fails() {
        let conn = store_conn().await;
        let watermark = ObjectStoreWatermark {
            epoch_hi_inclusive: 1,
            checkpoint_hi_inclusive: Some(1),
            tx_hi: 1,
            timestamp_ms_hi_inclusive: 1,
            reader_lo: 1,
            pruner_hi: 1,
            pruner_timestamp_ms: 1,
        };
        let mut path = watermark_marker_path(PIPELINE, &watermark).to_string();
        path = path.replace("/00000000000000000001.", "/00000000000000000002.");
        let bytes = serde_json::to_vec(&watermark).unwrap();
        conn.object_store
            .put(&ObjectPath::from(path), Bytes::from(bytes).into())
            .await
            .unwrap();
        let mut conn = conn;
        assert!(conn.init_watermark(PIPELINE, None).await.is_err());
    }

    #[tokio::test]
    async fn test_pruner_watermark_wait_for_ms() {
        let mut conn = store_conn().await;
        bootstrap(&mut conn, CHECKPOINT_HI).await;

        let watermark = conn
            .pruner_watermark(PIPELINE, delay_for_one_hour_wait())
            .await
            .unwrap()
            .unwrap();
        assert!(
            watermark.wait_for_ms > (ONE_HOUR_MS as i64 - 100_000)
                && watermark.wait_for_ms <= ONE_HOUR_MS as i64,
            "wait_for_ms = {}",
            watermark.wait_for_ms
        );
    }

    #[tokio::test]
    async fn test_pruner_watermark_saturates_when_ready() {
        let mut conn = store_conn().await;
        bootstrap(&mut conn, CHECKPOINT_HI).await;

        let watermark = conn
            .pruner_watermark(PIPELINE, Duration::ZERO)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(watermark.wait_for_ms, 0);
    }

    #[tokio::test]
    async fn test_pruner_watermark_overflow() {
        let conn = store_conn().await;
        put_watermark_marker(
            &conn,
            PIPELINE,
            &ObjectStoreWatermark {
                epoch_hi_inclusive: EPOCH_HI,
                checkpoint_hi_inclusive: Some(CHECKPOINT_HI),
                tx_hi: TX_HI,
                timestamp_ms_hi_inclusive: TIMESTAMP_MS_HI,
                reader_lo: 0,
                pruner_hi: 0,
                pruner_timestamp_ms: u64::MAX,
            },
        )
        .await;

        let mut conn = conn;
        let result = conn.pruner_watermark(PIPELINE, Duration::ZERO).await;
        assert!(result.is_err(), "expected overflow error, got {result:?}");
    }
}
