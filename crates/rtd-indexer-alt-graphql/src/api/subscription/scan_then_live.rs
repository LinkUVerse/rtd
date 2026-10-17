// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Scan-then-live subscription driver: stream the items matching a filter, in checkpoint order,
//! resuming from a cursor or checkpoint, in two phases:
//!
//! 1. Backfill ([`backfill`]): scan indexed data forward from the resume point to catch up, using
//!    the same pagination path.
//! 2. Live ([`live`]): read matching items from the checkpoint broadcast.
//!
//! Once the scan comes within `handoff_threshold` of the tip, the driver subscribes to the broadcast,
//! so reading is handed off to the live phase at `handoff + 1` with no gap or duplicate. A coverage
//! marker at each page's end lets the handoff advance even through stretches that match nothing.

use std::future::Future;
use std::ops::RangeInclusive;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context as _;
use async_graphql::OutputType;
use async_graphql::connection::CursorType;
use async_graphql::connection::Edge;
use async_graphql::connection::EmptyFields;
use async_stream::stream;
use backoff::ExponentialBackoff;
use futures::Stream;
use rtd_indexer_alt_reader::alpha_ledger_grpc_reader::AlphaLedgerGrpcReader;
use rtd_indexer_alt_reader::alpha_ledger_grpc_reader::StreamPage;
use rtd_rpc_cursor::CursorKind;
use rtd_rpc_cursor::CursorToken;
use tokio::sync::broadcast;
use tokio::sync::watch;
use tracing::warn;

use crate::config::SubscriptionConfig;
use crate::error::RpcError;
use crate::error::bad_user_input;
use crate::error::upcast;
use crate::pagination::Page;
use crate::pagination::PageLimits;
use crate::task::streaming::CheckpointBroadcaster;
use crate::task::streaming::ProcessedCheckpoint;
use crate::task::streaming::StreamedCaches;
use crate::task::streaming::SubscriberLimit;
use crate::task::streaming::SubscriptionBroadcast;
use crate::task::streaming::SubscriptionLifecycleGuard;
use crate::task::streaming::SubscriptionTerminationReason;
use crate::task::streaming::broadcast_error;
use crate::task::streaming::reconnect_error;
use crate::task::streaming::wait_for_pipelines_catching_up_at;
use crate::task::watermark::Watermarks;

use super::Error;

#[cfg(test)]
mod published_tests;

/// How long to wait before re-requesting when the scan has drained the indexer's current tip but not
/// yet reached the handoff.
const BACKFILL_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// A feed that can be streamed with the scan-then-live driver. One implementation per subscribable
/// GraphQL type (transactions, events) supplies its cursor, filter, and matching, and the free
/// functions here drive the shared machinery over it.
pub(super) trait Subscribable {
    /// The GraphQL node delivered in each edge.
    type Item: OutputType + Send + 'static;
    /// The opaque cursor minted for resumption.
    type Cursor: CursorType
        + Eq
        + Clone
        + Send
        + Sync
        + TryFrom<CursorToken, Error = anyhow::Error>
        + 'static;
    /// The filter selecting matching items.
    type Filter: Clone + Send + Sync + 'static;
    /// The raw scan payload one page yields before conversion to edges.
    type ScanItem: Send + 'static;

    /// Scan one page of matches over `cp_bounds`, starting after `page`'s cursor.
    fn scan<'a>(
        reader: &'a AlphaLedgerGrpcReader,
        cp_bounds: RangeInclusive<u64>,
        page: &'a Page<Self::Cursor>,
        filter: &'a Self::Filter,
    ) -> impl Future<Output = Result<StreamPage<Self::ScanItem>, RpcError>> + Send + 'a;

    /// Build the GraphQL node from one scanned item's payload, constructing whatever scope the feed
    /// resolves against. The driver mints the cursor from the item's position and assembles the edge.
    fn build_node(
        caches: &Arc<StreamedCaches>,
        resolver_limits: &rtd_package_resolver::Limits,
        payload: &Self::ScanItem,
    ) -> Result<Self::Item, RpcError>;

    /// The filter-matching edges in one live checkpoint, in delivery order.
    fn matching_edges(
        checkpoint: &Arc<ProcessedCheckpoint>,
        caches: &Arc<StreamedCaches>,
        resolver_limits: &rtd_package_resolver::Limits,
        filter: &Self::Filter,
    ) -> Result<Vec<Edge<String, Self::Item, EmptyFields>>, RpcError>;

    /// The feed's name, used to label its lifecycle metrics.
    fn subscription_type() -> &'static str;
}

