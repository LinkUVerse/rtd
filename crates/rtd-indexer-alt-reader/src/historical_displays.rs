// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use anyhow::Context as _;
use anyhow::ensure;
use async_graphql::dataloader::Loader;
use diesel::QueryableByName;
use diesel::sql_types::Array;
use diesel::sql_types::BigInt;
use diesel::sql_types::Bytea;
use diesel::sql_types::Integer;
use diesel::sql_types::Nullable;
use diesel::sql_types::Text;
use move_core_types::language_storage::StructTag;
use rtd_indexer_alt_schema::display_history::StoredDisplayCoverage;
use rtd_indexer_alt_schema::display_history::StoredHistoricalDisplay;
use rtd_types::display::DisplayVersionUpdatedEvent;

use crate::error::Error;
use crate::pg_reader::PgReader;

#[cfg(test)]
mod tests;

// Coverage, the pipeline watermark, and the selected event must come from one MVCC snapshot.
// A LEFT JOIN keeps uncovered and covered-but-absent requests distinguishable. The closed type
// range preserves the leading index order, including for rare, absent, and hot types.
const QUERY: &str = r#"
    SELECT k.ordinal::BIGINT AS ordinal,
           c.genesis_digest, c.source_id, c.generation, c.reader_lo, c.checkpoint_hi,
           c.checkpoint_digest, c.writer_epoch,
           g.genesis_digest AS stored_genesis,
           w.chain_id AS watermark_chain_id, w.reader_lo AS watermark_lo,
           w.checkpoint_hi_inclusive AS watermark_hi,
           z.checkpoint_digest AS genesis_marker,
           h.checkpoint_digest AS high_marker,
           s.checkpoint_digest AS requested_marker,
           v.object_type, v.cp_sequence_number, v.checkpoint_tx_index, v.event_index,
           v.transaction_digest, v.display_id, v.display_version, v.display,
           e.checkpoint_digest AS event_checkpoint_digest
    FROM UNNEST($1::BYTEA[], $2::BIGINT[]) WITH ORDINALITY k(object_type, checkpoint, ordinal)
    LEFT JOIN hist_display_coverage c ON c.id = 1
    LEFT JOIN kv_genesis g ON g.genesis_digest = c.genesis_digest
    LEFT JOIN watermarks w ON w.pipeline = 'hist_displays'
    LEFT JOIN hist_display_checkpoints z ON z.cp_sequence_number = 0
    LEFT JOIN hist_display_checkpoints h ON h.cp_sequence_number = c.checkpoint_hi
    LEFT JOIN hist_display_checkpoints s ON s.cp_sequence_number = k.checkpoint
    LEFT JOIN LATERAL (
        SELECT object_type, cp_sequence_number, checkpoint_tx_index, event_index,
               transaction_digest, display_id, display_version, display
        FROM hist_displays
        WHERE object_type >= k.object_type AND object_type <= k.object_type
          AND cp_sequence_number <= k.checkpoint
        ORDER BY object_type ASC, cp_sequence_number DESC, checkpoint_tx_index DESC, event_index DESC
        LIMIT 1
    ) v ON TRUE
    LEFT JOIN hist_display_checkpoints e ON e.cp_sequence_number = v.cp_sequence_number
"#;

/// Fetch the last published v1 template for a type at an inclusive checkpoint.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HistoricalDisplayKey(pub StructTag, pub u64);

/// Check history coverage without requesting a type's publication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DisplayHistoryCoverageKey(pub u64);

/// `published == None` is only returned after continuous coverage has been checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoricalDisplay {
    pub coverage: StoredDisplayCoverage,
    pub snapshot_checkpoint_digest: Vec<u8>,
    pub published: Option<StoredHistoricalDisplay>,
    pub event_checkpoint_digest: Option<Vec<u8>>,
}

