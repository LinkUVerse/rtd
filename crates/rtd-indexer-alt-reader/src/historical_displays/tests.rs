// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::time::Instant;

use async_graphql::dataloader::Loader;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::QueryableByName;
use diesel::sql_types::Array;
use diesel::sql_types::BigInt;
use diesel::sql_types::Bytea;
use diesel::sql_types::Text;
use diesel_async::RunQueryDsl;
use prometheus::Registry;
use rtd_indexer_alt_schema::MIGRATIONS;
use rtd_indexer_alt_schema::checkpoints::StoredGenesis;
use rtd_indexer_alt_schema::display_history::StoredDisplayCheckpoint;
use rtd_indexer_alt_schema::display_history::StoredDisplayCoverage;
use rtd_indexer_alt_schema::display_history::StoredHistoricalDisplay;
use rtd_indexer_alt_schema::schema::hist_display_checkpoints;
use rtd_indexer_alt_schema::schema::hist_display_coverage;
use rtd_indexer_alt_schema::schema::hist_displays;
use rtd_indexer_alt_schema::schema::kv_genesis;
use rtd_indexer_alt_schema::schema::watermarks;
use rtd_pg_db::Connection;
use rtd_pg_db::Db;
use rtd_pg_db::DbArgs;
use rtd_pg_db::temp::TempDb;
use rtd_types::base_types::ObjectID;
use rtd_types::collection_types::Entry;
use rtd_types::collection_types::VecMap;
use rtd_types::id::ID;
use serde::Serialize;

use super::DisplayHistoryCoverageKey as C;
use super::HistoricalDisplayKey as K;
use super::PgReader;
use super::QUERY;

#[derive(Serialize)]
struct Release {
    id: ID,
    version: u16,
    fields: VecMap<String, String>,
}

#[derive(QueryableByName)]
struct PlanLine {
    #[diesel(sql_type = Text, column_name = "QUERY PLAN")]
    line: String,
}

fn key(name: &str, cp: u64) -> K {
    K(format!("0x42::nft::{name}").parse().unwrap(), cp)
}

fn digest(cp: i64) -> Vec<u8> {
    let mut bytes = vec![0; 32];
    bytes[24..].copy_from_slice(&cp.to_be_bytes());
    bytes
}

fn row(name: &str, cp: i64, tx: i64, event: i64, version: u16, id: u8) -> StoredHistoricalDisplay {
    StoredHistoricalDisplay {
        object_type: bcs::to_bytes(&key(name, 0).0).unwrap(),
        cp_sequence_number: cp,
        checkpoint_tx_index: tx,
        event_index: event,
        transaction_digest: vec![10 + tx as u8; 32],
        display_id: vec![id; 32],
        display_version: i32::from(version),
        display: bcs::to_bytes(&Release {
            id: ID::new(ObjectID::from_bytes([id; 32]).unwrap()),
            version,
            fields: VecMap {
                contents: vec![Entry {
                    key: "name".into(),
                    value: format!("{name}-{cp}-{tx}-{event}"),
                }],
            },
        })
        .unwrap(),
    }
}

async fn setup() -> (TempDb, Db, PgReader) {
    let temp = TempDb::new().unwrap();
    let url = temp.database().url().clone();
    let writer = Db::for_write(url.clone(), DbArgs::default()).await.unwrap();
    writer.run_migrations(Some(&MIGRATIONS)).await.unwrap();
    let reader = PgReader::new(None, Some(url), DbArgs::default(), &Registry::new())
        .await
        .unwrap();
    let mut conn = writer.connect().await.unwrap();
    diesel::insert_into(kv_genesis::table)
        .values(StoredGenesis {
            genesis_digest: digest(0),
            initial_protocol_version: 1,
        })
        .execute(&mut conn)
        .await
        .unwrap();
    diesel::sql_query("INSERT INTO watermarks VALUES ('hist_displays',0,-1,0,0,0,NOW(),0,$1)")
        .bind::<Bytea, _>(digest(0))
        .execute(&mut conn)
        .await
        .unwrap();
    drop(conn);
    (temp, writer, reader)
}

