// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use async_graphql::Context;
use async_graphql::EmptyMutation;
use async_graphql::EmptySubscription;
use async_graphql::Object;
use async_graphql::Request;
use async_graphql::Schema;
use diesel::sql_types::Bytea;
use diesel_async::RunQueryDsl;
use fastcrypto::encoding::Base58;
use fastcrypto::encoding::Encoding;
use fastcrypto::encoding::Hex;
use fastcrypto::hash::HashFunction;
use fastcrypto::hash::Sha256;
use move_core_types::language_storage::StructTag;
use prometheus::Registry;
use rtd_indexer_alt_reader::historical_displays::HistoricalDisplay;
use rtd_indexer_alt_reader::pg_reader::PgReader;
use rtd_indexer_alt_schema::MIGRATIONS;
use rtd_indexer_alt_schema::display_history::StoredDisplayCheckpoint;
use rtd_indexer_alt_schema::display_history::StoredDisplayCoverage;
use rtd_indexer_alt_schema::display_history::StoredHistoricalDisplay;
use rtd_indexer_alt_schema::objects::StoredObjVersion;
use rtd_indexer_alt_schema::schema::hist_display_checkpoints;
use rtd_indexer_alt_schema::schema::hist_display_coverage;
use rtd_indexer_alt_schema::schema::hist_displays;
use rtd_indexer_alt_schema::schema::obj_versions;
use rtd_pg_db::Db;
use rtd_pg_db::DbArgs;
use rtd_pg_db::temp::TempDb;
use rtd_rpc::proto::rtd::rpc::v2::ExecutedTransaction;
use rtd_types::base_types::ObjectID;
use rtd_types::collection_types::Entry;
use rtd_types::collection_types::VecMap;
use rtd_types::display_registry;
use rtd_types::id::ID;
use serde::Serialize;

use crate::api::types::move_value::MoveValue;
use crate::api::types::service_config::ServiceConfig;
use crate::error::RpcError;
use crate::scope::Scope;
use crate::task::watermark::Watermarks;

use super::Display;
use super::DisplayBasis;
use super::LoadedDisplay;
use super::MAX_UINT53;
use super::V2Template;
use super::checked_uint53;
use super::load_display;
use super::select_template;
use super::snapshot_checkpoint;

#[derive(Serialize)]
struct Release {
    id: ID,
    version: u16,
    fields: VecMap<String, String>,
}

struct Query;

#[Object(name = "DisplayTestQuery")]
impl Query {
    async fn config(&self) -> ServiceConfig {
        ServiceConfig {
            scope: Scope::for_tests(),
        }
    }

    async fn historical(&self, ctx: &Context<'_>) -> Result<Option<Display>, RpcError> {
        let selected =
            load_display(ctx, Scope::for_tests().with_root_version(999), type_()).await?;
        Ok(selected.map(|selected| Display {
            output: None,
            errors: None,
            basis: match selected {
                LoadedDisplay::V1(_, b) => DisplayBasis::V1(b),
                LoadedDisplay::V2(_, b) => DisplayBasis::V2(b),
            },
        }))
    }

    async fn rendered(&self, v2: bool) -> Display {
        let selected = select_template(type_(), 3, history(), v2.then(|| registry(2)))
            .unwrap()
            .unwrap();
        let basis = match selected {
            LoadedDisplay::V1(_, b) => DisplayBasis::V1(b),
            LoadedDisplay::V2(_, b) => DisplayBasis::V2(b),
        };
        Display {
            output: None,
            errors: None,
            basis,
        }
    }
}

fn type_() -> StructTag {
    "0x42::nft::NFT".parse().unwrap()
}

