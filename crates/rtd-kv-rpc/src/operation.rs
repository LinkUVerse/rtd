// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::future::Future;
use std::time::Duration;

use futures::StreamExt;
use futures::TryStreamExt;
use futures::stream::BoxStream;
use rtd_inverted_index::BitmapQuery;
use rtd_inverted_index::SkipPolicy;
use rtd_kvstore::BitmapIndexSpec;
use rtd_kvstore::tables::event_bitmap_index;
use rtd_kvstore::tables::transaction_bitmap_index;
use rtd_rpc::proto::rtd::rpc::v2::EventFilter;
use rtd_rpc::proto::rtd::rpc::v2::TransactionFilter;
use rtd_rpc_api::RpcError;
use tokio::time::Instant;

use crate::KvRpcMetrics;
use crate::KvRpcServer;
use crate::PackageResolver;
use crate::bigtable_client::BigTableClient;
use crate::config::{LedgerHistoryConfig, PipelineStage, ResolvedStageConfig, StagesConfig};
use rtd_rpc_api::ledger_history::filter::event_filter_to_query;
use rtd_rpc_api::ledger_history::filter::transaction_filter_to_query;

#[derive(Clone, Copy, Debug)]
pub(crate) struct OperationSpec {
    pub(crate) name: &'static str,
    pub(crate) timeout: Duration,
}

impl OperationSpec {
    pub(crate) const fn new(name: &'static str, timeout: Duration) -> Self {
        Self { name, timeout }
    }
}

#[derive(Clone)]
pub(crate) struct QueryContext {
    client: BigTableClient,
    package_resolver: PackageResolver,
    metrics: std::sync::Arc<KvRpcMetrics>,
    method: &'static str,
    checkpoint_hi_exclusive: u64,
    ledger_history: LedgerHistoryConfig,
    stages: StagesConfig,
}

impl QueryContext {
    pub(crate) fn new(
        client: BigTableClient,
        package_resolver: PackageResolver,
        metrics: std::sync::Arc<KvRpcMetrics>,
        method: &'static str,
        checkpoint_hi_exclusive: u64,
        ledger_history: LedgerHistoryConfig,
        stages: StagesConfig,
    ) -> Self {
        Self {
            client,
            package_resolver,
            metrics,
            method,
            checkpoint_hi_exclusive,
            ledger_history,
            stages,
        }
    }

    pub(crate) fn ledger_history(&self) -> &LedgerHistoryConfig {
        &self.ledger_history
    }

    /// Resolved tunables for one read pipeline stage (chunk size + fan-out).
    pub(crate) fn stage(&self, stage: PipelineStage) -> ResolvedStageConfig {
        self.stages.stage(stage)
    }

    pub(crate) fn client(&self) -> &BigTableClient {
        &self.client
    }

    pub(crate) fn package_resolver(&self) -> &PackageResolver {
        &self.package_resolver
    }

    pub(crate) fn checkpoint_hi_exclusive(&self) -> u64 {
        self.checkpoint_hi_exclusive
    }

    /// Per-request evaluated-bucket budget for `spec`. Handlers pass this
    /// into `eval_bitmap_query_stream`, which constructs a
    /// `BitmapScanBudget` internally and reports the resulting
    /// `BitmapScanMetrics` via the `on_metrics` callback. The budget caps
    /// evaluated buckets, not backend reads — see
    /// `eval_bitmap_query_stream` for the (≤ unique_leaf_count) slop at
    /// exhaustion.
    pub(crate) fn scan_budget(&self, spec: BitmapIndexSpec) -> u64 {
        match spec.table_name {
            transaction_bitmap_index::NAME => self.ledger_history.bitmap_bucket_budget_tx(),
            event_bitmap_index::NAME => self.ledger_history.bitmap_bucket_budget_event(),
            other => panic!("unknown bitmap index table {other}; add a budget for it"),
        }
    }
    pub(crate) fn bitmap_skip_policy(&self) -> SkipPolicy {
        self.ledger_history.bitmap_skip_policy()
    }

