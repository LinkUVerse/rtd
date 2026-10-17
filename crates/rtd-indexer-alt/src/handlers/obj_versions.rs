// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use diesel::sql_types::BigInt;
use diesel_async::RunQueryDsl;
use rtd_indexer_alt_framework::pipeline::Processor;
use rtd_indexer_alt_framework::postgres::Connection;
use rtd_indexer_alt_framework::postgres::handler::Handler;
use rtd_indexer_alt_framework::types::effects::TransactionEffectsAPI;
use rtd_indexer_alt_framework::types::full_checkpoint_content::Checkpoint;
use rtd_indexer_alt_schema::objects::StoredObjVersion;
use rtd_indexer_alt_schema::schema::obj_versions;

pub(crate) struct ObjVersions;

#[async_trait]
impl Processor for ObjVersions {
    const NAME: &'static str = "obj_versions";
    type Value = StoredObjVersion;

    async fn process(&self, checkpoint: &Arc<Checkpoint>) -> Result<Vec<Self::Value>> {
        let Checkpoint {
            transactions,
            summary,
            ..
        } = checkpoint.as_ref();

        let cp_sequence_number = summary.sequence_number as i64;
        Ok(transactions
            .iter()
            .flat_map(|tx| {
                let lamport = tx.effects.lamport_version();

                tx.effects
                    .object_changes()
                    .into_iter()
                    .map(move |c| StoredObjVersion {
                        object_id: c.id.to_vec(),
                        // If the object was created or modified, it has an output version,
                        // otherwise it was deleted/wrapped and its version is the transaction's
                        // lamport version.
                        object_version: c.output_version.unwrap_or(lamport).value() as i64,
                        object_digest: c.output_digest.map(|d| d.inner().into()),
                        cp_sequence_number,
                    })
            })
            .collect())
    }
}

#[async_trait]
impl Handler for ObjVersions {
    const MIN_EAGER_ROWS: usize = 100;
    const MAX_PENDING_ROWS: usize = 10000;

    async fn commit<'a>(values: &[Self::Value], conn: &mut Connection<'a>) -> Result<usize> {
        Ok(diesel::insert_into(obj_versions::table)
            .values(values)
            .on_conflict_do_nothing()
            .execute(conn)
            .await?)
    }

    async fn prune<'a>(
        &self,
        _from: u64,
        to_exclusive: u64,
        conn: &mut Connection<'a>,
    ) -> Result<usize> {
        // Keep the last pre-boundary row for every object. A checkpoint-bounded
        // query still needs that row when the object has not changed since the
        // retained range began. Keep deletion markers for the same reason.
        // Revisit earlier chunks as the boundary moves so that an old anchor
        // can be replaced by a newer pre-boundary anchor.
        let to_exclusive = i64::try_from(to_exclusive)?;
        // Content and metadata use the same doomed set in one atomic SQL
        // statement. A crash cannot leave a retained version without content.
        // The last pre-boundary anchor, including a deletion marker, is kept.
        Ok(diesel::sql_query(
            r#"
                WITH doomed AS MATERIALIZED (
                    SELECT old.object_id, old.object_version
                    FROM obj_versions AS old
                    WHERE old.cp_sequence_number < $1
                      AND EXISTS (
                          SELECT 1 FROM obj_versions AS newer
                          WHERE newer.object_id = old.object_id
                            AND newer.cp_sequence_number < $1
                            AND (newer.cp_sequence_number, newer.object_version) >
                                (old.cp_sequence_number, old.object_version)
                      )
                ), deleted_content AS (
                    DELETE FROM kv_objects AS content
                    USING doomed
                    WHERE content.object_id = doomed.object_id
                      AND content.object_version = doomed.object_version
                )
                DELETE FROM obj_versions AS old
                USING doomed
                WHERE old.object_id = doomed.object_id
                  AND old.object_version = doomed.object_version
            "#,
        )
        .bind::<BigInt, _>(to_exclusive)
        .execute(conn)
        .await?)
    }
}

#[cfg(test)]
mod tests {
    use diesel::QueryDsl;
    use diesel_async::RunQueryDsl;
    use rtd_indexer_alt_framework::postgres::Connection;
    use rtd_indexer_alt_framework::postgres::Db;
    use rtd_indexer_alt_framework::postgres::DbArgs;
    use rtd_indexer_alt_framework::postgres::handler::Handler;
    use rtd_indexer_alt_framework::postgres::temp::TempDb;
    use rtd_indexer_alt_schema::MIGRATIONS;
    use rtd_indexer_alt_schema::objects::StoredObjVersion;
    use rtd_indexer_alt_schema::objects::StoredObject;
    use rtd_indexer_alt_schema::schema::kv_objects;
    use rtd_indexer_alt_schema::schema::obj_versions;

