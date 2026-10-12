// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Optional same-chain freshness floor, independent of historical query selectors.
//! Resetting genesis requires restarting the server and clearing caller floors.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use http::HeaderMap;
use tokio::sync::watch;
use tokio::time::Instant;
use tonic::Status;
use tower::{Layer, Service};

pub const X_RTD_MIN_CHECKPOINT: &str = "x-rtd-min-checkpoint";
// Match the existing dependency cache budget: an old success must not admit a read.
const VIEW_MAX_AGE: Duration = Duration::from_secs(1);
pub(crate) type ServiceView = Option<(Instant, u64)>;

#[derive(Clone)]
pub(crate) struct MonotonicReadLayer {
    view: watch::Receiver<ServiceView>,
    timeout: Duration,
}

impl MonotonicReadLayer {
    pub(crate) fn new(view: watch::Receiver<ServiceView>, timeout: Duration) -> Self {
        Self { view, timeout }
    }
}

impl<S> Layer<S> for MonotonicReadLayer {
    type Service = MonotonicRead<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MonotonicRead {
            inner,
            view: self.view.clone(),
            timeout: self.timeout,
        }
    }
}

#[derive(Clone)]
pub(crate) struct MonotonicRead<S> {
    inner: S,
    view: watch::Receiver<ServiceView>,
    timeout: Duration,
}

impl<S, ReqBody, ResBody> Service<http::Request<ReqBody>> for MonotonicRead<S>
where
    S: Service<http::Request<ReqBody>, Response = http::Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Default,
{
    type Response = http::Response<ResBody>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: http::Request<ReqBody>) -> Self::Future {
        // Keep the service instance that poll_ready actually readied.
        let clone = self.inner.clone();
        let mut ready_inner = std::mem::replace(&mut self.inner, clone);
        let mut view = self.view.clone();
        let timeout = self.timeout;
        Box::pin(async move {
            match await_checkpoint(&mut view, timeout, request.headers()).await {
                Ok(()) => ready_inner.call(request).await,
                Err(status) => Ok(status.into_http()),
            }
        })
    }
}

fn checkpoint_header(headers: &HeaderMap, name: &'static str) -> Result<Option<u64>, Status> {
    let Some(value) = headers.get(name) else {
        return Ok(None);
    };
    value
        .to_str()
        .map_err(|_| Status::invalid_argument(format!("{name}: invalid ASCII checkpoint")))?
        .parse::<u64>()
        .map(Some)
        .map_err(|_| Status::invalid_argument(format!("{name}: invalid checkpoint")))
}