    pub(crate) fn observe_response_render(
        &self,
        resolution: &'static str,
        elapsed: std::time::Duration,
    ) {
        self.metrics
            .observe_response_render(self.method, resolution, elapsed);
    }

    pub(crate) fn observe_response_page_bytes(&self, resolution: &'static str, bytes: usize) {
        self.metrics
            .observe_response_page_bytes(self.method, resolution, bytes);
    }

    pub(crate) fn observe_stream_first_frame_latency(
        &self,
        resolution: &'static str,
        elapsed: std::time::Duration,
    ) {
        self.metrics
            .observe_stream_first_frame_latency(self.method, resolution, elapsed);
    }

    pub(crate) fn observe_stream_frame_yield_wait(
        &self,
        resolution: &'static str,
        elapsed: std::time::Duration,
    ) {
        self.metrics
            .observe_stream_frame_yield_wait(self.method, resolution, elapsed);
    }

    pub(crate) fn inc_stream_watermark_frames(&self) {
        self.metrics.inc_stream_watermark_frames(self.method);
    }

    pub(crate) async fn next_response_item<T>(
        &self,
        resolution: &'static str,
        stream: &mut BoxStream<'static, T>,
    ) -> Option<T> {
        let poll_started_at = Instant::now();
        let item = stream.next().await;
        self.metrics.observe_final_stream_poll_wait(
            self.method,
            resolution,
            poll_started_at.elapsed(),
        );
        item
    }

    pub(crate) fn transaction_filter_query(
        &self,
        filter: &TransactionFilter,
    ) -> Result<BitmapQuery, RpcError> {
        transaction_filter_to_query(filter, self.ledger_history.max_bitmap_filter_literals())
    }

    pub(crate) fn event_filter_query(&self, filter: &EventFilter) -> Result<BitmapQuery, RpcError> {
        event_filter_to_query(filter, self.ledger_history.max_bitmap_filter_literals())
    }

    /// Callback for `eval_bitmap_query_stream`'s `on_metrics`. Fires exactly
    /// once when the eval pipeline drops and records charged, discarded, and
    /// seek counts for budget and gap-policy tuning.
    pub(crate) fn bitmap_scan_observer(
        &self,
    ) -> impl FnOnce(rtd_inverted_index::BitmapScanMetrics) + Send + 'static {
        let metrics = self.metrics.clone();
        let method = self.method;
        move |m| {
            metrics.observe_bitmap_buckets_evaluated(method, m.buckets_evaluated);
            metrics.observe_bitmap_buckets_discarded(method, m.buckets_discarded);
            metrics.observe_bitmap_leaf_seeks(method, m.leaf_seeks);
            tracing::debug!(
                method,
                buckets_evaluated = m.buckets_evaluated,
                buckets_discarded = m.buckets_discarded,
                leaf_seeks = m.leaf_seeks,
                "bitmap scan complete"
            );
        }
    }
}

impl KvRpcServer {
    pub(crate) fn limited_client(&self, operation: &'static str) -> BigTableClient {
        BigTableClient::new(
            self.client.clone(),
            self.request_bigtable_concurrency,
            self.metrics.bigtable_limiter.clone(),
            operation,
        )
    }

    async fn cached_checkpoint_hi_exclusive(&self) -> Result<u64, RpcError> {
        let checkpoint_hi_inclusive = {
            let cache = self.cache.read().await;
            cache
                .as_ref()
                .filter(|(sampled_at, _)| {
                    sampled_at.elapsed() < tokio::time::Duration::from_secs(1)
                })
                .and_then(|(_, info)| info.checkpoint_height)
        };
        // A slow background sample may expire before it is consumed. Read the
        // current watermark inside the existing request deadline instead of
        // treating cache age as a dependency outage or serving stale height.
        let checkpoint_hi_inclusive = match checkpoint_hi_inclusive {
            Some(height) => height,
            None => crate::v2::common_service_watermark(
                self.client.clone(),
                &self.service_info_watermark_pipelines,
            )
            .await?
            .and_then(|watermark| watermark.checkpoint_hi_inclusive)
            .ok_or_else(|| {
                RpcError::new(
                    tonic::Code::Unavailable,
                    "current service info missing checkpoint height",
                )
            })?,
        };

        checkpoint_hi_inclusive.checked_add(1).ok_or_else(|| {
            RpcError::new(
                tonic::Code::Internal,
                "cached checkpoint height cannot be represented as an exclusive bound",
            )
        })
    }