/// One backfill scan output: either a matching edge, or a coverage marker whose `checkpoint` is the
/// fully-scanned frontier (advances the handoff even on a match-less page).
enum Scanned<I: OutputType> {
    Match {
        checkpoint: u64,
        edge: Edge<String, I, EmptyFields>,
    },
    Covered {
        checkpoint: u64,
    },
}

impl<I: OutputType> Scanned<I> {
    /// The checkpoint this output sits at (the match's, or the covered frontier).
    fn checkpoint(&self) -> u64 {
        match self {
            Scanned::Match { checkpoint, .. } | Scanned::Covered { checkpoint } => *checkpoint,
        }
    }
}

/// Subscribe to items matching `filter`: backfill from `resume` toward the tip, then follow live.
/// The handoff is pinned mid-scan, once the scan frontier comes within `handoff_threshold` of the
/// tip, so even a deep backfill catches up within one connection instead of lagging the receiver.
#[allow(clippy::type_complexity)]
pub(super) fn subscribe<S: Subscribable>(
    reader: AlphaLedgerGrpcReader,
    broadcast: Arc<SubscriptionBroadcast>,
    caches: Arc<StreamedCaches>,
    resolver_limits: rtd_package_resolver::Limits,
    watermarks_rx: watch::Receiver<Arc<Watermarks>>,
    filter: S::Filter,
    after: Option<S::Cursor>,
    after_checkpoint: Option<u64>,
    subscriber_limit: SubscriberLimit,
    config: SubscriptionConfig,
    admission: Option<u64>,
    follow: Option<watch::Receiver<u64>>,
) -> Result<impl Stream<Item = Result<Edge<String, S::Item, EmptyFields>, RpcError>>, RpcError<Error>>
where
    for<'a> CursorToken: From<&'a S::Cursor>,
{
    // The checkpoint to resume after: the later of the `after` cursor and `afterCheckpoint`. A
    // cursor's checkpoint is read through its `CursorToken` position, so the driver needs no
    // per-feed accessor.
    let start_from = match (
        after
            .as_ref()
            .map(|c| CursorToken::from(c).position.checkpoint()),
        after_checkpoint,
    ) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (a, b) => a.or(b),
    };

    if admission.is_some_and(|height| start_from.is_some_and(|start| start > height)) {
        return Err(upcast(reconnect_error()));
    }

    // Reject a far-ahead resume before admitting, so bad input never claims a slot or lands in the
    // subscription lifecycle metrics.
    reject_if_start_too_far_ahead(start_from, &broadcast, &config)?;

    // Claim a concurrency slot for the subscription's lifetime; at capacity this is `Err` with no
    // guard created (and so no lifecycle metric).
    let guard = SubscriptionLifecycleGuard::new(
        S::subscription_type(),
        broadcast.metrics(),
        &subscriber_limit,
    )
    .map_err(upcast)?;

    // Size the backfill scan page to the resolve concurrency. Scans are sequential (each needs the
    // previous page's cursor), so feeding one window of `n` concurrent resolutions takes ceil(n /
    // page) scans: a page much smaller than the concurrency makes scanning the bottleneck, a much
    // larger one just holds a bigger page in memory. Matching them is roughly one scan per window.
    let scan_page_size = config.max_concurrent_resolutions;

    // Pin the handoff once the scan comes within half the live buffer of the tip, leaving room for
    // checkpoints that arrive during the handoff so the receiver does not lag.
    let handoff_threshold = config.broadcast_buffer as u64 / 2;

    Ok(stream! {
        // Certified archival streams follow publication, rather than exposing the
        // Full Node's unaudited tip. The same scan cursor preserves every item.
        if let Some(follow) = follow {
            let checkpoint_lo = after_checkpoint.map(|cp| cp.saturating_add(1))
                .unwrap_or_else(|| if after.is_some() { 0 } else { follow.borrow().saturating_add(1) });
            let scan = published_backfill::<S>(reader, caches, resolver_limits,
                watermarks_rx, filter, after, checkpoint_lo, scan_page_size, follow);
            for await item in scan {
                match item {
                    Ok(Scanned::Match { edge, .. }) => {
                        yield Ok(edge);
                        guard.record_backfill_delivered();
                    }
                    Ok(Scanned::Covered { .. }) => {}
                    Err(error) => {
                        guard.terminate(SubscriptionTerminationReason::BackfillError);
                        yield Err(error);
                        return;
                    }
                }
            }
            return;
        }
        let mut pending_receiver = None;
        let mut handoff: Option<u64> = None;
        let mut last_checkpoint: Option<u64> = None;

        // Phase 1: scan toward the tip, pinning the live receiver near it. `after` and/or
        // `afterCheckpoint` resume the backfill; neither means live-only.
        if after.is_some() || after_checkpoint.is_some() {
            let checkpoint_lo = after_checkpoint.map_or(0, |cp| cp.saturating_add(1));
            let scan = backfill::<S>(
                reader,
                caches.clone(),
                resolver_limits.clone(),
                watermarks_rx,
                filter.clone(),
                after,
                checkpoint_lo,
                scan_page_size,
                admission,
            );
            for await scanned in scan {
                let scanned = match scanned {
                    Ok(scanned) => scanned,
                    Err(e) => {
                        guard.terminate(SubscriptionTerminationReason::BackfillError);
                        yield Err(e);
                        return;
                    }
                };

                // Resubscribe-first and pin once the scan frontier is within threshold of the tip.
                if pending_receiver.is_none()
                    && broadcast.network_tip().saturating_sub(scanned.checkpoint()) <= handoff_threshold
                {
                    pending_receiver = Some(broadcast.broadcaster().resubscribe());
                    handoff = Some(broadcast.network_tip());
                }

                match scanned {
                    // A match: deliver it, unless it is past the handoff (then it is live's).
                    Scanned::Match { checkpoint, edge } => {
                        if handoff.is_some_and(|h| checkpoint > h) {
                            break;
                        }
                        yield Ok(edge);
                        guard.record_backfill_delivered();
                    }
                    // A coverage marker (its `checkpoint` is the fully-scanned frontier): stop once
                    // it has covered the handoff.
                    Scanned::Covered { checkpoint } => {
                        if admission.is_some_and(|height| checkpoint >= height) {
                            guard.terminate(SubscriptionTerminationReason::BackfillError);
                            yield Err(reconnect_error());
                            return;
                        }
                        if handoff.is_some_and(|h| checkpoint >= h) {
                            break;
                        }
                    }
                }
            }
            last_checkpoint = handoff;
        }

        // Phase 2: follow live from `handoff + 1` (a fresh receiver if there was no backfill).
        let receiver = pending_receiver.unwrap_or_else(|| broadcast.broadcaster().resubscribe());
        for await edge in
            live::<S>(receiver, last_checkpoint, caches, resolver_limits, filter, guard, admission)
        {
            yield edge;
        }
    })
}

