// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use base64::Engine as _;
use http::HeaderValue;
use http::Request;
use http::Response;
use prost::Message as _;
use tonic::body::Body;
use tonic::codegen::Service;

use crate::bigtable::proto::bigtable::v2::FeatureFlags;

/// Feature metadata needed by the RTD Bigtable-wire protocol compatibility gateway.
pub(crate) fn bigtable_features_header(batch_write_flow_control: bool) -> HeaderValue {
    let feature_flags = FeatureFlags {
        reverse_scans: true,
        mutate_rows_rate_limit: batch_write_flow_control,
        mutate_rows_rate_limit2: batch_write_flow_control,
        ..Default::default()
    };
    let encoded = base64::engine::general_purpose::URL_SAFE.encode(feature_flags.encode_to_vec());
    HeaderValue::from_str(&encoded).expect("base64 is always a valid header value")
}

#[derive(Clone)]
pub(crate) struct MetadataChannel<S> {
    inner: S,
    features_header: HeaderValue,
}

impl<S> MetadataChannel<S> {
    pub(crate) fn new(inner: S, features_header: HeaderValue) -> Self {
        Self {
            inner,
            features_header,
        }
    }
}

impl<S> Service<Request<Body>> for MetadataChannel<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    S::Future: Send,
{
    type Response = Response<Body>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        request
            .headers_mut()
            .insert("bigtable-features", self.features_header.clone());
        let cloned = self.inner.clone();
        let mut ready_inner = std::mem::replace(&mut self.inner, cloned);
        Box::pin(async move { ready_inner.call(request).await.map_err(Into::into) })
    }
}

#[cfg(test)]
mod tests {
    use base64::Engine as _;
    use prost::Message as _;

    use super::*;

    #[test]
    fn default_features_header_is_byte_compatible() {
        assert_eq!(
            bigtable_features_header(false),
            HeaderValue::from_static("CAE="),
        );
    }

    #[test]
    fn flow_control_features_header_round_trips() {
        let header = bigtable_features_header(true);
        let encoded = base64::engine::general_purpose::URL_SAFE
            .decode(header.as_bytes())
            .expect("features header should be valid websafe base64");
        let feature_flags =
            FeatureFlags::decode(encoded.as_slice()).expect("features header should be valid");

        assert_eq!(
            feature_flags,
            FeatureFlags {
                reverse_scans: true,
                mutate_rows_rate_limit: true,
                mutate_rows_rate_limit2: true,
                ..Default::default()
            }
        );
    }
}