    async fn query_context(&self, operation: &'static str) -> Result<QueryContext, RpcError> {
        Ok(QueryContext::new(
            self.limited_client(operation),
            self.package_resolver.clone(),
            self.metrics.clone(),
            operation,
            self.cached_checkpoint_hi_exclusive().await?,
            self.ledger_history.clone(),
            self.stages.clone(),
        ))
    }

    pub(crate) async fn serve_query_stream<Req, T, F, Fut>(
        &self,
        spec: OperationSpec,
        request: tonic::Request<Req>,
        handler: F,
    ) -> Result<tonic::Response<BoxStream<'static, Result<T, tonic::Status>>>, tonic::Status>
    where
        Req: Send + 'static,
        T: Send + 'static,
        F: FnOnce(QueryContext, Req) -> Fut + Send,
        Fut: Future<Output = Result<BoxStream<'static, Result<T, RpcError>>, RpcError>> + Send,
    {
        let deadline = Instant::now() + spec.timeout;
        let admission = admission_ceiling(request.metadata())?;
        let request = request.into_inner();

        let stream = tokio::time::timeout_at(deadline, async move {
            let mut ctx = self.query_context(spec.name).await?;
            ctx.checkpoint_hi_exclusive =
                admitted_hi_exclusive(ctx.checkpoint_hi_exclusive, admission);
            if admission.is_some() {
                let height = ctx.checkpoint_hi_exclusive.checked_sub(1).ok_or_else(|| {
                    RpcError::new(tonic::Code::Unavailable, "Admitted query range unavailable")
                })?;
                ctx.package_resolver = self.request_package_resolver(Some(height));
            }
            handler(ctx, request).await
        })
        .await
        .map_err(|_| {
            tracing::warn!(operation = spec.name, "construction phase timed out");
            tonic::Status::deadline_exceeded(format!("{} request deadline exceeded", spec.name))
        })?
        .map_err(tonic::Status::from)?;

        let stream = stream.map_err(tonic::Status::from);
        Ok(tonic::Response::new(
            rtd_rpc_api::grpc::deadline::with_deadline(stream, deadline, spec.name),
        ))
    }
}

// Freeze the request before constructing any scan or yielding response frames.
pub(crate) fn admission_ceiling(
    metadata: &tonic::metadata::MetadataMap,
) -> Result<Option<u64>, tonic::Status> {
    let values = metadata.get_all("x-rtd-visible-checkpoint");
    let mut values = values.iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    let invalid = || tonic::Status::invalid_argument("Invalid publication admission boundary");
    if values.next().is_some() {
        return Err(invalid());
    }
    let value = value.to_str().map_err(|_| invalid())?;
    if value.is_empty()
        || value.len() > 20
        || !value.bytes().all(|byte| byte.is_ascii_digit())
        || (value.len() > 1 && value.starts_with('0'))
    {
        return Err(invalid());
    }
    value.parse().map(Some).map_err(|_| invalid())
}

fn admitted_hi_exclusive(indexed_hi_exclusive: u64, admission: Option<u64>) -> u64 {
    // A maximum uint64 admission covers every representable indexed interval.
    admission
        .and_then(|height| height.checked_add(1))
        .map_or(indexed_hi_exclusive, |hi| indexed_hi_exclusive.min(hi))
}

#[cfg(test)]
mod admission_tests {
    use super::*;