// These are isolated PG fixtures, not certified checkpoint replay. The real schema guards
// validate continuous markers, predecessor digests, and complete event counts on publication.
async fn publish(conn: &mut Connection<'_>, first: i64, hi: i64, rows: &[StoredHistoricalDisplay]) {
    diesel::sql_query("BEGIN").execute(conn).await.unwrap();
    diesel::sql_query("SET LOCAL rtd.alt_writer_epoch = '1'")
        .execute(conn)
        .await
        .unwrap();
    let markers: Vec<_> = (first..=hi)
        .map(|cp| StoredDisplayCheckpoint {
            cp_sequence_number: cp,
            checkpoint_digest: digest(cp),
            previous_digest: (cp > 0).then(|| digest(cp - 1)),
            event_count: rows.iter().filter(|r| r.cp_sequence_number == cp).count() as i64,
            events_sha256: vec![99; 32],
        })
        .collect();
    diesel::insert_into(hist_display_checkpoints::table)
        .values(markers)
        .execute(conn)
        .await
        .unwrap();
    if !rows.is_empty() {
        diesel::insert_into(hist_displays::table)
            .values(rows)
            .execute(conn)
            .await
            .unwrap();
    }
    let coverage = StoredDisplayCoverage {
        id: 1,
        genesis_digest: digest(0),
        source_id: "certified-test-source".into(),
        generation: "test-generation".into(),
        reader_lo: 0,
        checkpoint_hi: hi,
        checkpoint_digest: Some(digest(hi)),
        writer_epoch: 1,
    };
    if first == 0 {
        diesel::insert_into(hist_display_coverage::table)
            .values(coverage)
            .execute(conn)
            .await
            .unwrap();
    } else {
        diesel::update(hist_display_coverage::table)
            .set((
                hist_display_coverage::checkpoint_hi.eq(hi),
                hist_display_coverage::checkpoint_digest.eq(Some(digest(hi))),
            ))
            .execute(conn)
            .await
            .unwrap();
    }
    diesel::update(watermarks::table.filter(watermarks::pipeline.eq("hist_displays")))
        .set(watermarks::checkpoint_hi_inclusive.eq(hi))
        .execute(conn)
        .await
        .unwrap();
}

async fn seed(writer: &Db, hi: i64, rows: &[StoredHistoricalDisplay]) {
    let mut conn = writer.connect().await.unwrap();
    publish(&mut conn, 0, hi, rows).await;
    diesel::sql_query("COMMIT")
        .execute(&mut conn)
        .await
        .unwrap();
}

#[tokio::test]
async fn coverage_only_batch_preserves_checkpoint_digests_without_templates() {
    let (_temp, writer, reader) = setup().await;
    seed(&writer, 3, &[row("NFT", 1, 0, 0, 1, 7)]).await;
    let keys = [C(0), C(1), C(3), C(1)];
    let values = Loader::load(&reader, &keys).await.unwrap();
    assert_eq!(values.len(), 3);
    for key in keys {
        let value = &values[&key];
        assert_eq!(value.published, None);
        assert_eq!(value.snapshot_checkpoint_digest, digest(key.0 as i64));
        assert_eq!(value.coverage.checkpoint_hi, 3);
    }
    assert!(Loader::load(&reader, &[C(1), C(4)]).await.is_err());
    assert!(Loader::load(&reader, &[C(u64::MAX)]).await.is_err());
    assert!(Loader::load(&reader, &[] as &[C]).await.unwrap().is_empty());
}

#[tokio::test]
async fn coverage_only_rejects_missing_history_and_watermark() {
    let (_temp, writer, reader) = setup().await;
    assert!(Loader::load(&reader, &[C(0)]).await.is_err());
    seed(&writer, 1, &[]).await;
    assert!(Loader::load(&reader, &[C(0)]).await.is_ok());
    diesel::sql_query("DELETE FROM watermarks")
        .execute(&mut writer.connect().await.unwrap())
        .await
        .unwrap();
    assert!(Loader::load(&reader, &[C(0)]).await.is_err());
}

