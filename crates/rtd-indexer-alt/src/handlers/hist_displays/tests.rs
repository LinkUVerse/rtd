// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use fastcrypto::hash::HashFunction;
use fastcrypto::hash::Sha256;
use move_core_types::ident_str;
use move_core_types::language_storage::StructTag;
use rtd_indexer_alt_framework::pipeline::Processor;
use rtd_indexer_alt_framework::pipeline::sequential::Handler;
use rtd_indexer_alt_framework::postgres::Db;
use rtd_indexer_alt_framework::postgres::DbArgs;
use rtd_indexer_alt_framework::postgres::temp::TempDb;
use rtd_indexer_alt_framework::store::CommitterWatermark;
use rtd_indexer_alt_framework::store::Connection;
use rtd_indexer_alt_framework::store::SequentialStore;
use rtd_indexer_alt_schema::MIGRATIONS;
use rtd_indexer_alt_schema::display_history::StoredDisplayCheckpoint;
use rtd_indexer_alt_schema::display_history::StoredDisplayCoverage;
use rtd_indexer_alt_schema::display_history::StoredHistoricalDisplay;
use rtd_indexer_alt_schema::schema::hist_display_checkpoints;
use rtd_indexer_alt_schema::schema::hist_display_coverage;
use rtd_indexer_alt_schema::schema::hist_displays;
use rtd_types::base_types::ObjectID;
use rtd_types::collection_types::VecMap;
use rtd_types::digests::CheckpointDigest;
use rtd_types::display::DisplayCreatedEvent;
use rtd_types::display::DisplayVersionUpdatedEvent;
use rtd_types::event::Event;
use rtd_types::full_checkpoint_content::Checkpoint;
use rtd_types::id::ID;
use rtd_types::test_checkpoint_data_builder::TestCheckpointBuilder;
use scoped_futures::ScopedFutureExt;
use serde::Serialize;
use tokio::sync::Notify;

use crate::writer_fence;

use super::DisplayCheckpoint;
use super::DisplayHistoryBinding;
use super::HistDisplays;

#[derive(Serialize)]
struct Release {
    id: ID,
    version: u16,
    fields: VecMap<String, String>,
}

fn handler(h0: &[u8]) -> HistDisplays {
    HistDisplays::new(
        DisplayHistoryBinding {
            expected_genesis_digest: CheckpointDigest::new(h0.try_into().unwrap()).to_string(),
            source_id: "certified-test-source".into(),
            generation: "preserved-test-generation".into(),
        },
        h0,
    )
    .unwrap()
}

fn h_binding() -> DisplayHistoryBinding {
    handler(&[1; 32]).binding
}

fn event(version: u16, id: ObjectID) -> Event {
    let object_type: StructTag = "0x42::nft::NFT".parse().unwrap();
    Event::new(
        &ObjectID::ZERO,
        ident_str!("display"),
        TestCheckpointBuilder::derive_address(0),
        DisplayVersionUpdatedEvent::type_(&object_type),
        bcs::to_bytes(&Release {
            id: ID::new(id),
            version,
            fields: VecMap { contents: vec![] },
        })
        .unwrap(),
    )
}

fn checkpoint(events: Vec<Event>) -> Checkpoint {
    TestCheckpointBuilder::new(0)
        .start_transaction(0)
        .with_events(events)
        .finish_transaction()
        .build_checkpoint()
}

fn value(cp: i64, versions: &[i32]) -> DisplayCheckpoint {
    let displays: Vec<_> = versions
        .iter()
        .enumerate()
        .map(|(i, version)| StoredHistoricalDisplay {
            object_type: vec![42],
            cp_sequence_number: cp,
            checkpoint_tx_index: 0,
            event_index: i as i64,
            transaction_digest: vec![40; 32],
            display_id: vec![10 + i as u8; 32],
            display_version: *version,
            display: vec![*version as u8],
        })
        .collect();
    DisplayCheckpoint {
        marker: StoredDisplayCheckpoint {
            cp_sequence_number: cp,
            checkpoint_digest: vec![cp as u8 + 1; 32],
            previous_digest: (cp > 0).then(|| vec![cp as u8; 32]),
            event_count: versions.len() as i64,
            events_sha256: Sha256::digest(bcs::to_bytes(&displays).unwrap())
                .as_ref()
                .to_vec(),
        },
        displays,
    }
}

