// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeSet;

use rtd_kvstore::{BigTableClient, WatermarkV1};
use rtd_rpc::proto::rtd::rpc::v2::{
    BatchGetObjectsRequest, BatchGetObjectsResponse, BatchGetTransactionsRequest,
    BatchGetTransactionsResponse, GetCheckpointRequest, GetCheckpointResponse, GetEpochRequest,
    GetEpochResponse, GetObjectRequest, GetObjectResponse, GetServiceInfoRequest,
    GetServiceInfoResponse, GetTransactionRequest, GetTransactionResponse, ListCheckpointsRequest,
    ListCheckpointsResponse, ListEventsRequest, ListEventsResponse, ListTransactionsRequest,
    ListTransactionsResponse, ledger_service_server::LedgerService,
};
use rtd_rpc_api::proto::timestamp_ms_to_proto;
use rtd_rpc_api::{CheckpointNotFoundError, RpcError, ServerVersion};
use rtd_types::digests::{ChainIdentifier, CheckpointDigest};
use tonic::codegen::BoxStream;

use crate::KvRpcServer;
use crate::operation::OperationSpec;

pub(crate) mod get_checkpoint;
mod get_epoch;
mod get_object;
pub(crate) mod get_transaction;
mod list_checkpoints;
mod list_events;
mod list_transactions;
pub(crate) mod move_package_service;

#[tonic::async_trait]
impl LedgerService for KvRpcServer {
    async fn get_service_info(
        &self,
        request: tonic::Request<GetServiceInfoRequest>,
    ) -> Result<tonic::Response<GetServiceInfoResponse>, tonic::Status> {
        if let Some(height) = crate::operation::admission_ceiling(request.metadata())? {
            return get_admitted_service_info(
                self.client.clone(),
                self.chain_id,
                self.server_version.clone(),
                &self.service_info_watermark_pipelines,
                height,
            )
            .await
            .map(tonic::Response::new)
            .map_err(Into::into);
        }
        {
            let cache = self.cache.read().await;
            if let Some((sampled_at, cached_info)) = cache.as_ref() {
                if sampled_at.elapsed() < tokio::time::Duration::from_secs(1) {
                    return Ok(tonic::Response::new(cached_info.clone()));
                }
            }
        }
        // If no cache available, fetch directly and update cache
        get_service_info(
            self.client.clone(),
            self.chain_id,
            self.server_version.clone(),
            &self.service_info_watermark_pipelines,
        )
        .await
        .map(tonic::Response::new)
        .map_err(Into::into)
    }

    async fn get_object(
        &self,
        request: tonic::Request<GetObjectRequest>,
    ) -> Result<tonic::Response<GetObjectResponse>, tonic::Status> {
        let admission = if let Some(height) =
            crate::operation::admission_ceiling(request.metadata())?
        {
            let info = get_admitted_service_info(
                self.client.clone(),
                self.chain_id,
                self.server_version.clone(),
                &self.service_info_watermark_pipelines,
                height,
            )
            .await?;
            Some(
                info.checkpoint_height
                    .ok_or_else(|| tonic::Status::unavailable("Missing admission checkpoint"))?,
            )
        } else {
            None
        };
        let resolver = self.request_package_resolver(admission);
        get_object::get_object(
            self.client.clone(),
            request.into_inner(),
            &resolver,
            admission,
        )
        .await
        .map(tonic::Response::new)
        .map_err(Into::into)
    }

    async fn batch_get_objects(
        &self,
        request: tonic::Request<BatchGetObjectsRequest>,
    ) -> Result<tonic::Response<BatchGetObjectsResponse>, tonic::Status> {
        let admission = if let Some(height) =
            crate::operation::admission_ceiling(request.metadata())?
        {
            let info = get_admitted_service_info(
                self.client.clone(),
                self.chain_id,
                self.server_version.clone(),
                &self.service_info_watermark_pipelines,
                height,
            )
            .await?;
            Some(
                info.checkpoint_height
                    .ok_or_else(|| tonic::Status::unavailable("Missing admission checkpoint"))?,
            )
        } else {
            None
        };
        let resolver = self.request_package_resolver(admission);
        get_object::batch_get_objects(
            self.client.clone(),
            request.into_inner(),
            &resolver,
            admission,
        )
        .await
        .map(tonic::Response::new)
        .map_err(Into::into)
    }

