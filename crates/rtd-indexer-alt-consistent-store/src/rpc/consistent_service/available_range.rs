// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use rtd_indexer_alt_consistent_api::proto::rpc::consistent::v1alpha as grpc;

use crate::rpc::consistent_service::State;
use crate::rpc::error::RpcError;
use crate::rpc::error::db_error;

const PIPELINES: [&str; 4] = [
    "address_balances",
    "balances",
    "object_by_owner",
    "object_by_type",
];

pub(super) fn available_range(
    state: &State,
    checkpoint: u64,
    grpc::AvailableRangeRequest {}: grpc::AvailableRangeRequest,
) -> Result<grpc::AvailableRangeResponse, RpcError> {
    let range = state.store.db().snapshot_range(checkpoint);
    let chain_id = state
        .store
        .db()
        .common_chain_id(&PIPELINES)
        .map_err(|error| db_error(error, "Failed to read consistent pipeline chain IDs"))?;
    Ok(grpc::AvailableRangeResponse {
        min_checkpoint: range.as_ref().map(|r| r.start().checkpoint_hi_inclusive),
        max_checkpoint: range.as_ref().map(|r| r.end().checkpoint_hi_inclusive),
        max_epoch: range.as_ref().map(|r| r.end().epoch_hi_inclusive),
        total_transactions: range.as_ref().map(|r| r.end().tx_hi),
        max_timestamp_ms: range.as_ref().map(|r| r.end().timestamp_ms_hi_inclusive),
        stride: Some(state.consistency_config.stride),
        chain_id: chain_id.map(|id| id.to_vec().into()),
    })
}