#[tokio::test]
async fn checkpoint_order_multi_id_full_u16_missing_and_duplicate_keys() {
    let (_temp, writer, reader) = setup().await;
    let releases = vec![
        row("NFT", 1, 0, 0, 32767, 1),
        row("NFT", 1, 0, 1, 32768, 1),
        row("NFT", 1, 1, 0, 65535, 1),
        row("NFT", 2, 0, 0, 1, 2),
        row("Other", 2, 0, 1, 2, 3),
        row("NFT", 3, 0, 0, 2, 2),
    ];
    seed(&writer, 4, &releases).await;
    let keys = [
        key("NFT", 0),
        key("NFT", 1),
        key("NFT", 2),
        key("NFT", 4),
        key("Other", 1),
        key("Other", 2),
        key("Missing", 3),
        key("NFT", 2),
    ];
    let result = Loader::load(&reader, &keys).await.unwrap();
    assert_eq!(result.len(), 7);
    assert_eq!(result[&keys[0]].published, None);
    assert_eq!(result[&keys[1]].published, Some(releases[2].clone()));
    assert_eq!(result[&keys[2]].published, Some(releases[3].clone()));
    assert_eq!(result[&keys[3]].published, Some(releases[5].clone()));
    assert_eq!(result[&keys[4]].published, None);
    assert_eq!(result[&keys[5]].published, Some(releases[4].clone()));
    assert_eq!(result[&keys[6]].published, None);
    for value in result.values() {
        assert_eq!(value.coverage.checkpoint_hi, 4);
        assert_eq!(value.coverage.genesis_digest, digest(0));
        assert_eq!(value.coverage.source_id, "certified-test-source");
        assert_eq!(value.coverage.generation, "test-generation");
        assert_eq!(
            value.event_checkpoint_digest,
            value
                .published
                .as_ref()
                .map(|p| digest(p.cp_sequence_number))
        );
    }
    assert!(Loader::load(&reader, &[key("Missing", 5)]).await.is_err());
}

#[tokio::test]
async fn empty_batch_needs_no_database_and_bigint_overflow_is_rejected() {
    let reader = PgReader::new(None, None, DbArgs::default(), &Registry::new())
        .await
        .unwrap();
    assert_eq!(
        Loader::<K>::load(&reader, &[]).await.unwrap(),
        HashMap::new()
    );
    let err = Loader::load(&reader, &[key("NFT", u64::MAX)])
        .await
        .unwrap_err();
    assert!(err.to_string().contains("BIGINT"));
}

#[tokio::test]
async fn missing_history_cannot_be_interpreted_as_no_template() {
    let (_temp, _writer, reader) = setup().await;
    assert!(Loader::load(&reader, &[key("Missing", 0)]).await.is_err());
}

