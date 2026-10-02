// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Database-enforced generation fence for the Alt writer's data and watermark tables.

use anyhow::Context;
use anyhow::Result;
use diesel::QueryableByName;
use diesel::sql_types::BigInt;
use diesel_async::RunQueryDsl;
use rtd_indexer_alt_framework::postgres::Connection;
use rtd_indexer_alt_framework::postgres::Db;

// Keep this list aligned with the active tables in rtd-indexer-alt-schema/src/schema.rs.
// The database also hosts publication and control tables, which must not be fenced here.
const FENCED_TABLES: &[&str] = &[
    "cp_bloom_blocks",
    "cp_blooms",
    "cp_digests",
    "cp_sequence_numbers",
    "ev_emit_mod",
    "ev_struct_inst",
    "kv_checkpoints",
    "kv_epoch_ends",
    "kv_epoch_starts",
    "kv_feature_flags",
    "kv_genesis",
    "kv_objects",
    "kv_packages",
    "kv_protocol_configs",
    "kv_transactions",
    "obj_versions",
    "sum_displays",
    "tx_affected_addresses",
    "tx_affected_objects",
    "tx_balance_changes",
    "tx_calls",
    "tx_digests",
    "tx_kinds",
    "watermarks",
];

const CREATE_FENCE_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS public.rtd_alt_writer_fence (
        id SMALLINT PRIMARY KEY CHECK (id = 1),
        epoch BIGINT NOT NULL CHECK (epoch > 0)
    )";

const ADVANCE_EPOCH: &str = "
    INSERT INTO public.rtd_alt_writer_fence (id, epoch)
    VALUES (1, 1)
    ON CONFLICT (id)
    DO UPDATE SET epoch = public.rtd_alt_writer_fence.epoch + 1
    RETURNING epoch";

const CREATE_GUARD_FUNCTION: &str = r#"
    CREATE OR REPLACE FUNCTION public.rtd_alt_require_writer_epoch()
    RETURNS trigger
    LANGUAGE plpgsql
    AS $rtd_fence$
    DECLARE active_epoch BIGINT;
    BEGIN
        -- The row lock lasts through the caller's transaction. A promotion waits
        -- for already admitted writes to commit before advancing the epoch.
        SELECT epoch INTO active_epoch
          FROM public.rtd_alt_writer_fence
         WHERE id = 1
         FOR SHARE;
        IF active_epoch IS NULL OR
           NULLIF(current_setting('rtd.alt_writer_epoch', true), '')::BIGINT
               IS DISTINCT FROM active_epoch THEN
            RAISE EXCEPTION 'Stale or missing Alt writer epoch'
                USING ERRCODE = '55000';
        END IF;
        RETURN NULL;
    END
    $rtd_fence$"#;

#[derive(QueryableByName)]
struct Epoch {
    #[diesel(sql_type = BigInt)]
    epoch: i64,
}

pub async fn advance_epoch(conn: &mut Connection<'_>) -> Result<i64> {
    diesel::sql_query(CREATE_FENCE_TABLE)
        .execute(conn)
        .await
        .context("Failed to create Alt writer fence table")?;
    let current: Epoch = diesel::sql_query(ADVANCE_EPOCH)
        .get_result(conn)
        .await
        .context("Failed to advance Alt writer epoch")?;
    Ok(current.epoch)
}

pub async fn install(db: &Db) -> Result<()> {
    let mut conn = db.connect().await?;
    diesel::sql_query(CREATE_GUARD_FUNCTION)
        .execute(&mut conn)
        .await
        .context("Failed to install Alt writer fence function")?;

    let table_names = FENCED_TABLES
        .iter()
        .map(|name| format!("'{name}'"))
        .collect::<Vec<_>>()
        .join(", ");
    let install_triggers = format!(
        r#"
        DO $rtd_fence$
        DECLARE table_name TEXT;
        BEGIN
            FOREACH table_name IN ARRAY ARRAY[{table_names}] LOOP
                IF to_regclass(format('%I.%I', current_schema(), table_name)) IS NULL THEN
                    RAISE EXCEPTION 'Missing Alt writer table: %', table_name;
                END IF;
                EXECUTE format(
                    'DROP TRIGGER IF EXISTS rtd_alt_epoch_guard ON %I.%I',
                    current_schema(), table_name
                );
                EXECUTE format(
                    'CREATE TRIGGER rtd_alt_epoch_guard
                     BEFORE INSERT OR UPDATE OR DELETE OR TRUNCATE ON %I.%I
                     FOR EACH STATEMENT
                     EXECUTE FUNCTION public.rtd_alt_require_writer_epoch()',
                    current_schema(), table_name
                );
            END LOOP;
        END
        $rtd_fence$"#
    );
    diesel::sql_query(install_triggers)
        .execute(&mut conn)
        .await
        .context("Failed to install Alt writer table guards")?;
    Ok(())
}