/// Reject a start point sitting more than `max_ahead` checkpoints past the chain tip. There is
/// nothing to backfill ahead of the tip, so such a request would only wait for the chain to reach
/// it, and a far-future one would hold the connection open indefinitely.
fn reject_if_start_too_far_ahead(
    start_from: Option<u64>,
    broadcast: &SubscriptionBroadcast,
    config: &SubscriptionConfig,
) -> Result<(), RpcError<Error>> {
    let max_ahead = config.max_start_checkpoints_ahead_of_tip;
    if let Some(start) = start_from
        && start > broadcast.network_tip().saturating_add(max_ahead)
    {
        return Err(bad_user_input(Error::TooFarAheadOfTip { max: max_ahead }));
    }
    Ok(())
}

/// Scan matches from the resume point toward the tip, yielding each match and a per-page coverage
/// marker. Open-ended: the caller stops it once the handoff is covered.
fn backfill<S: Subscribable>(
    reader: AlphaLedgerGrpcReader,
    caches: Arc<StreamedCaches>,
    resolver_limits: rtd_package_resolver::Limits,
    mut watermarks_rx: watch::Receiver<Arc<Watermarks>>,
    filter: S::Filter,
    mut after: Option<S::Cursor>,
    checkpoint_lo: u64,
    scan_page_size: usize,
    admission: Option<u64>,
) -> impl Stream<Item = Result<Scanned<S::Item>, RpcError>> {
    stream! {
        let limits = PageLimits {
            default: scan_page_size as u32,
            max: scan_page_size as u32,
        };

        // Resume from whichever is later: the `after` cursor or `[checkpoint_lo, ..]`. The upper
        // bound is `u64::MAX` because backfill is open-ended, chasing the moving tip; `subscribe`
        // stops it once the handoff is covered.
        let ceiling = admission.unwrap_or(u64::MAX);
        if checkpoint_lo > ceiling {
            yield Err(reconnect_error());
            return;
        }
        let cp_bounds = checkpoint_lo..=ceiling;

        loop {
            let (items, next_after) = match scan_page::<S>(
                &reader,
                &caches,
                &resolver_limits,
                &limits,
                cp_bounds.clone(),
                after.as_ref(),
                &filter,
                &mut watermarks_rx,
            )
            .await
            {
                Ok(page) => page,
                Err(e) => {
                    yield Err(e);
                    return;
                }
            };
            let covered = items.iter().any(|item| matches!(item, Scanned::Covered { checkpoint } if *checkpoint >= ceiling));
            for item in items {
                yield Ok(item);
            }
            if admission.is_some() && covered {
                yield Err(reconnect_error());
                return;
            }
            after = Some(next_after);
        }
    }
}

