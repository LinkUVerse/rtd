// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use anyhow::ensure;
use async_trait::async_trait;
use diesel::ExpressionMethods;
use diesel::OptionalExtension;
use diesel::QueryDsl;
use diesel::QueryableByName;
use diesel::sql_types::BigInt;
use diesel_async::RunQueryDsl;
use fastcrypto::hash::HashFunction;
use fastcrypto::hash::Sha256;
use rtd_indexer_alt_framework::FieldCount;
use rtd_indexer_alt_framework::pipeline::Processor;
use rtd_indexer_alt_framework::pipeline::sequential::Handler;
use rtd_indexer_alt_framework::postgres::Connection;
use rtd_indexer_alt_framework::postgres::Db;
use rtd_indexer_alt_schema::display_history::StoredDisplayCheckpoint;
use rtd_indexer_alt_schema::display_history::StoredDisplayCoverage;
use rtd_indexer_alt_schema::display_history::StoredHistoricalDisplay;
use rtd_indexer_alt_schema::schema::hist_display_checkpoints;
use rtd_indexer_alt_schema::schema::hist_display_coverage;
use rtd_indexer_alt_schema::schema::hist_displays;
use rtd_indexer_alt_schema::schema::watermarks;
use rtd_types::digests::CheckpointDigest;
use rtd_types::display::DisplayVersionUpdatedEvent;
use rtd_types::effects::TransactionEffectsAPI;
use rtd_types::full_checkpoint_content::Checkpoint;
use rtd_types::message_envelope::Message;
use serde::Deserialize;
use serde::Serialize;

#[cfg(test)]
mod tests;

const MAX_INSERT_CHUNK_ROWS: usize = i16::MAX as usize / StoredHistoricalDisplay::FIELD_COUNT;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct DisplayHistoryBinding {
    pub expected_genesis_digest: String,
    pub source_id: String,
    pub generation: String,
}

#[derive(Clone)]
pub(crate) struct HistDisplays {
    binding: DisplayHistoryBinding,
    genesis_digest: Vec<u8>,
}

#[derive(Clone)]
pub(crate) struct DisplayCheckpoint {
    marker: StoredDisplayCheckpoint,
    displays: Vec<StoredHistoricalDisplay>,
}

#[derive(QueryableByName)]
struct WriterEpoch {
    #[diesel(sql_type = BigInt)]
    epoch: i64,
}

impl HistDisplays {
    pub(crate) fn new(binding: DisplayHistoryBinding, genesis_digest: &[u8]) -> Result<Self> {
        let expected: CheckpointDigest = binding
            .expected_genesis_digest
            .parse()
            .context("Invalid Display history genesis digest")?;
        ensure!(
            expected.into_inner().as_slice() == genesis_digest,
            "Display history genesis differs"
        );
        for value in [&binding.source_id, &binding.generation] {
            ensure!(
                !value.is_empty() && value.len() <= 256 && !value.chars().any(char::is_control),
                "Invalid Display history source or generation"
            );
        }
        Ok(Self {
            binding,
            genesis_digest: genesis_digest.to_vec(),
        })
    }

    fn check_binding(&self, coverage: &StoredDisplayCoverage) -> Result<()> {
        ensure!(
            coverage.genesis_digest == self.genesis_digest
                && coverage.source_id == self.binding.source_id
                && coverage.generation == self.binding.generation
                && coverage.reader_lo == 0,
            "Display history coverage binding differs"
        );
        Ok(())
    }