    async fn get_transaction(
        &self,
        request: tonic::Request<GetTransactionRequest>,
    ) -> Result<tonic::Response<GetTransactionResponse>, tonic::Status> {
        let admission = crate::operation::admission_ceiling(request.metadata())?;
        let resolver = self.request_package_resolver(admission);
        get_transaction::get_transaction(
            self.limited_client("GetTransaction"),
            &self.stages,
            request.into_inner(),
            &resolver,
            admission,
        )
        .await
        .map(tonic::Response::new)
        .map_err(Into::into)
    }

    async fn batch_get_transactions(
        &self,
        request: tonic::Request<BatchGetTransactionsRequest>,
    ) -> Result<tonic::Response<BatchGetTransactionsResponse>, tonic::Status> {
        let admission = crate::operation::admission_ceiling(request.metadata())?;
        let resolver = self.request_package_resolver(admission);
        get_transaction::batch_get_transactions(
            self.limited_client("BatchGetTransactions"),
            &self.stages,
            request.into_inner(),
            &resolver,
            admission,
        )
        .await
        .map(tonic::Response::new)
        .map_err(Into::into)
    }

    async fn get_checkpoint(
        &self,
        request: tonic::Request<GetCheckpointRequest>,
    ) -> Result<tonic::Response<GetCheckpointResponse>, tonic::Status> {
        let admission = if let Some(height) =
            crate::operation::admission_ceiling(request.metadata())?
        {
            let info = get_admitted_service_info(
                self.client.clone(),
                self.chain_id,
                self.server_version.clone(),
                &self.service_info_watermark_pipelines,
                height,
            )
            .await?;
            Some(
                info.checkpoint_height
                    .ok_or_else(|| tonic::Status::unavailable("Missing admission checkpoint"))?,
            )
        } else {
            None
        };
        let resolver = self.request_package_resolver(admission);
        get_checkpoint::get_checkpoint(
            self.client.clone(),
            self.limited_client("GetCheckpoint"),
            &self.stages,
            &self.service_info_watermark_pipelines,
            request.into_inner(),
            admission,
            &resolver,
        )
        .await
        .map(tonic::Response::new)
        .map_err(Into::into)
    }

    async fn get_epoch(
        &self,
        request: tonic::Request<GetEpochRequest>,
    ) -> Result<tonic::Response<GetEpochResponse>, tonic::Status> {
        let admission = if let Some(height) =
            crate::operation::admission_ceiling(request.metadata())?
        {
            let info = get_admitted_service_info(
                self.client.clone(),
                self.chain_id,
                self.server_version.clone(),
                &self.service_info_watermark_pipelines,
                height,
            )
            .await?;
            Some((
                info.checkpoint_height
                    .ok_or_else(|| tonic::Status::unavailable("Missing admission checkpoint"))?,
                info.epoch
                    .ok_or_else(|| tonic::Status::unavailable("Missing admission epoch"))?,
            ))
        } else {
            None
        };
        get_epoch::get_epoch(
            self.client.clone(),
            request.into_inner(),
            self.chain_id.chain(),
            admission,
        )
        .await
        .map(tonic::Response::new)
        .map_err(Into::into)
    }

    // The list RPC hard timeout covers both computation and response delivery.
    // Expiry drops the stream with `DeadlineExceeded` without emitting a
    // terminal resume-cursor frame, so a client can resume only from the last
    // `Watermark` it retained.
    async fn list_checkpoints(
        &self,
        request: tonic::Request<ListCheckpointsRequest>,
    ) -> Result<tonic::Response<BoxStream<ListCheckpointsResponse>>, tonic::Status> {
        self.check_list_apis_enabled()?;
        self.serve_query_stream(
            OperationSpec::new(
                "list_checkpoints",
                self.ledger_history.list_checkpoints().timeout,
            ),
            request,
            list_checkpoints::list_checkpoints,
        )
        .await
    }

    async fn list_transactions(
        &self,
        request: tonic::Request<ListTransactionsRequest>,
    ) -> Result<tonic::Response<BoxStream<ListTransactionsResponse>>, tonic::Status> {
        self.check_list_apis_enabled()?;
        self.serve_query_stream(
            OperationSpec::new(
                "list_transactions",
                self.ledger_history.list_transactions().timeout,
            ),
            request,
            list_transactions::list_transactions,
        )
        .await
    }

