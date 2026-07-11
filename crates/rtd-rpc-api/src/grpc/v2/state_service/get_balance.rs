// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::{ErrorReason, Result, RpcError, RpcService};
use rtd_rpc::proto::google::rpc::bad_request::FieldViolation;
use rtd_rpc::proto::rtd::rpc::v2::{Balance, GetBalanceRequest, GetBalanceResponse};
use rtd_sdk_types::Address;
use rtd_sdk_types::StructTag;
use rtd_types::base_types::RtdAddress;
use rtd_types::rtd_sdk_types_conversions::struct_tag_sdk_to_core;

#[tracing::instrument(skip(service))]
pub fn get_balance(service: &RpcService, request: GetBalanceRequest) -> Result<GetBalanceResponse> {
    let indexes = service
        .reader
        .inner()
        .indexes()
        .ok_or_else(RpcError::not_found)?;

    let owner = parse_owner(request.owner.as_deref())?;

    let coin_type = request
        .coin_type
        .as_ref()
        .ok_or_else(|| {
            FieldViolation::new("coin_type")
                .with_description("missing coin_type")
                .with_reason(ErrorReason::FieldMissing)
        })?
        .parse::<StructTag>()
        .map_err(|e| {
            FieldViolation::new("coin_type")
                .with_description(format!("invalid coin_type: {e}"))
                .with_reason(ErrorReason::FieldInvalid)
        })?;

    let core_coin_type = struct_tag_sdk_to_core(coin_type.clone())?;

    let balance_info = indexes
        .get_balance(&owner, &core_coin_type)?
        .unwrap_or_default(); // Use default (zero) if no balance found

    let mut balance = Balance::default();
    balance.coin_type = Some(coin_type.to_string());
    balance.balance = Some(balance_info.balance);

    let mut response = GetBalanceResponse::default();
    response.balance = Some(balance);
    Ok(response)
}

fn parse_owner(owner: Option<&str>) -> Result<RtdAddress> {
    Ok(owner
        .ok_or_else(|| {
            FieldViolation::new("owner")
                .with_description("missing owner")
                .with_reason(ErrorReason::FieldMissing)
        })?
        .parse::<Address>()
        .map_err(|e| {
            FieldViolation::new("owner")
                .with_description(format!("invalid owner: {e}"))
                .with_reason(ErrorReason::FieldInvalid)
        })?
        .into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_shortened_owner_address() {
        let owner = parse_owner(Some("0x2")).unwrap();

        assert_eq!(owner.to_string(), format!("0x{:0>64}", "2"));
    }
}