    #[tokio::test]
    async fn expired_service_info_uses_current_watermark_and_rejects_missing_source() {
        use bytes::Bytes;
        use prometheus::Registry;
        use rtd_kvstore::{tables::watermarks, testing::MockBigtableServer};
        use rtd_rpc::proto::rtd::rpc::v2::ledger_service_server::LedgerService;
        let mock = MockBigtableServer::new();
        let (address, handle) = mock.start().await.unwrap();
        let client = rtd_kvstore::BigTableClient::new_local(address.to_string(), "test".into())
            .await
            .unwrap();
        let mut server = KvRpcServer::init(
            client,
            rtd_types::digests::ChainIdentifier::default(),
            None,
            false,
            crate::KvRpcMetrics::new(&Registry::default()),
            crate::config::LedgerHistoryConfig::default(),
            2,
            crate::config::StagesConfig::default(),
        )
        .unwrap();
        server.service_info_watermark_pipelines = vec!["current"];
        let values = [
            ("v", 1_u64),
            ("ehi", 3),
            ("chi", 5),
            ("th", 100),
            ("tmhi", 1000),
            ("rl", 0),
            ("ph", 0),
            ("ptm", 0),
        ];
        mock.insert_row(
            watermarks::NAME,
            watermarks::encode_key("current"),
            values
                .into_iter()
                .map(|(key, value)| (key, Bytes::copy_from_slice(&value.to_be_bytes()))),
        )
        .await;
        let mut stale = rtd_rpc::proto::rtd::rpc::v2::GetServiceInfoResponse::default();
        stale.checkpoint_height = Some(999);
        *server.cache.write().await = Some((
            Instant::now() - tokio::time::Duration::from_secs(2),
            stale.clone(),
        ));
        let info = server
            .get_service_info(tonic::Request::new(Default::default()))
            .await
            .unwrap();
        assert_eq!(info.get_ref().checkpoint_height, Some(5));
        assert_eq!(server.cached_checkpoint_hi_exclusive().await.unwrap(), 6);
        assert_eq!(
            admitted_hi_exclusive(
                server.cached_checkpoint_hi_exclusive().await.unwrap(),
                Some(3)
            ),
            4
        );
        server.service_info_watermark_pipelines = vec!["missing"];
        *server.cache.write().await =
            Some((Instant::now() - tokio::time::Duration::from_secs(2), stale));
        let error = server
            .cached_checkpoint_hi_exclusive()
            .await
            .expect_err("expired success must not mask absent current source");
        assert_eq!(tonic::Status::from(error).code(), tonic::Code::Unavailable);
        handle.abort();
    }