fn history() -> HistoricalDisplay {
    let event = bcs::to_bytes(&Release {
        id: ID::new(ObjectID::from_bytes([7; 32]).unwrap()),
        version: 65535,
        fields: VecMap {
            contents: vec![Entry {
                key: "name".into(),
                value: "published".into(),
            }],
        },
    })
    .unwrap();
    HistoricalDisplay {
        snapshot_checkpoint_digest: vec![6; 32],
        coverage: StoredDisplayCoverage {
            id: 1,
            genesis_digest: vec![1; 32],
            source_id: "certified-test-source".into(),
            generation: "test-generation".into(),
            reader_lo: 0,
            checkpoint_hi: 4,
            checkpoint_digest: Some(vec![5; 32]),
            writer_epoch: 1,
        },
        published: Some(StoredHistoricalDisplay {
            object_type: bcs::to_bytes(&type_()).unwrap(),
            cp_sequence_number: 2,
            checkpoint_tx_index: 8,
            event_index: 9,
            transaction_digest: vec![3; 32],
            display_id: vec![7; 32],
            display_version: 65535,
            display: event,
        }),
        event_checkpoint_digest: Some(vec![4; 32]),
    }
}

fn registry(cp: u64) -> V2Template {
    V2Template {
        display: display_registry::Display {
            id: display_registry::display_object_id(type_().into()).unwrap(),
            fields: VecMap {
                contents: vec![Entry {
                    key: "name".into(),
                    value: "registry".into(),
                }],
            },
            cap_id: None,
        },
        object_version: 20,
        object_digest: Base58::encode([10; 32]),
        source_checkpoint: cp,
        previous_transaction_digest: Base58::encode([11; 32]),
        contents_bcs_sha256: "a".repeat(64),
    }
}

async fn resolver_fixture(
    covered: bool,
) -> (TempDb, Schema<Query, EmptyMutation, EmptySubscription>) {
    let temp = TempDb::new().unwrap();
    let url = temp.database().url().clone();
    let writer = Db::for_write(url.clone(), DbArgs::default()).await.unwrap();
    writer.run_migrations(Some(&MIGRATIONS)).await.unwrap();
    let mut conn = writer.connect().await.unwrap();
    diesel::sql_query("BEGIN").execute(&mut conn).await.unwrap();
    diesel::sql_query("SET LOCAL rtd.alt_writer_epoch = '1'")
        .execute(&mut conn)
        .await
        .unwrap();
    diesel::sql_query("INSERT INTO kv_genesis VALUES ($1,1)")
        .bind::<Bytea, _>(vec![1; 32])
        .execute(&mut conn)
        .await
        .unwrap();
    diesel::insert_into(obj_versions::table)
        .values(StoredObjVersion {
            object_id: display_registry::display_object_id(type_().into())
                .unwrap()
                .as_ref()
                .to_vec(),
            object_version: 99,
            object_digest: Some(vec![8; 32]),
            cp_sequence_number: 1,
        })
        .execute(&mut conn)
        .await
        .unwrap();
    if covered {
        let mut history = history();
        let mut published = history.published.take().unwrap();
        published.cp_sequence_number = 0;
        diesel::insert_into(hist_display_checkpoints::table)
            .values(vec![
                StoredDisplayCheckpoint {
                    cp_sequence_number: 0,
                    checkpoint_digest: vec![1; 32],
                    previous_digest: None,
                    event_count: 1,
                    events_sha256: vec![9; 32],
                },
                StoredDisplayCheckpoint {
                    cp_sequence_number: 1,
                    checkpoint_digest: vec![2; 32],
                    previous_digest: Some(vec![1; 32]),
                    event_count: 0,
                    events_sha256: vec![9; 32],
                },
            ])
            .execute(&mut conn)
            .await
            .unwrap();
        diesel::insert_into(hist_displays::table)
            .values(published)
            .execute(&mut conn)
            .await
            .unwrap();
        history.coverage.checkpoint_hi = 1;
        history.coverage.checkpoint_digest = Some(vec![2; 32]);
        diesel::insert_into(hist_display_coverage::table)
            .values(history.coverage)
            .execute(&mut conn)
            .await
            .unwrap();
        diesel::sql_query("INSERT INTO watermarks VALUES ('hist_displays',0,1,0,0,0,NOW(),0,$1)")
            .bind::<Bytea, _>(vec![1; 32])
            .execute(&mut conn)
            .await
            .unwrap();
    }
    diesel::sql_query("COMMIT")
        .execute(&mut conn)
        .await
        .unwrap();
    drop(conn);
    let reader = PgReader::new(None, Some(url), DbArgs::default(), &Registry::new())
        .await
        .unwrap();
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .register_output_type::<MoveValue>()
        .data(Arc::new(reader.as_data_loader()))
        .data(Arc::new(Watermarks::for_test(&[
            ("hist_displays", 1),
            ("obj_versions", 1),
            ("consistent", 1),
            ("ledger_grpc", 1),
        ])))
        .finish();
    (temp, schema)
}