#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type = BigInt)]
    ordinal: i64,
    #[diesel(sql_type = Nullable<Bytea>)]
    genesis_digest: Option<Vec<u8>>,
    #[diesel(sql_type = Nullable<Text>)]
    source_id: Option<String>,
    #[diesel(sql_type = Nullable<Text>)]
    generation: Option<String>,
    #[diesel(sql_type = Nullable<BigInt>)]
    reader_lo: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    checkpoint_hi: Option<i64>,
    #[diesel(sql_type = Nullable<Bytea>)]
    checkpoint_digest: Option<Vec<u8>>,
    #[diesel(sql_type = Nullable<BigInt>)]
    writer_epoch: Option<i64>,
    #[diesel(sql_type = Nullable<Bytea>)]
    stored_genesis: Option<Vec<u8>>,
    #[diesel(sql_type = Nullable<Bytea>)]
    watermark_chain_id: Option<Vec<u8>>,
    #[diesel(sql_type = Nullable<BigInt>)]
    watermark_lo: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    watermark_hi: Option<i64>,
    #[diesel(sql_type = Nullable<Bytea>)]
    genesis_marker: Option<Vec<u8>>,
    #[diesel(sql_type = Nullable<Bytea>)]
    high_marker: Option<Vec<u8>>,
    #[diesel(sql_type = Nullable<Bytea>)]
    requested_marker: Option<Vec<u8>>,
    #[diesel(sql_type = Nullable<Bytea>)]
    object_type: Option<Vec<u8>>,
    #[diesel(sql_type = Nullable<BigInt>)]
    cp_sequence_number: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    checkpoint_tx_index: Option<i64>,
    #[diesel(sql_type = Nullable<BigInt>)]
    event_index: Option<i64>,
    #[diesel(sql_type = Nullable<Bytea>)]
    transaction_digest: Option<Vec<u8>>,
    #[diesel(sql_type = Nullable<Bytea>)]
    display_id: Option<Vec<u8>>,
    #[diesel(sql_type = Nullable<Integer>)]
    display_version: Option<i32>,
    #[diesel(sql_type = Nullable<Bytea>)]
    display: Option<Vec<u8>>,
    #[diesel(sql_type = Nullable<Bytea>)]
    event_checkpoint_digest: Option<Vec<u8>>,
}

impl Row {
    fn checked(self, checkpoint: i64, type_bcs: &[u8]) -> anyhow::Result<HistoricalDisplay> {
        let h0 = self
            .genesis_digest
            .context("Display history coverage is unavailable")?;
        ensure!(
            h0.len() == 32
                && self.stored_genesis.as_ref() == Some(&h0)
                && self.watermark_chain_id.as_ref() == Some(&h0)
                && self.genesis_marker.as_ref() == Some(&h0),
            "Display history genesis binding differs"
        );
        let hi = self
            .checkpoint_hi
            .context("Display history upper bound is unavailable")?;
        ensure!(
            self.reader_lo == Some(0)
                && self.watermark_lo == Some(0)
                && self.watermark_hi == Some(hi)
                && hi >= checkpoint
                && self
                    .checkpoint_digest
                    .as_ref()
                    .is_some_and(|d| d.len() == 32)
                && self.high_marker == self.checkpoint_digest
                && self
                    .requested_marker
                    .as_ref()
                    .is_some_and(|d| d.len() == 32),
            "Display history does not continuously cover the requested checkpoint"
        );
        let source_id = self
            .source_id
            .context("Display history source is unavailable")?;
        let generation = self
            .generation
            .context("Display history generation is unavailable")?;
        for value in [&source_id, &generation] {
            ensure!(
                !value.is_empty() && value.len() <= 256 && !value.chars().any(char::is_control),
                "Display history binding is invalid"
            );
        }
        let writer_epoch = self
            .writer_epoch
            .context("Display history writer epoch is unavailable")?;
        // A newer writer does not invalidate an immutable, previously committed prefix.
        ensure!(writer_epoch > 0, "Display history writer epoch is invalid");
        let coverage = StoredDisplayCoverage {
            id: 1,
            genesis_digest: h0,
            source_id,
            generation,
            reader_lo: 0,
            checkpoint_hi: hi,
            checkpoint_digest: self.checkpoint_digest,
            writer_epoch,
        };
        let published = if let Some(cp) = self.cp_sequence_number {
            let stored = StoredHistoricalDisplay {
                object_type: self
                    .object_type
                    .context("Display history type is unavailable")?,
                cp_sequence_number: cp,
                checkpoint_tx_index: self
                    .checkpoint_tx_index
                    .context("Display transaction position is unavailable")?,
                event_index: self
                    .event_index
                    .context("Display event position is unavailable")?,
                transaction_digest: self
                    .transaction_digest
                    .context("Display transaction digest is unavailable")?,
                display_id: self
                    .display_id
                    .context("Display template ID is unavailable")?,
                display_version: self
                    .display_version
                    .context("Display version is unavailable")?,
                display: self.display.context("Display event BCS is unavailable")?,
            };
            ensure!(
                stored.object_type == type_bcs
                    && cp >= 0
                    && cp <= checkpoint
                    && stored.checkpoint_tx_index >= 0
                    && stored.event_index >= 0
                    && stored.transaction_digest.len() == 32
                    && self
                        .event_checkpoint_digest
                        .as_ref()
                        .is_some_and(|d| d.len() == 32),
                "Display history event position or identity differs"
            );
            let event: DisplayVersionUpdatedEvent = bcs::from_bytes(&stored.display)
                .context("Failed to deserialize historical DisplayVersionUpdatedEvent")?;
            ensure!(
                event.id.bytes.as_ref() == stored.display_id.as_slice()
                    && i32::from(event.version) == stored.display_version,
                "Display history event and stored template metadata differ"
            );
            Some(stored)
        } else {
            None
        };
        Ok(HistoricalDisplay {
            coverage,
            snapshot_checkpoint_digest: self
                .requested_marker
                .context("Display snapshot checkpoint digest is unavailable")?,
            published,
            event_checkpoint_digest: self.event_checkpoint_digest,
        })
    }
}