/// Wait at the certified frontier, then continue from the exact scan cursor.
fn published_backfill<S: Subscribable>(
    reader: AlphaLedgerGrpcReader,
    caches: Arc<StreamedCaches>,
    resolver_limits: rtd_package_resolver::Limits,
    mut watermarks_rx: watch::Receiver<Arc<Watermarks>>,
    filter: S::Filter,
    mut after: Option<S::Cursor>,
    checkpoint_lo: u64,
    scan_page_size: usize,
    mut follow: watch::Receiver<u64>,
) -> impl Stream<Item = Result<Scanned<S::Item>, RpcError>> {
    stream! {
        let limits = PageLimits { default: scan_page_size as u32, max: scan_page_size as u32 };
        let mut covered = None;
        loop {
            let ceiling = *follow.borrow_and_update();
            if checkpoint_lo > ceiling || covered.is_some_and(|cp| cp >= ceiling) {
                if follow.changed().await.is_err() { return; }
                continue;
            }
            let (items, next_after) = match scan_page::<S>(&reader, &caches,
                &resolver_limits, &limits, checkpoint_lo..=ceiling,
                after.as_ref(), &filter, &mut watermarks_rx).await {
                Ok(page) => page,
                Err(error) => { yield Err(error); return; }
            };
            for item in items {
                if let Scanned::Covered { checkpoint } = &item { covered = Some(*checkpoint); }
                yield Ok(item);
            }
            after = Some(next_after);
        }
    }
}

/// The highest checkpoint fully scanned as of `token`: its own checkpoint for a `Boundary`, one less
/// for an `Item` (whose checkpoint may still hold later matches).
fn covered_checkpoint(token: &CursorToken) -> u64 {
    let checkpoint = token.position.checkpoint();
    match token.kind {
        CursorKind::Boundary => checkpoint,
        CursorKind::Item => checkpoint.saturating_sub(1),
    }
}