#[tokio::test]
async fn display_range_rejects_each_missing_cached_pipeline_despite_pg_coverage() {
    let (_temp, schema) = resolver_fixture(true).await;
    let all = [
        ("hist_displays", 1),
        ("obj_versions", 1),
        ("consistent", 1),
        ("ledger_grpc", 1),
    ];
    for (missing, _) in all {
        let pipelines: Vec<_> = all
            .into_iter()
            .filter(|(name, _)| *name != missing)
            .collect();
        let request = Request::new(
            "{ config { availableRange(type: \"MoveValue\", field: \"display\") { __typename } } }",
        )
        .data(Arc::new(Watermarks::for_test(&pipelines)));
        let response = Schema::execute(&schema, request).await;
        assert_eq!(
            response.errors.len(),
            1,
            "missing {missing}: {:?}",
            response.errors
        );
    }
}

#[tokio::test]
async fn capability_and_display_range_require_actual_pg_history_at_the_scope() {
    let (_temp, schema) = resolver_fixture(true).await;
    let response = Schema::execute(&schema, "{ config { displayHistory { apiVersion supportedTemplateKinds coverage { genesisDigest sourceId generation firstCheckpoint lastCheckpoint lastCheckpointDigest snapshotCheckpoint snapshotCheckpointDigest } } availableRange(type: \"MoveValue\", field: \"display\") { __typename } } }").await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    let data = response.data.into_json().unwrap();
    let cap = &data["config"]["displayHistory"];
    assert_eq!(cap["apiVersion"], 1);
    assert_eq!(
        cap["supportedTemplateKinds"],
        serde_json::json!(["V1", "V2"])
    );
    assert_eq!(cap["coverage"]["snapshotCheckpoint"], 0);
    assert_eq!(
        cap["coverage"]["snapshotCheckpointDigest"],
        Base58::encode([1; 32])
    );
    assert_eq!(cap["coverage"]["lastCheckpoint"], 1);
    assert_eq!(
        cap["coverage"]["lastCheckpointDigest"],
        Base58::encode([2; 32])
    );
    assert_eq!(
        data["config"]["availableRange"]["__typename"],
        "AvailableRange"
    );

    let (_temp, schema) = resolver_fixture(false).await;
    for query in [
        "{ config { displayHistory { apiVersion } } }",
        "{ config { availableRange(type: \"MoveValue\", field: \"display\") { __typename } } }",
    ] {
        let response = Schema::execute(&schema, query).await;
        assert_eq!(response.errors.len(), 1, "{:?}", response.errors);
        assert!(response.errors[0].message.contains("Display history"));
    }
}