    /// Refuse resuming a framework watermark that did not atomically commit our
    /// independent history coverage. In particular, an existing sum_displays
    /// watermark cannot bootstrap this pipeline at a nonzero checkpoint.
    pub(crate) async fn validate_resume(&self, db: &Db) -> Result<()> {
        let mut conn = db.connect().await?;
        let coverage: Option<StoredDisplayCoverage> = hist_display_coverage::table
            .first(&mut conn)
            .await
            .optional()?;
        let watermark: Option<i64> = watermarks::table
            .filter(watermarks::pipeline.eq(Self::NAME))
            .select(watermarks::checkpoint_hi_inclusive)
            .first(&mut conn)
            .await
            .optional()?;
        if let Some(coverage) = &coverage {
            self.check_binding(coverage)?;
        }
        ensure!(
            watermark.unwrap_or(-1) == coverage.as_ref().map_or(-1, |c| c.checkpoint_hi),
            "Display history watermark and coverage differ"
        );
        Ok(())
    }
}

#[async_trait]
impl Processor for HistDisplays {
    const NAME: &'static str = "hist_displays";
    type Value = DisplayCheckpoint;

    async fn process(&self, checkpoint: &Arc<Checkpoint>) -> Result<Vec<Self::Value>> {
        let cp = i64::try_from(checkpoint.summary.sequence_number)
            .context("Display checkpoint exceeds PostgreSQL BIGINT")?;
        ensure!(
            checkpoint.contents.digest() == &checkpoint.summary.content_digest,
            "Display checkpoint contents digest differs"
        );
        ensure!(
            checkpoint.contents.size() == checkpoint.transactions.len(),
            "Display checkpoint transaction count differs"
        );
        let mut displays = Vec::new();
        for (tx_index, (tx, digests)) in checkpoint
            .transactions
            .iter()
            .zip(checkpoint.contents.iter())
            .enumerate()
        {
            ensure!(
                tx.transaction.digest() == digests.transaction
                    && tx.effects.digest() == digests.effects
                    && tx.effects.transaction_digest() == &digests.transaction,
                "Display transaction or effects digest differs"
            );
            ensure!(
                tx.events.as_ref().map(|events| events.digest())
                    == tx.effects.events_digest().copied(),
                "Display transaction events digest differs"
            );
            let Some(events) = &tx.events else {
                continue;
            };
            for (event_index, event) in events.data.iter().enumerate() {
                if !DisplayVersionUpdatedEvent::is_display_updated_event(&event.type_) {
                    continue;
                }
                let (object_type, update) = DisplayVersionUpdatedEvent::try_from_event(event)
                    .context("Invalid published Display VersionUpdated event")?;
                displays.push(StoredHistoricalDisplay {
                    object_type: bcs::to_bytes(object_type)?,
                    cp_sequence_number: cp,
                    checkpoint_tx_index: i64::try_from(tx_index)?,
                    event_index: i64::try_from(event_index)?,
                    transaction_digest: digests.transaction.into_inner().to_vec(),
                    display_id: update.id.bytes.to_vec(),
                    display_version: i32::from(update.version),
                    display: event.contents.clone(),
                });
            }
        }
        let marker = StoredDisplayCheckpoint {
            cp_sequence_number: cp,
            checkpoint_digest: checkpoint.summary.digest().into_inner().to_vec(),
            previous_digest: checkpoint
                .summary
                .previous_digest
                .map(|d| d.into_inner().to_vec()),
            event_count: i64::try_from(displays.len())?,
            events_sha256: Sha256::digest(bcs::to_bytes(&displays)?).as_ref().to_vec(),
        };
        // Even a checkpoint without events has a durable, ordered marker.
        Ok(vec![DisplayCheckpoint { marker, displays }])
    }
}

#[async_trait]
impl Handler for HistDisplays {
    type Store = Db;
    type Batch = Vec<DisplayCheckpoint>;
    const MAX_BATCH_CHECKPOINTS: usize = 100;

    fn batch(&self, batch: &mut Self::Batch, values: std::vec::IntoIter<Self::Value>) {
        batch.extend(values);
    }