/// Follow the live broadcast from `last_checkpoint + 1`, delivering matching items. Drops the
/// one-checkpoint seam overlap (the resubscribe/tip race), gap-checks, and disconnects on anomalies.
fn live<S: Subscribable>(
    mut receiver: CheckpointBroadcaster,
    mut last_checkpoint: Option<u64>,
    caches: Arc<StreamedCaches>,
    resolver_limits: rtd_package_resolver::Limits,
    filter: S::Filter,
    guard: SubscriptionLifecycleGuard,
    admission: Option<u64>,
) -> impl Stream<Item = Result<Edge<String, S::Item, EmptyFields>, RpcError>> {
    stream! {
        let mut delivered_live = false;
        loop {
            if admission.is_some_and(|height| last_checkpoint.is_some_and(|last| last >= height)) {
                guard.terminate(SubscriptionTerminationReason::Error);
                yield Err(reconnect_error());
                return;
            }
            match receiver.recv().await {
                Ok(checkpoint) => {
                    let seq = checkpoint.summary.sequence_number;
                    if admission.is_some_and(|height| seq > height) {
                        guard.terminate(SubscriptionTerminationReason::Error);
                        yield Err(reconnect_error());
                        return;
                    }
                    if let Some(last) = last_checkpoint {
                        // Already covered by the scan (resubscribe/tip overlap): skip.
                        if seq <= last {
                            continue;
                        }
                        if seq > last + 1 {
                            warn!(
                                last_checkpoint = last,
                                received = seq,
                                "Unexpected gap between scan and live; disconnecting"
                            );
                            guard.terminate(SubscriptionTerminationReason::UnexpectedGap);
                            yield Err(reconnect_error());
                            return;
                        }
                    }
                    // Deliver each matching item as its own payload, ordered within the checkpoint.
                    // Empty checkpoints yield nothing.
                    let edges = match S::matching_edges(&checkpoint, &caches, &resolver_limits, &filter) {
                        Ok(edges) => edges,
                        Err(e) => {
                            guard.terminate(SubscriptionTerminationReason::Error);
                            yield Err(e);
                            return;
                        }
                    };
                    for edge in edges {
                        yield Ok(edge);
                        guard.record_delivered(checkpoint.summary.timestamp_ms);
                    }
                    last_checkpoint = Some(seq);
                    delivered_live = true;
                }
                // A lag before the first live checkpoint is catch-up overflow (likely kv-rpc lag).
                Err(broadcast::error::RecvError::Lagged(missed)) if !delivered_live => {
                    warn!(missed, "Subscriber fell behind during catch-up; disconnecting");
                    guard.terminate(SubscriptionTerminationReason::Lagged);
                    yield Err(reconnect_error());
                    return;
                }
                Err(e) => {
                    guard.terminate(SubscriptionTerminationReason::from_recv_error(&e));
                    yield Err(broadcast_error(e));
                    return;
                }
            }
        }
    }
}

/// Retry policy for the backfill scan: jittered exponential backoff, giving up after 60s (past which
/// a failure is unlikely to be a transient blip).
fn scan_backoff() -> ExponentialBackoff {
    ExponentialBackoff {
        initial_interval: Duration::from_millis(100),
        max_interval: Duration::from_secs(5),
        max_elapsed_time: Some(Duration::from_secs(60)),
        ..Default::default()
    }
}