    #[tokio::test]
    async fn poller_failure_clears_cache_and_monotonic_view_then_recovers() {
        use bytes::Bytes;
        use prometheus::Registry;
        use rtd_kvstore::{tables::watermarks, testing::MockBigtableServer};
        use rtd_rpc::proto::rtd::rpc::v2::ledger_service_server::LedgerService;

        let mock = MockBigtableServer::new();
        for pipeline in crate::DEFAULT_SERVICE_INFO_WATERMARK_PIPELINES {
            let values = [
                ("v", 1_u64),
                ("ehi", 3),
                ("chi", 5),
                ("th", 100),
                ("tmhi", 1000),
                ("rl", 0),
                ("ph", 0),
                ("ptm", 0),
            ];
            mock.insert_row(
                watermarks::NAME,
                watermarks::encode_key(pipeline),
                values
                    .into_iter()
                    .map(|(key, value)| (key, Bytes::copy_from_slice(&value.to_be_bytes()))),
            )
            .await;
        }
        let (address, handle) = mock.start().await.unwrap();
        let client = rtd_kvstore::BigTableClient::new_local(address.to_string(), "test".into())
            .await
            .unwrap();
        let server = KvRpcServer::init(
            client,
            rtd_types::digests::ChainIdentifier::default(),
            None,
            false,
            crate::KvRpcMetrics::new(&Registry::default()),
            crate::config::LedgerHistoryConfig::default(),
            2,
            crate::config::StagesConfig::default(),
        )
        .unwrap();

        async fn await_sample(server: &KvRpcServer, expected: Option<u64>) {
            tokio::time::timeout(Duration::from_secs(5), async {
                loop {
                    let cached = server
                        .cache
                        .read()
                        .await
                        .as_ref()
                        .and_then(|(_, info)| info.checkpoint_height);
                    let floor_view = server
                        .min_checkpoint_view
                        .borrow()
                        .map(|(_, height)| height);
                    if cached == expected && floor_view == expected {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .expect("poller must publish the dependency outcome to both views");
        }

        await_sample(&server, Some(5)).await;
        mock.insert_row(
            watermarks::NAME,
            watermarks::encode_key(crate::DEFAULT_SERVICE_INFO_WATERMARK_PIPELINES[0]),
            [("chi", Bytes::from_static(&[0]))],
        )
        .await;
        await_sample(&server, None).await;
        assert!(
            server
                .get_service_info(tonic::Request::new(Default::default()))
                .await
                .is_err()
        );
        assert!(server.cached_checkpoint_hi_exclusive().await.is_err());

        for pipeline in crate::DEFAULT_SERVICE_INFO_WATERMARK_PIPELINES {
            mock.insert_row(
                watermarks::NAME,
                watermarks::encode_key(pipeline),
                [("chi", Bytes::copy_from_slice(&6_u64.to_be_bytes()))],
            )
            .await;
        }
        await_sample(&server, Some(6)).await;
        let info = server
            .get_service_info(tonic::Request::new(Default::default()))
            .await
            .unwrap();
        assert_eq!(info.get_ref().checkpoint_height, Some(6));
        assert_eq!(server.cached_checkpoint_hi_exclusive().await.unwrap(), 7);
        handle.abort();
    }

    #[tokio::test]
    async fn canonical_metadata_and_absent_boundary() {
        let mut metadata = tonic::metadata::MetadataMap::new();
        assert_eq!(admission_ceiling(&metadata).unwrap(), None);
        for value in ["0", "1755", "18446744073709551615"] {
            metadata.insert("x-rtd-visible-checkpoint", value.parse().unwrap());
            assert_eq!(
                admission_ceiling(&metadata).unwrap(),
                Some(value.parse().unwrap())
            );
        }
    }

    #[tokio::test]
    async fn malformed_and_duplicate_boundary_rejected() {
        for value in [
            "",
            "01",
            "-1",
            "+1",
            "1.0",
            " 1",
            "1 ",
            "18446744073709551616",
        ] {
            let mut metadata = tonic::metadata::MetadataMap::new();
            metadata.insert("x-rtd-visible-checkpoint", value.parse().unwrap());
            assert_eq!(
                admission_ceiling(&metadata).unwrap_err().code(),
                tonic::Code::InvalidArgument
            );
        }
        let mut metadata = tonic::metadata::MetadataMap::new();
        metadata.append("x-rtd-visible-checkpoint", "1".parse().unwrap());
        metadata.append("x-rtd-visible-checkpoint", "1".parse().unwrap());
        assert_eq!(
            admission_ceiling(&metadata).unwrap_err().code(),
            tonic::Code::InvalidArgument
        );
    }

    #[tokio::test]
    async fn inclusive_admission_preserves_genesis_and_indexed_limit() {
        assert_eq!(admitted_hi_exclusive(9581, Some(1755)), 1756);
        assert_eq!(admitted_hi_exclusive(9581, Some(0)), 1);
        assert_eq!(admitted_hi_exclusive(9581, Some(10000)), 9581);
        assert_eq!(admitted_hi_exclusive(9581, Some(u64::MAX)), 9581);
        assert_eq!(admitted_hi_exclusive(9581, None), 9581);
        assert_eq!(admitted_hi_exclusive(0, Some(0)), 0);
    }
}