    async fn list_events(
        &self,
        request: tonic::Request<ListEventsRequest>,
    ) -> Result<tonic::Response<BoxStream<ListEventsResponse>>, tonic::Status> {
        self.check_list_apis_enabled()?;
        self.serve_query_stream(
            OperationSpec::new("list_events", self.ledger_history.list_events().timeout),
            request,
            list_events::list_events,
        )
        .await
    }
}

pub(crate) async fn get_service_info(
    client: BigTableClient,
    chain_id: ChainIdentifier,
    server_version: Option<ServerVersion>,
    watermark_pipelines: &[&str],
) -> Result<GetServiceInfoResponse, RpcError> {
    let Some(wm) = common_service_watermark(client, watermark_pipelines).await? else {
        return Err(CheckpointNotFoundError::sequence_number(0).into());
    };
    let Some(checkpoint_hi_inclusive) = wm.checkpoint_hi_inclusive else {
        return Err(CheckpointNotFoundError::sequence_number(0).into());
    };
    let mut message = GetServiceInfoResponse::default();
    message.chain_id = Some(CheckpointDigest::new(*chain_id.as_bytes()).base58_encode());
    message.chain = Some(chain_id.chain().as_str().into());
    message.epoch = Some(wm.epoch_hi_inclusive);
    message.checkpoint_height = Some(checkpoint_hi_inclusive);
    message.timestamp = Some(timestamp_ms_to_proto(wm.timestamp_ms_hi_inclusive));
    message.lowest_available_checkpoint = Some(wm.reader_lo);
    message.lowest_available_checkpoint_objects = Some(wm.reader_lo);
    message.server = server_version.as_ref().map(ToString::to_string);
    Ok(message)
}

/// RPC coverage is the intersection of all served pipelines. Keep metadata from
/// the least advanced pipeline, but never use its lower bound for the others.
pub(crate) async fn common_service_watermark(
    mut client: BigTableClient,
    pipelines: &[&str],
) -> Result<Option<WatermarkV1>, RpcError> {
    use rtd_kvstore::tables::watermarks;

    let mut expected: BTreeSet<Vec<u8>> = pipelines
        .iter()
        .map(|pipeline| watermarks::encode_key(pipeline))
        .collect();
    if expected.is_empty() || expected.len() != pipelines.len() {
        return Ok(None);
    }
    // One batched read also avoids multiplying the background poller's RPC load.
    let rows = client
        .multi_get(watermarks::NAME, expected.iter().cloned().collect(), None)
        .await?;
    if rows.len() != expected.len() {
        return Ok(None);
    }
    let mut earliest: Option<(u64, WatermarkV1)> = None;
    let mut lower = 0;
    for (key, row) in rows {
        if !expected.remove(key.as_ref()) {
            return Err(RpcError::new(
                tonic::Code::Unavailable,
                "Unexpected service watermark row",
            ));
        }
        let Some(watermark) = watermarks::decode_v1(&row)? else {
            return Ok(None);
        };
        let Some(upper) = watermark.checkpoint_hi_inclusive else {
            return Ok(None);
        };
        lower = lower.max(watermark.reader_lo);
        if earliest.as_ref().is_none_or(|(height, _)| upper < *height) {
            earliest = Some((upper, watermark));
        }
    }
    let Some((upper, mut watermark)) = earliest else {
        return Ok(None);
    };
    if upper < lower {
        return Ok(None);
    }
    watermark.reader_lo = lower;
    Ok(Some(watermark))
}

