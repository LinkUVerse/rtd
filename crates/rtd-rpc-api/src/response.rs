// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use axum::{
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
};

use crate::RpcService;
use rtd_rpc::headers::{
    X_RTD_CHAIN, X_RTD_CHAIN_ID, X_RTD_CHECKPOINT_HEIGHT, X_RTD_EPOCH,
    X_RTD_LOWEST_AVAILABLE_CHECKPOINT, X_RTD_LOWEST_AVAILABLE_CHECKPOINT_OBJECTS, X_RTD_TIMESTAMP,
    X_RTD_TIMESTAMP_MS,
};

pub async fn append_info_headers(
    State(state): State<RpcService>,
    response: Response,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();

    if let Ok(chain_id) = state.chain_id().to_string().try_into() {
        headers.insert(X_RTD_CHAIN_ID, chain_id);
    }

    if let Ok(chain) = state.chain_id().chain().as_str().try_into() {
        headers.insert(X_RTD_CHAIN, chain);
    }

    if let Ok(latest_checkpoint) = state.reader.inner().get_latest_checkpoint() {
        headers.insert(X_RTD_EPOCH, latest_checkpoint.epoch().into());
        headers.insert(
            X_RTD_CHECKPOINT_HEIGHT,
            latest_checkpoint.sequence_number.into(),
        );
        headers.insert(X_RTD_TIMESTAMP_MS, latest_checkpoint.timestamp_ms.into());

        headers.insert(
            X_RTD_TIMESTAMP,
            crate::proto::timestamp_ms_to_proto(latest_checkpoint.timestamp_ms)
                .to_string()
                .try_into()
                .expect("timestamp is a valid HeaderValue"),
        );
    }

    if let Ok(lowest_available_checkpoint) = state.reader.inner().get_lowest_available_checkpoint()
    {
        headers.insert(
            X_RTD_LOWEST_AVAILABLE_CHECKPOINT,
            lowest_available_checkpoint.into(),
        );
    }

    if let Ok(lowest_available_checkpoint_objects) = state
        .reader
        .inner()
        .get_lowest_available_checkpoint_objects()
    {
        headers.insert(
            X_RTD_LOWEST_AVAILABLE_CHECKPOINT_OBJECTS,
            lowest_available_checkpoint_objects.into(),
        );
    }

    if let Some(server_version) = state
        .server_version()
        .and_then(|version| version.to_string().try_into().ok())
    {
        headers.insert(axum::http::header::SERVER, server_version);
    }

    (headers, response)
}