    /// The sequential framework encloses this operation and its watermark in
    /// one transaction. Epoch triggers hold the promotion lock until it ends.
    async fn commit<'a>(&self, batch: &Self::Batch, conn: &mut Connection<'a>) -> Result<usize> {
        if batch.is_empty() {
            return Ok(0);
        }
        let epoch: WriterEpoch = diesel::sql_query(
            "SELECT NULLIF(current_setting('rtd.alt_writer_epoch', true), '')::BIGINT AS epoch",
        )
        .get_result(conn)
        .await
        .context("Display history requires a fenced writer epoch")?;
        ensure!(epoch.epoch > 0, "Invalid Display history writer epoch");
        let initial = StoredDisplayCoverage {
            id: 1,
            genesis_digest: self.genesis_digest.clone(),
            source_id: self.binding.source_id.clone(),
            generation: self.binding.generation.clone(),
            reader_lo: 0,
            checkpoint_hi: -1,
            checkpoint_digest: None,
            writer_epoch: epoch.epoch,
        };
        diesel::insert_into(hist_display_coverage::table)
            .values(&initial)
            .on_conflict_do_nothing()
            .execute(conn)
            .await?;
        let mut coverage: StoredDisplayCoverage = hist_display_coverage::table
            .for_update()
            .first(conn)
            .await?;
        self.check_binding(&coverage)?;
        let mut affected = 0;
        let mut previous_cp = None;
        for value in batch {
            let marker = &value.marker;
            let cp = marker.cp_sequence_number;
            ensure!(
                previous_cp.is_none_or(|previous| cp == previous + 1),
                "Display history batch is not consecutive"
            );
            previous_cp = Some(cp);
            ensure!(
                marker.event_count == i64::try_from(value.displays.len())?
                    && marker.events_sha256
                        == Sha256::digest(bcs::to_bytes(&value.displays)?).as_ref()
                    && value.displays.iter().all(|d| d.cp_sequence_number == cp),
                "Display checkpoint event batch differs"
            );
            if cp <= coverage.checkpoint_hi {
                let stored: StoredDisplayCheckpoint =
                    hist_display_checkpoints::table.find(cp).first(conn).await?;
                ensure!(&stored == marker, "Conflicting Display checkpoint replay");
                let stored: Vec<StoredHistoricalDisplay> = hist_displays::table
                    .filter(hist_displays::cp_sequence_number.eq(cp))
                    .order((
                        hist_displays::checkpoint_tx_index.asc(),
                        hist_displays::event_index.asc(),
                    ))
                    .load(conn)
                    .await?;
                ensure!(stored == value.displays, "Conflicting Display event replay");
                continue;
            }
            ensure!(
                cp == coverage
                    .checkpoint_hi
                    .checked_add(1)
                    .context("Display coverage overflow")?,
                "Display history checkpoint gap"
            );
            ensure!(
                marker.previous_digest == coverage.checkpoint_digest,
                "Display history checkpoint predecessor differs"
            );
            if cp == 0 {
                ensure!(
                    marker.checkpoint_digest == self.genesis_digest,
                    "Display history genesis checkpoint differs"
                );
            }
            diesel::insert_into(hist_display_checkpoints::table)
                .values(marker)
                .execute(conn)
                .await?;
            for chunk in value.displays.chunks(MAX_INSERT_CHUNK_ROWS) {
                // Old checkpoints were compared byte-for-byte above. New ones
                // must contain only new chain positions; never ignore conflicts.
                affected += diesel::insert_into(hist_displays::table)
                    .values(chunk)
                    .execute(conn)
                    .await?;
            }
            coverage.checkpoint_hi = cp;
            coverage.checkpoint_digest = Some(marker.checkpoint_digest.clone());
        }
        diesel::update(hist_display_coverage::table.find(1))
            .set((
                hist_display_coverage::checkpoint_hi.eq(coverage.checkpoint_hi),
                hist_display_coverage::checkpoint_digest.eq(coverage.checkpoint_digest),
                hist_display_coverage::writer_epoch.eq(epoch.epoch),
            ))
            .execute(conn)
            .await?;
        Ok(affected)
    }
}
