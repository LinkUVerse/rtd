// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use async_graphql::ServerError;
use axum::http::HeaderMap;
use rtd_indexer_alt_reader::kv_loader::KvLoader;

use crate::task::watermark::Watermarks;

#[derive(Clone, Copy)]
pub(crate) struct SubscriptionAdmission(pub Option<u64>);

#[derive(Clone)]
pub(crate) struct PublicationFollow(pub tokio::sync::watch::Receiver<u64>);

const HEADER: &str = "x-rtd-visible-checkpoint";

pub(crate) fn ceiling(headers: &HeaderMap) -> Result<Option<u64>, ServerError> {
    let mut values = headers.get_all(HEADER).iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    let invalid = || ServerError::new("Invalid publication admission boundary", None);
    if values.next().is_some() {
        return Err(invalid());
    }
    let text = value.to_str().map_err(|_| invalid())?;
    if text.is_empty()
        || text.len() > 20
        || !text.bytes().all(|b| b.is_ascii_digit())
        || (text.len() > 1 && text.starts_with('0'))
    {
        return Err(invalid());
    }
    text.parse::<u64>().map(Some).map_err(|_| invalid())
}

pub(crate) async fn watermarks(
    headers: &HeaderMap,
    current: Arc<Watermarks>,
    loader: &KvLoader,
) -> Result<Arc<Watermarks>, ServerError> {
    let ceiling = ceiling(headers)?;
    let unavailable = || ServerError::new("Publication admission snapshot is unavailable", None);
    if !current.initialized() {
        return Err(unavailable());
    }
    let Some(ceiling) = ceiling else {
        return Ok(current);
    };
    let checkpoint = ceiling.min(current.high_watermark().checkpoint());
    // An indexed snapshot already below the admission is safe without rewriting it.
    if checkpoint == current.high_watermark().checkpoint() {
        return Ok(current);
    }
    let Some((summary, _, _)) = loader
        .load_one_checkpoint(checkpoint)
        .await
        .map_err(|_| unavailable())?
    else {
        return Err(unavailable());
    };
    if summary.sequence_number != checkpoint {
        return Err(unavailable());
    }
    current
        .admitted(
            checkpoint,
            summary.epoch,
            summary.network_total_transactions,
            summary.timestamp_ms,
        )
        .map(Arc::new)
        .map_err(|_| unavailable())
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderMap;
    use axum::http::HeaderValue;

    use super::HEADER;
    use super::ceiling;

    #[tokio::test]
    async fn uninitialized_requests_fail_with_or_without_admission_header() {
        use std::sync::Arc;

        use prometheus::Registry;
        use rtd_indexer_alt_reader::kv_loader::KvLoader;
        use rtd_indexer_alt_reader::ledger_grpc_reader::LedgerGrpcArgs;
        use rtd_indexer_alt_reader::ledger_grpc_reader::LedgerGrpcReader;

        use crate::task::watermark::Watermarks;

        let reader = LedgerGrpcReader::new(
            "http://127.0.0.1:9".parse().unwrap(),
            LedgerGrpcArgs::default(),
            None,
            &Registry::new(),
            10,
            10,
        )
        .await
        .unwrap();
        let loader = KvLoader::new(reader);
        for bounded in [false, true] {
            let mut headers = HeaderMap::new();
            if bounded {
                headers.insert(HEADER, HeaderValue::from_static("7"));
            }
            assert!(
                super::watermarks(&headers, Arc::new(Watermarks::default()), &loader)
                    .await
                    .is_err()
            );
        }
    }

    #[tokio::test]
    async fn absent_or_canonical_height() {
        let mut headers = HeaderMap::new();
        assert_eq!(ceiling(&headers).unwrap(), None);
        for (text, value) in [("0", 0), ("10", 10), ("18446744073709551615", u64::MAX)] {
            headers.insert(HEADER, HeaderValue::from_static(text));
            assert_eq!(ceiling(&headers).unwrap(), Some(value));
        }
    }

    #[tokio::test]
    async fn invalid_and_duplicate_height() {
        for text in ["", "01", "-1", "+1", "1.0", " 1", "18446744073709551616"] {
            let mut headers = HeaderMap::new();
            headers.insert(HEADER, HeaderValue::from_static(text));
            assert!(ceiling(&headers).is_err(), "{text}");
        }
        let mut headers = HeaderMap::new();
        headers.append(HEADER, HeaderValue::from_static("10"));
        headers.append(HEADER, HeaderValue::from_static("10"));
        assert!(ceiling(&headers).is_err());
    }
}