/// Scan the next page of matches, waiting at the indexer tip if nothing is ready yet. Returns the
/// matches, a trailing coverage marker, and the cursor to resume from.
async fn scan_page<S: Subscribable>(
    reader: &AlphaLedgerGrpcReader,
    caches: &Arc<StreamedCaches>,
    resolver_limits: &rtd_package_resolver::Limits,
    limits: &PageLimits,
    cp_bounds: RangeInclusive<u64>,
    after: Option<&S::Cursor>,
    filter: &S::Filter,
    watermarks_rx: &mut watch::Receiver<Arc<Watermarks>>,
) -> Result<(Vec<Scanned<S::Item>>, S::Cursor), RpcError> {
    loop {
        let result = scan_with_retry::<S>(reader, limits, cp_bounds.clone(), after, filter).await?;

        // No end cursor means nothing past the indexer tip is indexed yet; wait, then re-request. A
        // range that was scanned but matched nothing still returns a boundary cursor, not `None`.
        let Some(end_cursor) = result.last_cursor() else {
            if *cp_bounds.end() != u64::MAX
                && watermarks_rx.borrow().initialized()
                && watermarks_rx.borrow().high_watermark().checkpoint() >= *cp_bounds.end()
            {
                return Err(reconnect_error());
            }
            tokio::time::sleep(BACKFILL_POLL_INTERVAL).await;
            continue;
        };
        let end_cursor = CursorToken::decode(end_cursor).context("Failed to decode scan cursor")?;
        if end_cursor.position.checkpoint() > *cp_bounds.end() {
            return Err(reconnect_error());
        }

        // Hold the page until all pipelines have indexed through its end.
        wait_for_pipelines_catching_up_at(
            end_cursor.position.checkpoint().min(*cp_bounds.end()),
            watermarks_rx,
        )
        .await?;

        // A forward page keeps `items` in ascending order, one delivered edge each. The decoded
        // token yields both the checkpoint and the edge cursor, so it is only decoded once.
        let mut items = Vec::with_capacity(result.items.len() + 1);
        for item in &result.items {
            let token =
                CursorToken::decode(&item.cursor).context("Failed to decode scan item cursor")?;
            let checkpoint = token.position.checkpoint();
            if !cp_bounds.contains(&checkpoint) {
                return Err(reconnect_error());
            }
            let cursor = S::Cursor::try_from(token)
                .context("Unexpected position in scan item cursor")?
                .encode_cursor();
            items.push(Scanned::Match {
                checkpoint,
                edge: Edge::new(
                    cursor,
                    S::build_node(caches, resolver_limits, &item.payload)?,
                ),
            });
        }
        // Coverage marker at the fully-scanned end: advances the handoff even on a match-less page.
        items.push(Scanned::Covered {
            checkpoint: covered_checkpoint(&end_cursor),
        });

        let next_after =
            S::Cursor::try_from(end_cursor).context("Unexpected position in scan end cursor")?;
        return Ok((items, next_after));
    }
}