async fn setup() -> (TempDb, Db, Db, HistDisplays) {
    let temp = TempDb::new().unwrap();
    let admin = Db::for_write(temp.database().url().clone(), DbArgs::default())
        .await
        .unwrap();
    admin.run_migrations(Some(&MIGRATIONS)).await.unwrap();
    let epoch = writer_fence::advance_epoch(&mut admin.connect().await.unwrap())
        .await
        .unwrap();
    let writer = Db::for_write(
        temp.database().url().clone(),
        DbArgs {
            writer_epoch: Some(epoch),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    writer_fence::install(&writer).await.unwrap();
    writer
        .connect()
        .await
        .unwrap()
        .init_watermark(HistDisplays::NAME, None)
        .await
        .unwrap();
    (temp, admin, writer, handler(&[1; 32]))
}

async fn commit(
    writer: &Db,
    handler: &HistDisplays,
    values: Vec<DisplayCheckpoint>,
) -> Result<usize> {
    let cp = values.last().unwrap().marker.cp_sequence_number as u64;
    let handler = handler.clone();
    writer
        .transaction(|conn| {
            async move {
                conn.set_committer_watermark(
                    HistDisplays::NAME,
                    CommitterWatermark::new_for_testing(cp),
                )
                .await?;
                handler.commit(&values, conn).await
            }
            .scope_boxed()
        })
        .await
}

async fn coverage(writer: &Db) -> StoredDisplayCoverage {
    hist_display_coverage::table
        .first(&mut writer.connect().await.unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn processor_preserves_all_positions_ids_and_u16_versions() {
    let old_id = ObjectID::random();
    let new_id = ObjectID::random();
    let mut builder = TestCheckpointBuilder::new(0)
        .start_transaction(0)
        .with_events(vec![event(32767, old_id), event(32768, old_id)])
        .finish_transaction()
        .start_transaction(1)
        .with_events(vec![event(1, new_id), event(65535, new_id)])
        .finish_transaction();
    let cp = Arc::new(builder.build_checkpoint());
    let h = handler(cp.summary.digest().as_ref());
    let values = h.process(&cp).await.unwrap();
    let rows = &values[0].displays;
    assert_eq!(
        rows.iter()
            .map(|r| (r.checkpoint_tx_index, r.event_index, r.display_version))
            .collect::<Vec<_>>(),
        vec![(0, 0, 32767), (0, 1, 32768), (1, 0, 1), (1, 1, 65535)]
    );
    assert_eq!(rows[0].display_id, old_id.into_bytes().to_vec());
    assert_eq!(rows[2].display_id, new_id.into_bytes().to_vec());
    assert_eq!(values[0].marker.event_count, 4);
    assert_eq!(
        rows[3].display,
        cp.transactions[1].events.as_ref().unwrap().data[1].contents
    );
}

#[tokio::test]
async fn processor_rejects_malformed_publication_and_changed_events() {
    let mut malformed = event(1, ObjectID::random());
    malformed.contents = vec![0xff];
    let cp = Arc::new(checkpoint(vec![malformed]));
    assert!(
        handler(cp.summary.digest().as_ref())
            .process(&cp)
            .await
            .is_err()
    );
    let mut cp = checkpoint(vec![event(1, ObjectID::random())]);
    cp.transactions[0].events.as_mut().unwrap().data[0]
        .contents
        .push(0);
    assert!(
        handler(cp.summary.digest().as_ref())
            .process(&Arc::new(cp))
            .await
            .is_err()
    );
}

#[tokio::test]
async fn empty_checkpoints_advance_coverage_and_restart_from_durable_boundary() {
    let (_temp, _admin, writer, h) = setup().await;
    assert_eq!(
        commit(&writer, &h, vec![value(0, &[]), value(1, &[])])
            .await
            .unwrap(),
        0
    );
    assert_eq!(coverage(&writer).await.checkpoint_hi, 1);
    assert_eq!(
        hist_display_checkpoints::table
            .count()
            .get_result::<i64>(&mut writer.connect().await.unwrap())
            .await
            .unwrap(),
        2
    );
    h.validate_resume(&writer).await.unwrap();
    let reopened = Db::for_write(
        _temp.database().url().clone(),
        DbArgs {
            writer_epoch: Some(1),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    h.validate_resume(&reopened).await.unwrap();
    commit(&reopened, &h, vec![value(2, &[65535])])
        .await
        .unwrap();
    assert_eq!(coverage(&reopened).await.checkpoint_hi, 2);
}

#[tokio::test]
async fn gap_or_wrong_predecessor_rolls_back_rows_and_watermark() {
    let (_temp, _admin, writer, h) = setup().await;
    commit(&writer, &h, vec![value(0, &[])]).await.unwrap();
    assert!(
        commit(&writer, &h, vec![value(1, &[1]), value(3, &[2])])
            .await
            .is_err()
    );
    assert_eq!(coverage(&writer).await.checkpoint_hi, 0);
    h.validate_resume(&writer).await.unwrap();
    let mut wrong = value(1, &[1]);
    wrong.marker.previous_digest = Some(vec![0; 32]);
    assert!(commit(&writer, &h, vec![wrong]).await.is_err());
    assert_eq!(
        hist_displays::table
            .count()
            .get_result::<i64>(&mut writer.connect().await.unwrap())
            .await
            .unwrap(),
        0
    );
}

#[tokio::test]
async fn identical_replay_succeeds_conflicting_bytes_or_event_set_fail() {
    let (_temp, _admin, writer, h) = setup().await;
    let values = vec![value(0, &[32767]), value(1, &[32768, 65535])];
    assert_eq!(commit(&writer, &h, values.clone()).await.unwrap(), 3);
    assert_eq!(commit(&writer, &h, values.clone()).await.unwrap(), 0);
    let mut conflict = values[1].clone();
    conflict.displays[0].display.push(99);
    conflict.marker.events_sha256 = Sha256::digest(bcs::to_bytes(&conflict.displays).unwrap())
        .as_ref()
        .to_vec();
    assert!(commit(&writer, &h, vec![conflict]).await.is_err());
    assert!(commit(&writer, &h, vec![value(1, &[])]).await.is_err());
    assert_eq!(coverage(&writer).await.checkpoint_hi, 1);
    assert_eq!(
        hist_displays::table
            .count()
            .get_result::<i64>(&mut writer.connect().await.unwrap())
            .await
            .unwrap(),
        3
    );
    h.validate_resume(&writer).await.unwrap();
}

#[tokio::test]
async fn wrong_binding_missing_genesis_and_unsigned_version_fail_closed() {
    let (_temp, _admin, writer, h) = setup().await;
    assert!(commit(&writer, &h, vec![value(1, &[])]).await.is_err());
    commit(&writer, &h, vec![value(0, &[])]).await.unwrap();
    let mut wrong = h.clone();
    wrong.binding.generation = "different".into();
    assert!(commit(&writer, &wrong, vec![value(1, &[])]).await.is_err());
    assert!(wrong.validate_resume(&writer).await.is_err());
    assert!(commit(&writer, &h, vec![value(1, &[65536])]).await.is_err());
    assert!(commit(&writer, &h, vec![value(1, &[-1])]).await.is_err());
    assert_eq!(coverage(&writer).await.checkpoint_hi, 0);
}

#[tokio::test]
async fn interruption_after_writes_rolls_back_all_history_and_resume_is_safe() {
    let (_temp, _admin, writer, h) = setup().await;
    commit(&writer, &h, vec![value(0, &[])]).await.unwrap();
    let batch = vec![value(1, &[5])];
    let interrupted_batch = batch.clone();
    let interrupted_handler = h.clone();
    let result: Result<()> = writer
        .transaction(|conn| {
            async move {
                conn.set_committer_watermark(
                    HistDisplays::NAME,
                    CommitterWatermark::new_for_testing(1),
                )
                .await?;
                interrupted_handler.commit(&interrupted_batch, conn).await?;
                anyhow::bail!("Injected interruption before transaction commit")
            }
            .scope_boxed()
        })
        .await;
    assert!(result.is_err());
    assert_eq!(coverage(&writer).await.checkpoint_hi, 0);
    h.validate_resume(&writer).await.unwrap();
    assert_eq!(commit(&writer, &h, batch).await.unwrap(), 1);
    assert_eq!(coverage(&writer).await.checkpoint_hi, 1);
}

#[tokio::test]
async fn database_rejects_mutation_and_truncate_even_with_current_epoch() {
    let (_temp, _admin, writer, h) = setup().await;
    commit(&writer, &h, vec![value(0, &[10])]).await.unwrap();
    for sql in [
        "UPDATE hist_displays SET display_version=11",
        "DELETE FROM hist_displays",
        "TRUNCATE hist_displays",
        "UPDATE hist_display_checkpoints SET event_count=0",
        "DELETE FROM hist_display_checkpoints",
        "TRUNCATE hist_display_checkpoints CASCADE",
    ] {
        assert!(
            diesel::sql_query(sql)
                .execute(&mut writer.connect().await.unwrap())
                .await
                .is_err(),
            "{sql}"
        );
    }
    assert_eq!(coverage(&writer).await.checkpoint_hi, 0);
}

#[tokio::test]
async fn paused_old_writer_is_rejected_after_epoch_promotion() {
    let (_temp, admin, writer, h) = setup().await;
    commit(&writer, &h, vec![value(0, &[])]).await.unwrap();
    let paused = Arc::new(Notify::new());
    let resume = Arc::new(Notify::new());
    let old_writer = writer.clone();
    let old_h = h.clone();
    let parked = paused.clone();
    let resumed = resume.clone();
    let old = tokio::spawn(async move {
        old_writer
            .transaction(|conn| {
                async move {
                    diesel::sql_query("SELECT 1").execute(conn).await?;
                    parked.notify_one();
                    resumed.notified().await;
                    old_h.commit(&vec![value(1, &[1])], conn).await
                }
                .scope_boxed()
            })
            .await
    });
    paused.notified().await;
    assert_eq!(
        writer_fence::advance_epoch(&mut admin.connect().await.unwrap())
            .await
            .unwrap(),
        2
    );
    resume.notify_one();
    assert!(old.await.unwrap().is_err());
    assert_eq!(coverage(&writer).await.checkpoint_hi, 0);
    for sql in [
        "INSERT INTO hist_display_checkpoints VALUES(1,decode(repeat('02',32),'hex'),decode(repeat('01',32),'hex'),0,decode(repeat('00',32),'hex'))",
        "UPDATE hist_display_coverage SET checkpoint_hi=0",
        "TRUNCATE hist_displays",
    ] {
        assert!(
            diesel::sql_query(sql)
                .execute(&mut writer.connect().await.unwrap())
                .await
                .is_err()
        );
    }
}

#[tokio::test]
async fn epoch_promotion_waits_for_admitted_transaction_to_commit() {
    let (_temp, admin, writer, h) = setup().await;
    let admitted = Arc::new(Notify::new());
    let resume = Arc::new(Notify::new());
    let parked = admitted.clone();
    let resumed = resume.clone();
    let old_writer = writer.clone();
    let old_h = h.clone();
    let old = tokio::spawn(async move {
        old_writer
            .transaction(|conn| {
                async move {
                    old_h.commit(&vec![value(0, &[1])], conn).await?;
                    parked.notify_one();
                    resumed.notified().await;
                    Ok(())
                }
                .scope_boxed()
            })
            .await
    });
    admitted.notified().await;
    let mut promotion = tokio::spawn(async move {
        writer_fence::advance_epoch(&mut admin.connect().await.unwrap()).await
    });
    assert!(
        tokio::time::timeout(Duration::from_millis(100), &mut promotion)
            .await
            .is_err()
    );
    resume.notify_one();
    old.await.unwrap().unwrap();
    assert_eq!(promotion.await.unwrap().unwrap(), 2);
    assert_eq!(coverage(&writer).await.checkpoint_hi, 0);
    assert!(commit(&writer, &h, vec![value(1, &[1])]).await.is_err());
}

#[tokio::test]
async fn coverage_cannot_change_identity_advance_without_rows_or_be_removed() {
    let (_temp, _admin, writer, h) = setup().await;
    commit(&writer, &h, vec![value(0, &[])]).await.unwrap();
    for sql in [
        "UPDATE hist_display_coverage SET checkpoint_hi=1",
        "UPDATE hist_display_coverage SET source_id='another-source'",
        "UPDATE hist_display_coverage SET generation='another-generation'",
        "UPDATE hist_display_coverage SET writer_epoch=2",
        "DELETE FROM hist_display_coverage",
        "TRUNCATE hist_display_coverage",
    ] {
        assert!(
            diesel::sql_query(sql)
                .execute(&mut writer.connect().await.unwrap())
                .await
                .is_err(),
            "{sql}"
        );
    }
    assert_eq!(coverage(&writer).await.checkpoint_hi, 0);
    h.validate_resume(&writer).await.unwrap();
}

#[tokio::test]
async fn missing_or_ahead_framework_watermark_cannot_resume_history() {
    let (_temp, _admin, writer, h) = setup().await;
    writer
        .connect()
        .await
        .unwrap()
        .set_committer_watermark(HistDisplays::NAME, CommitterWatermark::new_for_testing(10))
        .await
        .unwrap();
    assert!(h.validate_resume(&writer).await.is_err());
}

#[tokio::test]
async fn successor_commits_next_checkpoint_old_epoch_still_cannot_replay() {
    let (temp, admin, writer, h) = setup().await;
    commit(&writer, &h, vec![value(0, &[1])]).await.unwrap();
    let epoch = writer_fence::advance_epoch(&mut admin.connect().await.unwrap())
        .await
        .unwrap();
    let successor = Db::for_write(
        temp.database().url().clone(),
        DbArgs {
            writer_epoch: Some(epoch),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    successor
        .connect()
        .await
        .unwrap()
        .init_watermark(HistDisplays::NAME, None)
        .await
        .unwrap();
    assert_eq!(
        commit(&successor, &h, vec![value(1, &[32768])])
            .await
            .unwrap(),
        1
    );
    assert!(commit(&writer, &h, vec![value(1, &[32768])]).await.is_err());
    assert_eq!(
        commit(&successor, &h, vec![value(1, &[32768])])
            .await
            .unwrap(),
        0
    );
    assert_eq!(coverage(&successor).await.writer_epoch, 2);
    assert_eq!(coverage(&successor).await.checkpoint_hi, 1);
    h.validate_resume(&successor).await.unwrap();
}

#[tokio::test]
async fn pipeline_requires_exclusive_writer_published_source_and_binding_before_database_access() {
    use prometheus::Registry;
    use rtd_indexer_alt_framework::IndexerArgs;
    use rtd_indexer_alt_framework::ingestion::ClientArgs;
    use url::Url;

    use crate::config::DisplayHistoryLayer;
    use crate::config::IndexerConfig;

    for (epoch, published, binding, expected) in [
        (None, true, true, "exclusive fenced writer"),
        (Some(1), false, true, "published checkpoint source"),
        (Some(1), true, false, "explicit source binding"),
    ] {
        let mut config = IndexerConfig::default();
        config.pipeline.hist_displays = Some(DisplayHistoryLayer {
            binding: binding.then(|| h_binding()),
            ..Default::default()
        });
        let mut client = ClientArgs::default();
        client.ingestion.require_published_watermark = published;
        let result = crate::setup_indexer(
            Url::parse("postgres://unused@127.0.0.1:1/unused").unwrap(),
            DbArgs {
                writer_epoch: epoch,
                ..Default::default()
            },
            IndexerArgs::default(),
            client,
            config,
            None,
            &Registry::new(),
        )
        .await;
        let Err(error) = result else {
            panic!("Unexpected database connection");
        };
        assert!(error.to_string().contains(expected), "{error:#}");
    }
}

#[tokio::test]
async fn created_event_is_not_a_published_template() {
    let object_type: StructTag = "0x42::nft::NFT".parse().unwrap();
    let created = Event::new(
        &ObjectID::ZERO,
        ident_str!("display"),
        TestCheckpointBuilder::derive_address(0),
        DisplayCreatedEvent::type_(&object_type),
        bcs::to_bytes(&ID::new(ObjectID::random())).unwrap(),
    );
    let cp = Arc::new(checkpoint(vec![created]));
    let processed = handler(cp.summary.digest().as_ref())
        .process(&cp)
        .await
        .unwrap();
    assert!(processed[0].displays.is_empty());
    assert_eq!(processed[0].marker.event_count, 0);
}

#[tokio::test]
async fn checkpoint_above_postgres_bigint_is_rejected() {
    let mut cp = checkpoint(vec![]);
    cp.summary.data_mut_for_testing().sequence_number = i64::MAX as u64 + 1;
    let result = handler(&[1; 32]).process(&Arc::new(cp)).await;
    assert!(result.err().unwrap().to_string().contains("BIGINT"));
}

#[tokio::test]
async fn one_chain_event_position_cannot_claim_two_types() {
    let (_temp, _admin, writer, h) = setup().await;
    commit(&writer, &h, vec![value(0, &[])]).await.unwrap();
    let mut invalid = value(1, &[1, 2]);
    invalid.displays[1].object_type = vec![43];
    invalid.displays[1].event_index = 0;
    invalid.marker.events_sha256 = Sha256::digest(bcs::to_bytes(&invalid.displays).unwrap())
        .as_ref()
        .to_vec();
    assert!(commit(&writer, &h, vec![invalid]).await.is_err());
    assert_eq!(coverage(&writer).await.checkpoint_hi, 0);
    assert_eq!(
        hist_displays::table
            .count()
            .get_result::<i64>(&mut writer.connect().await.unwrap())
            .await
            .unwrap(),
        0
    );
    h.validate_resume(&writer).await.unwrap();
}