    use super::ObjVersions;

    type Rows = Vec<(Vec<u8>, i64, Option<Vec<u8>>)>;

    async fn snapshot(conn: &mut Connection<'_>) -> (Rows, Rows) {
        let versions = obj_versions::table
            .select((
                obj_versions::object_id,
                obj_versions::object_version,
                obj_versions::object_digest,
            ))
            .order((obj_versions::object_id, obj_versions::object_version))
            .load(conn)
            .await
            .unwrap();
        let content = kv_objects::table
            .select((
                kv_objects::object_id,
                kv_objects::object_version,
                kv_objects::serialized_object,
            ))
            .order((kv_objects::object_id, kv_objects::object_version))
            .load(conn)
            .await
            .unwrap();
        (versions, content)
    }

    #[tokio::test]
    async fn pruning_preserves_anchors_and_tombstones_and_rolls_back_both_tables() {
        let temp = TempDb::new().unwrap();
        let db = Db::for_write(temp.database().url().clone(), DbArgs::default())
            .await
            .unwrap();
        db.run_migrations(Some(&MIGRATIONS)).await.unwrap();
        let mut conn = db.connect().await.unwrap();
        let mut versions = Vec::new();
        let mut content = Vec::new();
        for (id, version, checkpoint, deleted) in [
            (1, 1, 1, false),
            (1, 2, 2, false),
            (1, 3, 3, false),
            (1, 4, 6, false),
            (2, 1, 1, false),
            (2, 2, 2, true),
        ] {
            versions.push(StoredObjVersion {
                object_id: vec![id; 32],
                object_version: version,
                cp_sequence_number: checkpoint,
                object_digest: (!deleted).then(|| vec![id; 32]),
            });
            content.push(StoredObject {
                object_id: vec![id; 32],
                object_version: version,
                serialized_object: (!deleted).then(|| vec![id, version as u8]),
            });
        }
        diesel::insert_into(obj_versions::table)
            .values(versions)
            .execute(&mut conn)
            .await
            .unwrap();
        diesel::insert_into(kv_objects::table)
            .values(content)
            .execute(&mut conn)
            .await
            .unwrap();
        let before = snapshot(&mut conn).await;
        diesel::sql_query("CREATE FUNCTION reject_prune() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'injected prune failure'; END $$")
            .execute(&mut conn).await.unwrap();
        diesel::sql_query("CREATE TRIGGER reject_prune BEFORE DELETE ON obj_versions FOR EACH ROW EXECUTE FUNCTION reject_prune()")
            .execute(&mut conn).await.unwrap();
        assert!(ObjVersions.prune(0, 3, &mut conn).await.is_err());
        assert_eq!(snapshot(&mut conn).await, before);
        diesel::sql_query("DROP TRIGGER reject_prune ON obj_versions")
            .execute(&mut conn)
            .await
            .unwrap();
        assert_eq!(ObjVersions.prune(0, 3, &mut conn).await.unwrap(), 2);
        let retained = snapshot(&mut conn).await;
        assert_eq!(retained.0.len(), 4);
        assert_eq!(retained.1.len(), 4);
        assert_eq!(
            retained
                .0
                .iter()
                .map(|row| (row.0[0], row.1))
                .collect::<Vec<_>>(),
            vec![(1, 2), (1, 3), (1, 4), (2, 2)]
        );
        assert!(retained.0.last().unwrap().2.is_none());
        assert!(retained.1.last().unwrap().2.is_none());
        assert_eq!(ObjVersions.prune(3, 6, &mut conn).await.unwrap(), 1);
        let stable = snapshot(&mut conn).await;
        assert_eq!(ObjVersions.prune(3, 6, &mut conn).await.unwrap(), 0);
        assert!(ObjVersions.prune(0, u64::MAX, &mut conn).await.is_err());
        assert_eq!(snapshot(&mut conn).await, stable);
        assert_eq!(ObjVersions.prune(6, 7, &mut conn).await.unwrap(), 1);
        let final_state = snapshot(&mut conn).await;
        assert_eq!(
            final_state
                .0
                .iter()
                .map(|row| (row.0[0], row.1))
                .collect::<Vec<_>>(),
            vec![(1, 4), (2, 2)]
        );
        assert_eq!(
            final_state
                .1
                .iter()
                .map(|row| (row.0[0], row.1))
                .collect::<Vec<_>>(),
            vec![(1, 4), (2, 2)]
        );
    }
}