async fn await_checkpoint(
    view: &mut watch::Receiver<ServiceView>,
    timeout: Duration,
    headers: &HeaderMap,
) -> Result<(), Status> {
    let Some(floor) = checkpoint_header(headers, X_RTD_MIN_CHECKPOINT)? else {
        return Ok(());
    };
    // Public admission supplies a ceiling. A client floor may never bypass it.
    let ceiling = checkpoint_header(headers, "x-rtd-visible-checkpoint")?;
    if ceiling.is_some_and(|height| height < floor) {
        return Err(Status::unavailable(
            "minimum checkpoint exceeds publication admission; retry",
        ));
    }
    let reached = tokio::time::timeout(
        timeout,
        view.wait_for(|sample| {
            sample.is_some_and(|(sampled_at, height)| {
                sampled_at.elapsed() < VIEW_MAX_AGE && height >= floor
            })
        }),
    )
    .await
    .is_ok_and(|result| result.is_ok());
    if reached {
        Ok(())
    } else {
        Err(Status::unavailable(format!(
            "monotonic read at checkpoint {floor} not available on this replica; retry"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const WAIT: Duration = Duration::from_millis(100);

    fn headers(value: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(X_RTD_MIN_CHECKPOINT, value.parse().unwrap());
        headers
    }

    #[tokio::test(start_paused = true)]
    async fn no_header_bypasses_missing_view_and_zero_is_a_valid_floor() {
        let (tx, mut rx) = watch::channel(None);
        await_checkpoint(&mut rx, WAIT, &HeaderMap::new())
            .await
            .unwrap();
        assert_eq!(
            await_checkpoint(&mut rx, WAIT, &headers("0"))
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Unavailable
        );
        tx.send_replace(Some((Instant::now(), 0)));
        await_checkpoint(&mut rx, WAIT, &headers("0"))
            .await
            .unwrap();
    }

    #[tokio::test(start_paused = true)]
    async fn waits_for_progress_and_accepts_newer_view() {
        let (tx, mut rx) = watch::channel(Some((Instant::now(), 98)));
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(40)).await;
            tx.send_replace(Some((Instant::now(), 101)));
        });
        await_checkpoint(&mut rx, WAIT, &headers("100"))
            .await
            .unwrap();
    }

    #[tokio::test(start_paused = true)]
    async fn missing_future_expired_and_closed_views_cannot_admit() {
        for sample in [
            None,
            Some((Instant::now(), 98)),
            Some((Instant::now() - Duration::from_secs(2), 200)),
        ] {
            let (_tx, mut rx) = watch::channel(sample);
            assert_eq!(
                await_checkpoint(&mut rx, WAIT, &headers("100"))
                    .await
                    .unwrap_err()
                    .code(),
                tonic::Code::Unavailable
            );
        }
        let (tx, mut rx) = watch::channel(None);
        drop(tx);
        assert_eq!(
            await_checkpoint(&mut rx, WAIT, &headers("100"))
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Unavailable
        );
    }

    #[tokio::test(start_paused = true)]
    async fn failed_refresh_and_reset_to_zero_do_not_reuse_old_height() {
        let (tx, mut rx) = watch::channel(Some((Instant::now(), 200)));
        tx.send_replace(None);
        assert_eq!(
            await_checkpoint(&mut rx, WAIT, &headers("200"))
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Unavailable
        );
        tx.send_replace(Some((Instant::now(), 0)));
        assert_eq!(
            await_checkpoint(&mut rx, WAIT, &headers("200"))
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Unavailable
        );
        await_checkpoint(&mut rx, WAIT, &headers("0"))
            .await
            .unwrap();
    }

    #[tokio::test(start_paused = true)]
    async fn publication_ceiling_cannot_be_overridden_by_minimum() {
        let (_tx, mut rx) = watch::channel(Some((Instant::now(), 200)));
        let mut request = headers("100");
        request.insert("x-rtd-visible-checkpoint", "99".parse().unwrap());
        assert_eq!(
            await_checkpoint(&mut rx, WAIT, &request)
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Unavailable
        );
        request.insert("x-rtd-visible-checkpoint", "100".parse().unwrap());
        await_checkpoint(&mut rx, WAIT, &request).await.unwrap();
    }

    #[test]
    fn invalid_values_are_invalid_argument_and_u64_max_parses() {
        for value in ["", "invalid", "-1", "18446744073709551616", "1.5", " 1"] {
            assert_eq!(
                checkpoint_header(&headers(value), X_RTD_MIN_CHECKPOINT)
                    .unwrap_err()
                    .code(),
                tonic::Code::InvalidArgument
            );
        }
        assert_eq!(
            checkpoint_header(&headers("18446744073709551615"), X_RTD_MIN_CHECKPOINT).unwrap(),
            Some(u64::MAX)
        );
    }

    #[tokio::test(start_paused = true)]
    async fn rejected_request_never_calls_handler() {
        use std::convert::Infallible;
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        use tower::ServiceExt;
        let calls = Arc::new(AtomicUsize::new(0));
        let count = calls.clone();
        let inner = tower::service_fn(move |_: http::Request<()>| {
            count.fetch_add(1, Ordering::SeqCst);
            async { Ok::<_, Infallible>(http::Response::new(tonic::body::Body::empty())) }
        });
        let (_tx, rx) = watch::channel(Some((Instant::now(), 0)));
        let layer = MonotonicReadLayer::new(rx, WAIT);
        let mut request = http::Request::new(());
        *request.headers_mut() = headers("1");
        let response = layer.layer(inner.clone()).oneshot(request).await.unwrap();
        assert_eq!(response.headers().get("grpc-status").unwrap(), "14");
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        layer
            .layer(inner)
            .oneshot(http::Request::new(()))
            .await
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