#[tokio::test]
async fn v1_basis_uses_published_event_not_object_version() {
    let history = history();
    let expected_sha = Hex::encode(Sha256::digest(&history.published.as_ref().unwrap().display));
    let Some(LoadedDisplay::V1(event, basis)) = select_template(type_(), 3, history, None).unwrap()
    else {
        panic!("Expected v1 publication");
    };
    assert_eq!(event.version, 65535);
    assert_eq!(
        basis.template_id,
        ObjectID::from_bytes([7; 32]).unwrap().into()
    );
    assert_eq!(u64::from(basis.snapshot_checkpoint), 3);
    assert_eq!(u64::from(basis.published_version), 65535);
    assert_eq!(u64::from(basis.source_checkpoint), 2);
    assert_eq!(u64::from(basis.checkpoint_transaction_index), 8);
    assert_eq!(u64::from(basis.event_index), 9);
    assert_eq!(basis.object_type, type_().to_canonical_string(true));
    assert_eq!(basis.event_bcs_sha256, expected_sha);
    assert_eq!(basis.transaction_digest, Base58::encode([3; 32]));
    assert_eq!(basis.source_checkpoint_digest, Base58::encode([4; 32]));
    assert_eq!(basis.history_coverage.source_id, "certified-test-source");
    assert_eq!(basis.history_coverage.generation, "test-generation");
}

#[tokio::test]
async fn v2_at_snapshot_wins_and_has_registry_identity() {
    // This address is also checked against the TypeScript SDK's derivation in wallet tests.
    assert_eq!(
        registry(3).display.id.to_string(),
        "0x740798106da92e9542250651d68bbd154073b8dbcfa98199598783becdfbc49f"
    );
    let Some(LoadedDisplay::V2(display, basis)) =
        select_template(type_(), 3, history(), Some(registry(3))).unwrap()
    else {
        panic!("Expected v2 registry");
    };
    assert_eq!(display.fields.contents[0].value, "registry");
    assert_eq!(
        basis.template_id,
        display_registry::display_object_id(type_().into())
            .unwrap()
            .into()
    );
    assert_eq!(u64::from(basis.object_version), 20);
    assert_eq!(u64::from(basis.source_checkpoint), 3);
    assert_eq!(u64::from(basis.snapshot_checkpoint), 3);
    assert_eq!(
        basis.history_coverage.genesis_digest,
        Base58::encode([1; 32])
    );
}

#[tokio::test]
async fn future_v2_or_v1_cannot_be_grafted_onto_older_snapshot() {
    assert!(select_template(type_(), 3, history(), Some(registry(4))).is_err());
    assert!(select_template(type_(), 1, history(), None).is_err());
}

#[tokio::test]
async fn no_publication_is_null_only_when_history_covers_snapshot() {
    let mut history = history();
    history.published = None;
    history.event_checkpoint_digest = None;
    assert!(
        select_template(type_(), 3, history.clone(), None)
            .unwrap()
            .is_none()
    );
    history.coverage.checkpoint_hi = 2;
    assert!(select_template(type_(), 3, history.clone(), None).is_err());
    assert!(select_template(type_(), 3, history, Some(registry(2))).is_err());
}

#[tokio::test]
async fn wrong_type_id_version_and_event_digest_fail_closed() {
    for fault in 0..4 {
        let mut history = history();
        let row = history.published.as_mut().unwrap();
        match fault {
            0 => {
                row.object_type =
                    bcs::to_bytes(&"0x42::nft::Other".parse::<StructTag>().unwrap()).unwrap()
            }
            1 => row.display_id = vec![9; 32],
            2 => row.display_version = 1,
            _ => history.event_checkpoint_digest = None,
        }
        assert!(select_template(type_(), 3, history, None).is_err());
    }
}

#[tokio::test]
async fn basis_numbers_must_fit_javascript_safe_integer() {
    assert!(checked_uint53(MAX_UINT53).is_ok());
    assert!(checked_uint53(MAX_UINT53 + 1).is_err());
    let mut history = history();
    history.published.as_mut().unwrap().event_index = (MAX_UINT53 + 1) as i64;
    assert!(select_template(type_(), 3, history, None).is_err());
    let mut registry = registry(2);
    registry.object_version = MAX_UINT53 + 1;
    assert!(select_template(type_(), 3, self::history(), Some(registry)).is_err());
}