/// Scan one page over `cp_bounds`, retrying transient failures under a bounded budget.
async fn scan_with_retry<S: Subscribable>(
    reader: &AlphaLedgerGrpcReader,
    limits: &PageLimits,
    cp_bounds: RangeInclusive<u64>,
    after: Option<&S::Cursor>,
    filter: &S::Filter,
) -> Result<StreamPage<S::ScanItem>, RpcError> {
    let page = Page::from_params(
        limits,
        Some(limits.default as u64),
        after.cloned(),
        None,
        None,
    )
    .map_err(anyhow::Error::from)?;

    backoff::future::retry(scan_backoff(), || async {
        S::scan(reader, cp_bounds.clone(), &page, filter)
            .await
            .map_err(|e| {
                warn!(error = ?e, "scan failed, retrying");
                backoff::Error::transient(e)
            })
    })
    .await
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use prometheus::Registry;
    use rtd_indexer_alt_reader::ledger_grpc_reader::LedgerGrpcArgs;
    use rtd_indexer_alt_reader::package_resolver::DbPackageStore;
    use rtd_indexer_alt_reader::package_resolver::PackageCache;
    use rtd_indexer_alt_reader::pg_reader::PgReader;
    use rtd_rpc::field::FieldMask;
    use rtd_rpc::field::FieldMaskTree;
    use rtd_rpc::field::FieldMaskUtil;
    use rtd_rpc::merge::Merge;
    use rtd_types::test_checkpoint_data_builder::TestCheckpointBuilder;

    use crate::api::types::checkpoint::Checkpoint;
    use crate::metrics::SubscriptionMetrics;
    use crate::task::streaming::StreamedObjectStore;
    use crate::task::streaming::StreamedTransactionStore;
    use crate::task::streaming::StreamingPackageStore;
    use crate::task::streaming::process_checkpoint;

    use super::*;

    pub(super) async fn caches() -> Arc<StreamedCaches> {
        let reader = PgReader::new(None, None, Default::default(), &Registry::new())
            .await
            .unwrap();
        let loader = Arc::new(reader.as_data_loader());
        Arc::new(
            StreamedCaches::new(
                Arc::new(StreamingPackageStore::new(Arc::new(PackageCache::new(
                    DbPackageStore::new(loader.clone()),
                )))),
                Arc::new(StreamedTransactionStore::new()),
                Arc::new(StreamedObjectStore::new()),
            )
            .with_package_loader(loader),
        )
    }

    fn checkpoint(height: u64) -> Arc<ProcessedCheckpoint> {
        let source = TestCheckpointBuilder::new(height).build_checkpoint();
        let proto = rtd_rpc::proto::rtd::rpc::v2::Checkpoint::merge_from(
            &source,
            &FieldMaskTree::from(FieldMask::from_paths(["*"])),
        );
        Arc::new(process_checkpoint(proto).unwrap())
    }

    #[tokio::test]
    async fn live_ceiling_stops_before_future_payload_and_reconnect_resumes() {
        let caches = caches().await;
        for (previous, ceiling, sent_through, expected) in [
            (None, Some(5), 7, vec![4, 5]),
            (None, Some(5), 5, vec![4, 5]),
            (Some(5), Some(6), 6, vec![6]),
            (Some(5), Some(5), 3, vec![]),
            (None, None, 7, vec![4, 5, 6]),
        ] {
            let (sender, receiver) = broadcast::channel(8);
            for height in 4..=sent_through {
                assert!(sender.send(checkpoint(height)).is_ok());
            }
            let metrics = SubscriptionMetrics::new(&Registry::new());
            let guard =
                SubscriptionLifecycleGuard::new("checkpoints", &metrics, &SubscriberLimit::new(1))
                    .unwrap();
            let stream = live::<Checkpoint>(
                receiver,
                previous,
                caches.clone(),
                crate::config::Limits::default().package_resolver(),
                (),
                guard,
                ceiling,
            );
            futures::pin_mut!(stream);
            for height in expected {
                assert_eq!(
                    stream.next().await.unwrap().unwrap().node.sequence_number,
                    height
                );
            }
            if ceiling.is_some() {
                assert!(
                    tokio::time::timeout(Duration::from_secs(1), stream.next())
                        .await
                        .unwrap()
                        .unwrap()
                        .is_err()
                );
                assert!(stream.next().await.is_none());
            } else {
                assert_eq!(
                    stream.next().await.unwrap().unwrap().node.sequence_number,
                    7
                );
            }
        }
    }

    #[tokio::test]
    async fn backfill_beyond_admission_ends_before_any_scan() {
        let reader = AlphaLedgerGrpcReader::new(
            "http://127.0.0.1:9".parse().unwrap(),
            LedgerGrpcArgs::default(),
            None,
            &Registry::new(),
        )
        .await
        .unwrap();
        let (_sender, receiver) = watch::channel(Arc::new(Watermarks::default()));
        let stream = backfill::<Checkpoint>(
            reader,
            caches().await,
            crate::config::Limits::default().package_resolver(),
            receiver,
            (),
            None,
            6,
            2,
            Some(5),
        );
        futures::pin_mut!(stream);
        let item = tokio::time::timeout(Duration::from_secs(1), stream.next())
            .await
            .unwrap();
        assert!(item.unwrap().is_err());
        assert!(stream.next().await.is_none());
    }
    #[tokio::test]
    async fn published_feed_waits_without_querying_above_frontier_and_closes_on_owner_drop() {
        let reader = AlphaLedgerGrpcReader::new(
            "http://127.0.0.1:9".parse().unwrap(),
            LedgerGrpcArgs::default(),
            None,
            &Registry::new(),
        )
        .await
        .unwrap();
        let (_watermarks, receiver) = watch::channel(Arc::new(Watermarks::default()));
        let (frontier, follow) = watch::channel(5);
        let stream = published_backfill::<Checkpoint>(
            reader,
            caches().await,
            crate::config::Limits::default().package_resolver(),
            receiver,
            (),
            None,
            6,
            2,
            follow,
        );
        futures::pin_mut!(stream);
        assert!(
            tokio::time::timeout(Duration::from_millis(50), stream.next())
                .await
                .is_err()
        );
        frontier.send_replace(5);
        assert!(
            tokio::time::timeout(Duration::from_millis(50), stream.next())
                .await
                .is_err()
        );
        drop(frontier);
        assert!(
            tokio::time::timeout(Duration::from_secs(1), stream.next())
                .await
                .unwrap()
                .is_none()
        );
    }
}
