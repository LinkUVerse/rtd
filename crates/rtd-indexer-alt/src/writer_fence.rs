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
pub(crate) const FENCED_TABLES: &[&str] = &[
    "cp_bloom_blocks",
    "cp_blooms",
    "cp_digests",
    "cp_sequence_numbers",
    "ev_emit_mod",
    "ev_struct_inst",
    "hist_display_checkpoints",
    "hist_display_coverage",
    "hist_displays",
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
    advance_existing_epoch(conn).await
}

pub async fn advance_existing_epoch(conn: &mut Connection<'_>) -> Result<i64> {
    let current: Epoch = diesel::sql_query(ADVANCE_EPOCH)
        .get_result(conn)
        .await
        .context("Failed to advance Alt writer epoch")?;
    Ok(current.epoch)
}

/// Explicit initialization creates the empty fence but never promotes a writer.
pub async fn prepare(db: &Db) -> Result<()> {
    let mut conn = db.connect().await?;
    diesel::sql_query(CREATE_FENCE_TABLE)
        .execute(&mut conn)
        .await?;
    drop(conn);
    install(db).await
}

/// Startup observes the installed function and all statement triggers; a lost
/// or modified guard must not be repaired implicitly under runtime credentials.
pub async fn verify(db: &Db) -> Result<()> {
    let conn = db.connect().await?;
    verify_connection(conn).await.map(|_| ())
}

/// Return the exact verified session so admission can promote on the same
/// connection while retaining its writer/migration exclusion lock.
pub async fn verify_connection<'a>(mut conn: Connection<'a>) -> Result<Connection<'a>> {
    verify_on(&mut conn).await?;
    Ok(conn)
}

pub(crate) async fn verify_on(conn: &mut Connection<'_>) -> Result<()> {
    #[derive(QueryableByName)]
    struct Verified {
        #[diesel(sql_type=diesel::sql_types::Bool)]
        valid: bool,
    }
    let body = CREATE_GUARD_FUNCTION
        .split("$rtd_fence$")
        .nth(1)
        .expect("embedded fence body");
    let valid:Verified=diesel::sql_query("SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace JOIN pg_language l ON l.oid=p.prolang WHERE n.nspname='public' AND p.proname='rtd_alt_require_writer_epoch' AND p.pronargs=0 AND p.prorettype='trigger'::regtype AND l.lanname='plpgsql' AND NOT p.prosecdef AND NOT p.proleakproof AND NOT p.proisstrict AND p.provolatile='v' AND p.prosrc=$1) AND EXISTS(SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relname='rtd_alt_writer_fence' AND c.relkind='r' AND NOT(c.relrowsecurity OR c.relforcerowsecurity)) AS valid").bind::<diesel::sql_types::Text,_>(body).get_result(&mut **conn).await?;
    anyhow::ensure!(
        valid.valid,
        "Alt writer fence is unavailable or differs; run explicit schema initialization"
    );
    let valid:Verified=diesel::sql_query("SELECT (SELECT string_agg(a.attname||':'||format_type(a.atttypid,a.atttypmod)||':'||a.attnotnull,',' ORDER BY a.attnum) FROM pg_attribute a WHERE a.attrelid='public.rtd_alt_writer_fence'::regclass AND a.attnum>0 AND NOT a.attisdropped)='id:smallint:true,epoch:bigint:true' AND (SELECT count(*)=3 AND bool_and(k.convalidated AND NOT k.condeferrable AND pg_get_constraintdef(k.oid) IN ('PRIMARY KEY (id)','CHECK ((id = 1))','CHECK ((epoch > 0))')) FROM pg_constraint k WHERE k.conrelid='public.rtd_alt_writer_fence'::regclass) AND NOT EXISTS(SELECT 1 FROM pg_trigger WHERE tgrelid='public.rtd_alt_writer_fence'::regclass AND NOT tgisinternal) AS valid").get_result(&mut **conn).await?;
    anyhow::ensure!(
        valid.valid,
        "Alt writer fence table structure differs; ordinary startup cannot repair it"
    );
    for table in FENCED_TABLES {
        let valid:Verified=diesel::sql_query("SELECT EXISTS(SELECT 1 FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relname=$1 AND c.relkind='r' AND NOT(c.relrowsecurity OR c.relforcerowsecurity) AND t.tgname='rtd_alt_epoch_guard' AND t.tgfoid='public.rtd_alt_require_writer_epoch()'::regprocedure AND t.tgtype=62 AND t.tgenabled='O' AND NOT t.tgisinternal AND t.tgconstraint=0 AND t.tgnargs=0 AND t.tgqual IS NULL AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL) AS valid").bind::<diesel::sql_types::Text,_>(*table).get_result(&mut **conn).await?;
        anyhow::ensure!(
            valid.valid,
            "Alt writer table guard differs for {table}; ordinary startup never repairs schema"
        );
    }
    Ok(())
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