#[tokio::test]
async fn watermark_binding_bounds_and_missing_genesis_fail_closed() {
    let (_temp, writer, reader) = setup().await;
    seed(&writer, 2, &[]).await;
    let mut conn = writer.connect().await.unwrap();
    for fault in [
        "UPDATE watermarks SET chain_id = NULL",
        "UPDATE watermarks SET chain_id = decode(repeat('01',32),'hex')",
        "UPDATE watermarks SET checkpoint_hi_inclusive = 1",
        "UPDATE watermarks SET checkpoint_hi_inclusive = 3",
        "UPDATE watermarks SET reader_lo = 1",
        "DELETE FROM watermarks",
        "DELETE FROM kv_genesis",
    ] {
        diesel::sql_query("BEGIN").execute(&mut conn).await.unwrap();
        diesel::sql_query(fault).execute(&mut conn).await.unwrap();
        diesel::sql_query("COMMIT")
            .execute(&mut conn)
            .await
            .unwrap();
        assert!(
            Loader::load(&reader, &[key("Missing", 1)]).await.is_err(),
            "{fault}"
        );
        if fault.starts_with("DELETE FROM kv_genesis") {
            break;
        }
        diesel::sql_query("DELETE FROM watermarks")
            .execute(&mut conn)
            .await
            .unwrap();
        diesel::sql_query("INSERT INTO watermarks VALUES ('hist_displays',0,2,0,0,0,NOW(),0,$1)")
            .bind::<Bytea, _>(digest(0))
            .execute(&mut conn)
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn event_bcs_id_and_version_must_agree_with_stored_metadata() {
    for fault in 0..3 {
        let (_temp, writer, reader) = setup().await;
        let mut release = row("NFT", 0, 0, 0, 32768, 1);
        match fault {
            0 => release.display = vec![0],
            1 => release.display_id = vec![2; 32],
            _ => release.display_version = 1,
        }
        seed(&writer, 0, &[release]).await;
        assert!(Loader::load(&reader, &[key("NFT", 0)]).await.is_err());
    }
}

#[tokio::test]
async fn coverage_and_events_are_visible_only_after_the_same_commit() {
    let (_temp, writer, reader) = setup().await;
    let first = row("NFT", 0, 0, 0, 1, 1);
    seed(&writer, 0, &[first.clone()]).await;
    let mut conn = writer.connect().await.unwrap();
    let next = row("NFT", 1, 0, 0, 2, 1);
    publish(&mut conn, 1, 1, &[next.clone()]).await;
    assert!(Loader::load(&reader, &[key("NFT", 1)]).await.is_err());
    assert_eq!(
        Loader::load(&reader, &[key("NFT", 0)]).await.unwrap()[&key("NFT", 0)].published,
        Some(first)
    );
    diesel::sql_query("COMMIT")
        .execute(&mut conn)
        .await
        .unwrap();
    assert_eq!(
        Loader::load(&reader, &[key("NFT", 1)]).await.unwrap()[&key("NFT", 1)].published,
        Some(next)
    );
}

#[tokio::test]
async fn actual_array_lateral_plan_seeks_hot_rare_missing_and_duplicate_requests() {
    let (_temp, writer, reader) = setup().await;
    let mut releases: Vec<_> = (1..=1024)
        .map(|cp| row("Hot", cp, 0, 0, cp as u16, 1))
        .collect();
    releases.push(row("Rare", 1, 0, 1, 1, 2));
    seed(&writer, 1024, &releases).await;
    let mut conn = writer.connect().await.unwrap();
    diesel::sql_query("ANALYZE hist_displays")
        .execute(&mut conn)
        .await
        .unwrap();
    let keys = [
        key("Hot", 0),
        key("Hot", 512),
        key("Hot", 1024),
        key("Rare", 1),
        key("Rare", 1000),
        key("Missing", 1000),
        key("Hot", 512),
    ];
    let started = Instant::now();
    let values = Loader::load(&reader, &keys).await.unwrap();
    let elapsed = started.elapsed();
    assert_eq!(values.len(), 6);
    assert_eq!(values[&keys[0]].published, None);
    assert_eq!(values[&keys[1]].published, Some(releases[511].clone()));
    assert_eq!(values[&keys[2]].published, Some(releases[1023].clone()));
    assert_eq!(values[&keys[3]].published, Some(releases[1024].clone()));
    assert_eq!(values[&keys[4]].published, Some(releases[1024].clone()));
    assert_eq!(values[&keys[5]].published, None);
    let types: Vec<_> = keys.iter().map(|k| bcs::to_bytes(&k.0).unwrap()).collect();
    let cps: Vec<_> = keys.iter().map(|k| k.1 as i64).collect();
    let plan: Vec<PlanLine> = diesel::sql_query(format!("EXPLAIN (ANALYZE, BUFFERS) {QUERY}"))
        .bind::<Array<Bytea>, _>(types)
        .bind::<Array<BigInt>, _>(cps)
        .load(&mut conn)
        .await
        .unwrap();
    let plan = plan
        .into_iter()
        .map(|p| p.line)
        .collect::<Vec<_>>()
        .join("\n");
    println!("Historical Display loader 7 requests / 1025 events: {elapsed:?}\n{plan}");
    assert!(plan.contains("hist_displays_type_position"), "{plan}");
    assert!(!plan.contains("Seq Scan on hist_displays"), "{plan}");
    assert!(!plan.contains("Sort Key: hist_displays"), "{plan}");
}