async fn get_admitted_service_info(
    mut client: BigTableClient,
    chain_id: ChainIdentifier,
    server_version: Option<ServerVersion>,
    pipelines: &[&str],
    admission: u64,
) -> Result<GetServiceInfoResponse, RpcError> {
    let unavailable = || {
        RpcError::new(
            tonic::Code::Unavailable,
            "Publication admission snapshot unavailable",
        )
    };
    let watermark = common_service_watermark(client.clone(), pipelines)
        .await?
        .ok_or_else(unavailable)?;
    let height = admission.min(watermark.checkpoint_hi_inclusive.ok_or_else(unavailable)?);
    let lower = watermark.reader_lo;
    if height < lower {
        return Err(unavailable());
    }
    let summary = client
        .get_checkpoints_filtered(
            &[height],
            Some(&[rtd_kvstore::tables::checkpoints::col::SUMMARY]),
        )
        .await?
        .pop()
        .and_then(|checkpoint| checkpoint.summary)
        .ok_or_else(unavailable)?;
    if summary.sequence_number != height {
        return Err(unavailable());
    }
    let mut message = GetServiceInfoResponse::default();
    message.chain_id = Some(CheckpointDigest::new(*chain_id.as_bytes()).base58_encode());
    message.chain = Some(chain_id.chain().as_str().into());
    message.epoch = Some(summary.epoch);
    message.checkpoint_height = Some(height);
    message.timestamp = Some(timestamp_ms_to_proto(summary.timestamp_ms));
    // Advertise the common retained range across every pipeline this instance serves.
    message.lowest_available_checkpoint = Some(lower);
    message.lowest_available_checkpoint_objects = Some(lower);
    message.server = server_version.as_ref().map(ToString::to_string);
    Ok(message)
}

#[cfg(test)]
pub(crate) mod test_utils {
    use std::sync::Arc;

    use prometheus::Registry;
    use prometheus::proto::{Counter, Histogram, MetricFamily};
    use rtd_kvstore::BigTableClient as InnerBigTableClient;
    use rtd_kvstore::testing::MockBigtableServer;
    use rtd_package_resolver::PackageStore;
    use rtd_package_resolver::Resolver;
    use rtd_rpc::proto::rtd::rpc::v2::ExecutedTransaction;
    use rtd_rpc::proto::rtd::rpc::v2::Ordering;
    use rtd_rpc::proto::rtd::rpc::v2::QueryOptions;
    use rtd_types::balance_change::derive_balance_changes_2;
    use rtd_types::full_checkpoint_content::Checkpoint;
    use rtd_types::storage::{ObjectKey, get_transaction_object_set};
    use rtd_types::test_checkpoint_data_builder::TestCheckpointBuilder;

    use crate::KvRpcMetrics;
    use crate::LedgerHistoryConfig;
    use crate::LedgerHistoryMethodConfig;
    use crate::StageConfig;
    use crate::StagesConfig;
    use crate::bigtable_client::BigTableClient;
    use crate::operation::QueryContext;
    use crate::package_store::BigTablePackageStore;

    pub(crate) fn framework_objects_for_json_tests() -> Vec<rtd_types::object::Object> {
        use rtd_types::move_package::MovePackage;
        use rtd_types::object::Object;
        use std::collections::BTreeMap;

        // Locked RTD bytecode provides real Coin/UID layouts for parent masks.
        let snapshots: [&[u8]; 2] = [
            include_bytes!(
                "../../../rtd-framework-snapshot/bytecode_snapshot/138/0x0000000000000000000000000000000000000000000000000000000000000001"
            ),
            include_bytes!(
                "../../../rtd-framework-snapshot/bytecode_snapshot/138/0x0000000000000000000000000000000000000000000000000000000000000002"
            ),
        ];
        snapshots
            .into_iter()
            .map(|bytes| {
                // BCS layout of rtd_framework::SystemPackage, without introducing a build dependency.
                let (id, bytecode, dependencies): (
                    rtd_types::base_types::ObjectID,
                    Vec<Vec<u8>>,
                    Vec<rtd_types::base_types::ObjectID>,
                ) = bcs::from_bytes(bytes).unwrap();
                let config = rtd_protocol_config::ProtocolConfig::get_for_max_version_UNSAFE()
                    .binary_config(Some(false));
                let modules: Vec<_> = bytecode
                    .into_iter()
                    .map(|bytes| {
                        let package = MovePackage::new(
                            id,
                            1.into(),
                            BTreeMap::from([("module".to_owned(), bytes)]),
                            u64::MAX,
                            vec![],
                            BTreeMap::new(),
                        )
                        .unwrap();
                        package
                            .deserialize_module_by_str("module", &config)
                            .unwrap()
                    })
                    .collect();
                Object::new_system_package(
                    &modules,
                    1.into(),
                    dependencies,
                    rtd_types::digests::TransactionDigest::genesis_marker(),
                )
            })
            .collect()
    }