#[tokio::test]
async fn root_bounds_are_removed_without_changing_snapshot_and_execution_has_no_snapshot() {
    let scope = Scope::for_tests().with_root_version(99);
    assert_eq!(snapshot_checkpoint(&scope).unwrap(), 0);
    assert_eq!(scope.without_root_bound().checkpoint_viewed_at(), Some(0));
    assert_eq!(scope.without_root_bound().root_version(), None);
    let executed = scope
        .with_executed_transaction(&ExecutedTransaction::default())
        .unwrap();
    assert!(snapshot_checkpoint(&executed).is_err());
}

#[tokio::test]
async fn graphql_basis_union_exposes_typed_v1_and_v2_fields() {
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let selection = "output errors basis { __typename ... on DisplayV1Basis { snapshotCheckpoint objectType templateId publishedVersion sourceCheckpoint sourceCheckpointDigest transactionDigest checkpointTransactionIndex eventIndex eventBcsSha256 historyCoverage { genesisDigest sourceId generation firstCheckpoint lastCheckpoint lastCheckpointDigest } } ... on DisplayV2Basis { snapshotCheckpoint objectType templateId objectVersion objectDigest sourceCheckpoint previousTransactionDigest contentsBcsSha256 historyCoverage { genesisDigest sourceId generation firstCheckpoint lastCheckpoint lastCheckpointDigest } } }";
    for v2 in [false, true] {
        let response = Schema::execute(
            &schema,
            format!("{{ rendered(v2: {v2}) {{ {selection} }} }}"),
        )
        .await;
        assert!(response.errors.is_empty(), "{:?}", response.errors);
        let json = response.data.into_json().unwrap();
        let basis = &json["rendered"]["basis"];
        assert_eq!(
            basis["__typename"],
            if v2 {
                "DisplayV2Basis"
            } else {
                "DisplayV1Basis"
            }
        );
        assert_eq!(basis["snapshotCheckpoint"], 3);
        assert_eq!(basis["sourceCheckpoint"], 2);
        assert_eq!(
            basis["historyCoverage"]["genesisDigest"],
            Base58::encode([1; 32])
        );
        assert_eq!(
            basis["historyCoverage"]["sourceId"],
            "certified-test-source"
        );
        if v2 {
            assert_eq!(basis["objectVersion"], 20);
            assert_eq!(basis["objectDigest"], Base58::encode([10; 32]));
            assert!(basis.get("publishedVersion").is_none());
        } else {
            assert_eq!(basis["publishedVersion"], 65535);
            assert_eq!(basis["transactionDigest"], Base58::encode([3; 32]));
            assert!(basis.get("objectVersion").is_none());
        }
    }
}

#[tokio::test]
async fn resolver_uses_pg_history_and_excludes_future_v2_despite_inherited_root_version() {
    let (_temp, schema) = resolver_fixture(true).await;
    let response = Schema::execute(&schema, "{ historical { basis { __typename ... on DisplayV1Basis { snapshotCheckpoint sourceCheckpoint publishedVersion historyCoverage { lastCheckpoint } } } } }").await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    let data = response.data.into_json().unwrap();
    let basis = &data["historical"]["basis"];
    assert_eq!(basis["__typename"], "DisplayV1Basis");
    assert_eq!(basis["snapshotCheckpoint"], 0);
    assert_eq!(basis["sourceCheckpoint"], 0);
    assert_eq!(basis["publishedVersion"], 65535);
    assert_eq!(basis["historyCoverage"]["lastCheckpoint"], 1);
}

#[tokio::test]
async fn resolver_reports_missing_pg_coverage_as_error_instead_of_no_template() {
    let (_temp, schema) = resolver_fixture(false).await;
    let response = Schema::execute(&schema, "{ historical { basis { __typename } } }").await;
    assert_eq!(response.errors.len(), 1);
    assert_eq!(response.errors[0].path.len(), 1);
    assert!(response.errors[0].message.contains("historical Display"));
}
