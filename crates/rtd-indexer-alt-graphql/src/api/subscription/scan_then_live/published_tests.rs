// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::ops::RangeInclusive;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use async_graphql::connection::Edge;
use async_graphql::connection::EmptyFields;
use futures::StreamExt;
use prometheus::Registry;
use rtd_indexer_alt_reader::alpha_ledger_grpc_reader::AlphaLedgerGrpcReader;
use rtd_indexer_alt_reader::alpha_ledger_grpc_reader::PageItem;
use rtd_indexer_alt_reader::alpha_ledger_grpc_reader::StreamPage;
use rtd_indexer_alt_reader::ledger_grpc_reader::LedgerGrpcArgs;
use rtd_rpc::proto::rtd::rpc::v2::QueryEndReason;
use rtd_rpc_cursor::CursorKind;
use rtd_rpc_cursor::CursorToken;
use rtd_rpc_cursor::Position;
use tokio::sync::watch;

use crate::api::types::checkpoint::CCheckpoint;
use crate::error::RpcError;
use crate::pagination::Page;
use crate::task::streaming::ProcessedCheckpoint;
use crate::task::streaming::StreamedCaches;
use crate::task::watermark::Watermarks;

use super::Scanned;
use super::Subscribable;
use super::published_backfill;
use super::tests::caches;

#[derive(Default)]
struct Scan {
    calls: Mutex<Vec<(u64, u64, Option<u64>)>>,
    forged_coverage: bool,
}

struct Feed;

impl Subscribable for Feed {
    type Item = u64;
    type Cursor = CCheckpoint;
    type Filter = Arc<Scan>;
    type ScanItem = u64;

    async fn scan<'a>(
        _: &'a AlphaLedgerGrpcReader,
        bounds: RangeInclusive<u64>,
        page: &'a Page<Self::Cursor>,
        filter: &'a Self::Filter,
    ) -> Result<StreamPage<u64>, RpcError> {
        let after = page
            .after()
            .map(|cursor| CursorToken::from(cursor).position.checkpoint());
        filter
            .calls
            .lock()
            .unwrap()
            .push((*bounds.start(), *bounds.end(), after));
        let height = if filter.forged_coverage {
            bounds.end() + 1
        } else {
            after.map(|height| height + 1).unwrap_or(*bounds.start())
        };
        let token = |kind| {
            CursorToken {
                kind,
                position: Position::Checkpoints { checkpoint: height },
            }
            .encode()
        };
        // Checkpoint 5 deliberately has no match. Its coverage cursor must still
        // be retained when publication advances, without repeating older items.
        let items = if height == 5 || filter.forged_coverage {
            vec![]
        } else {
            vec![PageItem {
                cursor: token(CursorKind::Item),
                payload: height,
            }]
        };
        Ok(StreamPage::for_test(
            items,
            None,
            Some(token(CursorKind::Boundary)),
            Some(QueryEndReason::ItemLimit),
        ))
    }

    fn build_node(
        _: &Arc<StreamedCaches>,
        _: &rtd_package_resolver::Limits,
        value: &u64,
    ) -> Result<u64, RpcError> {
        Ok(*value)
    }

    fn matching_edges(
        _: &Arc<ProcessedCheckpoint>,
        _: &Arc<StreamedCaches>,
        _: &rtd_package_resolver::Limits,
        _: &Self::Filter,
    ) -> Result<Vec<Edge<String, u64, EmptyFields>>, RpcError> {
        panic!("certified backfill must not read the unaudited live broadcast")
    }

    fn subscription_type() -> &'static str {
        "fixture"
    }
}

async fn reader() -> AlphaLedgerGrpcReader {
    AlphaLedgerGrpcReader::new(
        "http://127.0.0.1:9".parse().unwrap(),
        LedgerGrpcArgs::default(),
        None,
        &Registry::new(),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn published_pages_preserve_empty_coverage_and_resume_the_exact_cursor() {
    let scan = Arc::new(Scan::default());
    let (_watermarks, indexed) =
        watch::channel(Arc::new(Watermarks::for_test(&[("kv_packages", 20)])));
    let (frontier, follow) = watch::channel(5);
    let stream = published_backfill::<Feed>(
        reader().await,
        caches().await,
        crate::config::Limits::default().package_resolver(),
        indexed,
        scan.clone(),
        None,
        3,
        1,
        follow,
    );
    futures::pin_mut!(stream);
    let mut delivered = Vec::new();
    for target in [5, 7] {
        frontier.send_replace(target);
        loop {
            let item = tokio::time::timeout(Duration::from_secs(2), stream.next())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            match item {
                Scanned::Match { edge, .. } => delivered.push(edge.node),
                Scanned::Covered { checkpoint } if checkpoint == target => break,
                Scanned::Covered { .. } => {}
            }
        }
        let count = scan.calls.lock().unwrap().len();
        assert!(
            tokio::time::timeout(Duration::from_millis(40), stream.next())
                .await
                .is_err()
        );
        assert_eq!(scan.calls.lock().unwrap().len(), count);
    }
    assert_eq!(delivered, [3, 4, 6, 7]);
    assert_eq!(
        *scan.calls.lock().unwrap(),
        [
            (3, 5, None),
            (3, 5, Some(3)),
            (3, 5, Some(4)),
            (3, 7, Some(5)),
            (3, 7, Some(6))
        ]
    );
    drop(frontier);
    assert!(
        tokio::time::timeout(Duration::from_secs(2), stream.next())
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn future_coverage_cursor_is_rejected_even_without_a_business_item() {
    let scan = Arc::new(Scan {
        forged_coverage: true,
        ..Default::default()
    });
    let (_watermarks, indexed) =
        watch::channel(Arc::new(Watermarks::for_test(&[("kv_packages", 20)])));
    let (_frontier, follow) = watch::channel(5);
    let stream = published_backfill::<Feed>(
        reader().await,
        caches().await,
        crate::config::Limits::default().package_resolver(),
        indexed,
        scan,
        None,
        3,
        1,
        follow,
    );
    futures::pin_mut!(stream);
    assert!(
        tokio::time::timeout(Duration::from_secs(2), stream.next())
            .await
            .unwrap()
            .unwrap()
            .is_err()
    );
    assert!(stream.next().await.is_none());
}