    pub(crate) fn two_transaction_object_checkpoint() -> Checkpoint {
        TestCheckpointBuilder::new(0)
            .start_transaction(0)
            .create_owned_object(10)
            .finish_transaction()
            .start_transaction(0)
            .create_owned_object(11)
            .finish_transaction()
            .build_checkpoint()
    }

    pub(crate) fn kv_transaction_data(
        checkpoint: &Checkpoint,
        index: usize,
    ) -> rtd_kvstore::TransactionData {
        let transaction = &checkpoint.transactions[index];
        rtd_kvstore::TransactionData {
            digest: transaction.transaction.digest(),
            transaction_data: Some(transaction.transaction.clone()),
            signatures: Some(transaction.signatures.clone()),
            effects: Some(transaction.effects.clone()),
            events: transaction.events.clone(),
            checkpoint_number: checkpoint.summary.sequence_number,
            timestamp: checkpoint.summary.timestamp_ms,
            balance_changes: derive_balance_changes_2(&transaction.effects, &checkpoint.object_set),
            unchanged_loaded_runtime_objects: transaction.unchanged_loaded_runtime_objects.clone(),
        }
    }

    pub(crate) fn canonical_transaction_object_keys(
        checkpoint: &Checkpoint,
        index: usize,
    ) -> Vec<ObjectKey> {
        let transaction = &checkpoint.transactions[index];
        get_transaction_object_set(
            &transaction.transaction,
            &transaction.effects,
            &transaction.unchanged_loaded_runtime_objects,
        )
        .into_iter()
        .collect()
    }

    pub(crate) fn response_object_keys(transaction: &ExecutedTransaction) -> Vec<ObjectKey> {
        let objects = &transaction
            .objects
            .as_ref()
            .expect("objects should be populated when requested")
            .objects;
        assert!(
            !objects.is_empty(),
            "requested object set should be non-empty"
        );
        objects
            .iter()
            .map(|object| {
                ObjectKey(
                    object
                        .object_id
                        .as_deref()
                        .expect("object_id should be populated when requested")
                        .parse()
                        .expect("object_id should be valid"),
                    object
                        .version
                        .expect("version should be populated when requested")
                        .into(),
                )
            })
            .collect()
    }

    pub(crate) fn assert_identity_only_object_mask(transaction: &ExecutedTransaction) {
        let objects = &transaction
            .objects
            .as_ref()
            .expect("objects should be populated when requested")
            .objects;
        assert!(
            !objects.is_empty(),
            "requested object set should be non-empty"
        );
        for object in objects {
            assert!(object.object_id.is_some());
            assert!(object.version.is_some());
            assert!(object.bcs.is_none());
            assert!(object.digest.is_none());
            assert!(object.owner.is_none());
            assert!(object.object_type.is_none());
            assert!(object.has_public_transfer.is_none());
            assert!(object.contents.is_none());
            assert!(object.package.is_none());
            assert!(object.previous_transaction.is_none());
            assert!(object.storage_rebate.is_none());
            assert!(object.json.is_none());
            assert!(object.balance.is_none());
            assert!(object.display.is_none());
        }
    }

    pub(super) const LIST_PIPELINE_METRICS: [&str; 6] = [
        "kv_rpc_response_render_latency_ms",
        "kv_rpc_response_page_bytes",
        "kv_rpc_stream_first_frame_latency_ms",
        "kv_rpc_stream_frame_yield_wait_ms",
        "kv_rpc_stream_watermark_frames_total",
        "kv_rpc_final_stream_poll_wait_ms",
    ];