#[async_trait::async_trait]
impl Loader<DisplayHistoryCoverageKey> for PgReader {
    type Value = HistoricalDisplay;
    type Error = Error;

    async fn load(
        &self,
        keys: &[DisplayHistoryCoverageKey],
    ) -> Result<HashMap<DisplayHistoryCoverageKey, HistoricalDisplay>, Error> {
        Ok(async {
            if keys.is_empty() {
                return Ok(HashMap::new());
            }
            let checkpoints = keys
                .iter()
                .map(|k| i64::try_from(k.0))
                .collect::<Result<Vec<_>, _>>()
                .context("Display coverage checkpoint exceeds PostgreSQL BIGINT")?;
            // NULL type bounds disable publication lookups while retaining exactly the same
            // MVCC coverage checks. No fabricated Move type or missing template is used as proof.
            let types = vec![None::<Vec<u8>>; keys.len()];
            let rows: Vec<Row> = self
                .connect()
                .await?
                .results(
                    diesel::sql_query(QUERY)
                        .bind::<Array<Nullable<Bytea>>, _>(types)
                        .bind::<Array<BigInt>, _>(&checkpoints),
                )
                .await?;
            ensure!(
                rows.len() == keys.len(),
                "Display coverage batch cardinality differs"
            );
            let mut seen = vec![false; keys.len()];
            let mut result = HashMap::new();
            for row in rows {
                let index = usize::try_from(row.ordinal - 1)
                    .context("Invalid Display coverage batch ordinal")?;
                ensure!(
                    index < keys.len() && !seen[index],
                    "Duplicate or invalid Display coverage batch ordinal"
                );
                seen[index] = true;
                let value = row.checked(checkpoints[index], &[])?;
                ensure!(
                    value.published.is_none(),
                    "Coverage-only lookup returned a publication"
                );
                result.insert(keys[index], value);
            }
            Ok::<_, anyhow::Error>(result)
        }
        .await?)
    }
}

#[async_trait::async_trait]
impl Loader<HistoricalDisplayKey> for PgReader {
    type Value = HistoricalDisplay;
    type Error = Error;

    async fn load(
        &self,
        keys: &[HistoricalDisplayKey],
    ) -> Result<HashMap<HistoricalDisplayKey, HistoricalDisplay>, Error> {
        Ok(async {
            if keys.is_empty() {
                return Ok(HashMap::new());
            }
            let types = keys
                .iter()
                .map(|k| bcs::to_bytes(&k.0))
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to serialize historical Display types")?;
            let checkpoints = keys
                .iter()
                .map(|k| i64::try_from(k.1))
                .collect::<Result<Vec<_>, _>>()
                .context("Display checkpoint exceeds PostgreSQL BIGINT")?;
            let rows: Vec<Row> = self
                .connect()
                .await?
                .results(
                    diesel::sql_query(QUERY)
                        .bind::<Array<Bytea>, _>(&types)
                        .bind::<Array<BigInt>, _>(&checkpoints),
                )
                .await?;
            ensure!(
                rows.len() == keys.len(),
                "Display history batch cardinality differs"
            );
            let mut result = HashMap::new();
            let mut seen = vec![false; keys.len()];
            for row in rows {
                let index =
                    usize::try_from(row.ordinal - 1).context("Invalid Display batch ordinal")?;
                ensure!(
                    index < keys.len() && !seen[index],
                    "Duplicate or invalid Display batch ordinal"
                );
                seen[index] = true;
                let display = row.checked(checkpoints[index], &types[index])?;
                result.insert(keys[index].clone(), display);
            }
            Ok::<_, anyhow::Error>(result)
        }
        .await?)
    }
}