    pub(super) fn list_histogram<'a>(
        families: &'a [MetricFamily],
        name: &str,
        method: &str,
        resolution: &str,
    ) -> &'a Histogram {
        let family = families
            .iter()
            .find(|family| family.name() == name)
            .unwrap_or_else(|| panic!("metric family {name} not registered"));
        let [metric] = family.get_metric() else {
            panic!("{name} has unexpected series");
        };
        assert!(
            metric
                .get_label()
                .iter()
                .map(|label| (label.name(), label.value()))
                .eq([("method", method), ("resolution", resolution)]),
            "{name} has unexpected labels"
        );
        metric.get_histogram()
    }
    pub(super) fn list_counter<'a>(
        families: &'a [MetricFamily],
        name: &str,
        method: &str,
    ) -> &'a Counter {
        let family = families
            .iter()
            .find(|family| family.name() == name)
            .unwrap_or_else(|| panic!("metric family {name} not registered"));
        let [metric] = family.get_metric() else {
            panic!("{name} has unexpected series");
        };
        assert!(
            metric
                .get_label()
                .iter()
                .map(|label| (label.name(), label.value()))
                .eq([("method", method)]),
            "{name} has unexpected labels"
        );
        metric.get_counter()
    }

    pub(super) fn assert_list_metric_absent(families: &[MetricFamily], name: &str) {
        assert!(
            families.iter().all(|family| family.name() != name),
            "{name} unexpectedly registered"
        );
    }

    pub(super) fn ascending_options() -> QueryOptions {
        let mut options = QueryOptions::default();
        options.limit = Some(10);
        options.ordering = Some(Ordering::Ascending as i32);
        options
    }

    pub(super) async fn query_context(
        method: &'static str,
        checkpoint_hi_exclusive: u64,
    ) -> (QueryContext, tokio::task::JoinHandle<()>) {
        let (ctx, _registry, server) =
            query_context_with_registry(method, checkpoint_hi_exclusive).await;
        (ctx, server)
    }

    pub(super) async fn query_context_with_registry(
        method: &'static str,
        checkpoint_hi_exclusive: u64,
    ) -> (QueryContext, Registry, tokio::task::JoinHandle<()>) {
        let (ctx, registry, _mock, server) =
            query_context_with_mock_and_registry(method, checkpoint_hi_exclusive).await;
        (ctx, registry, server)
    }

    pub(super) async fn query_context_with_mock_and_registry(
        method: &'static str,
        checkpoint_hi_exclusive: u64,
    ) -> (
        QueryContext,
        Registry,
        MockBigtableServer,
        tokio::task::JoinHandle<()>,
    ) {
        let mock = MockBigtableServer::new();
        for object in framework_objects_for_json_tests() {
            mock.insert_row(
                rtd_kvstore::tables::objects::NAME,
                rtd_kvstore::tables::objects::encode_key(&ObjectKey(object.id(), object.version())),
                rtd_kvstore::tables::objects::encode(&object).unwrap(),
            )
            .await;
        }
        let (addr, server) = mock.start().await.expect("start mock BigTable");
        let inner = InnerBigTableClient::new_local(addr.to_string(), "test".to_string())
            .await
            .expect("connect to mock BigTable");

        let registry = Registry::new();
        let metrics = KvRpcMetrics::new(&registry);
        let client =
            BigTableClient::new(inner.clone(), 2, metrics.bigtable_limiter.clone(), method);
        let package_store: Arc<dyn PackageStore> = Arc::new(BigTablePackageStore::new(inner));
        let package_resolver = Arc::new(Resolver::new(package_store));

        let method_config = LedgerHistoryMethodConfig {
            timeout_ms: Some(5_000),
            default_limit_items: Some(10),
            max_limit_items: Some(100),
            render_ahead: Some(2),
        };
        let ledger_history = LedgerHistoryConfig {
            list_transactions: Some(method_config.clone()),
            list_events: Some(method_config.clone()),
            list_checkpoints: Some(method_config),
            bitmap_bucket_budget_tx: Some(10),
            bitmap_bucket_budget_event: Some(10),
            bitmap_drain_probe_rows: None,
            max_bitmap_filter_literals: Some(1),
        };
        let stage = StageConfig {
            chunk_size: Some(2),
            concurrency: Some(1),
        };
        let stages = StagesConfig {
            tx_seq_digest: Some(stage.clone()),
            transactions: Some(stage.clone()),
            objects: Some(stage.clone()),
            checkpoints: Some(stage),
        };

        (
            QueryContext::new(
                client,
                package_resolver,
                metrics,
                method,
                checkpoint_hi_exclusive,
                ledger_history,
                stages,
            ),
            registry,
            mock,
            server,
        )
    }
}

#[cfg(test)]
mod info_admission_tests {
    use super::*;
    use bytes::Bytes;
    use rtd_kvstore::tables::watermarks;
    use rtd_kvstore::testing::{MockBigtableServer, insert_checkpoint_rows};
    use rtd_types::test_checkpoint_data_builder::TestCheckpointBuilder;

    async fn watermark(mock: &MockBigtableServer, name: &str, upper: u64, lower: u64) {
        let values = [
            ("v", 1),
            ("ehi", 3),
            ("chi", upper),
            ("th", 100),
            ("tmhi", 1000),
            ("rl", lower),
            ("ph", 0),
            ("ptm", 0),
        ];
        mock.insert_row(
            watermarks::NAME,
            watermarks::encode_key(name),
            values
                .into_iter()
                .map(|(key, value)| (key, Bytes::copy_from_slice(&value.to_be_bytes()))),
        )
        .await;
    }

    #[tokio::test]
    async fn service_info_reports_common_retention_and_refuses_disjoint_or_missing_ranges() {
        let mock = MockBigtableServer::new();
        let (address, handle) = mock.start().await.unwrap();
        let client = BigTableClient::new_local(address.to_string(), "test".into())
            .await
            .unwrap();
        watermark(&mock, "slow", 10, 1).await;
        watermark(&mock, "pruned", 12, 7).await;
        let info = get_service_info(
            client.clone(),
            ChainIdentifier::default(),
            None,
            &["slow", "pruned"],
        )
        .await
        .unwrap();
        assert_eq!(info.checkpoint_height, Some(10));
        assert_eq!(info.lowest_available_checkpoint, Some(7));
        assert_eq!(info.lowest_available_checkpoint_objects, Some(7));
        assert_eq!(mock.read_rows_calls().await.len(), 1);

        watermark(&mock, "pruned", 12, 11).await;
        for pipelines in [
            vec!["slow", "pruned"],
            vec!["slow", "missing"],
            vec!["slow", "slow"],
            vec![],
        ] {
            assert!(
                common_service_watermark(client.clone(), &pipelines)
                    .await
                    .unwrap()
                    .is_none()
            );
            assert!(
                get_service_info(client.clone(), ChainIdentifier::default(), None, &pipelines)
                    .await
                    .is_err()
            );
        }
        watermark(&mock, "pruned", 12, 10).await;
        let restored = get_service_info(
            client,
            ChainIdentifier::default(),
            None,
            &["slow", "pruned"],
        )
        .await
        .unwrap();
        assert_eq!(
            restored.lowest_available_checkpoint,
            restored.checkpoint_height
        );
        handle.abort();
    }

    #[tokio::test]
    async fn admitted_info_uses_summary_and_common_retained_range() {
        let mock = MockBigtableServer::new();
        let (address, handle) = mock.start().await.unwrap();
        let client = BigTableClient::new_local(address.to_string(), "test".to_string())
            .await
            .unwrap();
        watermark(&mock, "left", 10, 1).await;
        watermark(&mock, "right", 12, 4).await;
        let checkpoint = TestCheckpointBuilder::new(5)
            .with_epoch(2)
            .with_timestamp_ms(500)
            .build_checkpoint();
        insert_checkpoint_rows(&mock, &checkpoint).await;
        let info = get_admitted_service_info(
            client.clone(),
            ChainIdentifier::default(),
            None,
            &["left", "right"],
            5,
        )
        .await
        .unwrap();
        assert_eq!(info.checkpoint_height, Some(5));
        assert_eq!(info.epoch, Some(2));
        assert_eq!(info.timestamp, Some(timestamp_ms_to_proto(500)));
        assert_eq!(info.lowest_available_checkpoint, Some(4));
        assert_eq!(info.lowest_available_checkpoint_objects, Some(4));
        for (pipelines, height) in [
            (vec!["left", "right"], 3),
            (vec!["left", "right"], 6),
            (vec!["left", "missing"], 5),
            (vec![], 5),
        ] {
            let error = get_admitted_service_info(
                client.clone(),
                ChainIdentifier::default(),
                None,
                &pipelines,
                height,
            )
            .await
            .unwrap_err();
            assert_eq!(tonic::Status::from(error).code(), tonic::Code::Unavailable);
        }
        watermark(&mock, "right", 12, 6).await;
        assert!(
            get_admitted_service_info(
                client,
                ChainIdentifier::default(),
                None,
                &["left", "right"],
                5
            )
            .await
            .is_err()
        );
        handle.abort();
    }
}
