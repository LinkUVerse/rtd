// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use rtd_kvstore::BigTableClient;
use rtd_kvstore::tables::epochs::col;
use rtd_protocol_config::{Chain, ProtocolConfig};
use rtd_rpc::field::{FieldMask, FieldMaskTree, FieldMaskUtil};
use rtd_rpc::merge::Merge;
use rtd_rpc::proto::rtd::rpc::v2::Epoch;
use rtd_rpc::proto::rtd::rpc::v2::ProtocolConfig as RpcProtocolConfig;
use rtd_rpc::proto::rtd::rpc::v2::{GetEpochRequest, GetEpochResponse};
use rtd_rpc_api::{
    ErrorReason,
    grpc::v2::protocol_config_to_proto,
    proto::{google::rpc::bad_request::FieldViolation, timestamp_ms_to_proto},
};
use rtd_types::rtd_system_state::RtdSystemStateTrait;

pub const READ_MASK_DEFAULT: &str = rtd_rpc_api::read_mask_defaults::EPOCH;

pub async fn get_epoch(
    mut client: BigTableClient,
    request: GetEpochRequest,
    chain: Chain,
    admission: Option<(u64, u64)>,
) -> rtd_rpc_api::Result<GetEpochResponse> {
    let read_mask = {
        let read_mask = request
            .read_mask
            .unwrap_or_else(|| FieldMask::from_str(READ_MASK_DEFAULT));
        read_mask.validate::<Epoch>().map_err(|path| {
            FieldViolation::new("read_mask")
                .with_description(format!("invalid read_mask path: {path}"))
                .with_reason(ErrorReason::FieldInvalid)
        })?;
        FieldMaskTree::from(read_mask)
    };

    let mut message = Epoch::default();

    let mut columns = epoch_columns(&read_mask);
    if admission.is_some() {
        for column in [col::START_CHECKPOINT, col::END_CHECKPOINT] {
            if !columns.contains(&column) {
                columns.push(column);
            }
        }
    }
    let requested_epoch = match admission {
        Some((_, epoch)) => {
            if request.epoch.is_some_and(|requested| requested > epoch) {
                return Err(rtd_rpc_api::RpcError::new(
                    tonic::Code::FailedPrecondition,
                    "Epoch exceeds publication admission boundary",
                ));
            }
            Some(request.epoch.unwrap_or(epoch))
        }
        None => request.epoch,
    };
    let maybe_epoch_info = if let Some(epoch) = requested_epoch {
        client
            .get_epochs_filtered(&[epoch], Some(&columns))
            .await?
            .pop()
    } else {
        client.get_latest_epoch_filtered(Some(&columns)).await?
    };

    let Some(mut epoch_info) = maybe_epoch_info else {
        if admission.is_some() {
            return Err(rtd_rpc_api::RpcError::new(
                tonic::Code::Unavailable,
                "Admitted epoch unavailable",
            ));
        }
        return Ok(GetEpochResponse::default());
    };
    if let Some((height, current_epoch)) = admission {
        admit_epoch(
            &mut epoch_info,
            height,
            current_epoch,
            requested_epoch.unwrap(),
        )?;
    }

    if read_mask.contains(Epoch::EPOCH_FIELD.name) {
        message.epoch = epoch_info.epoch;
    }
    if read_mask.contains(Epoch::FIRST_CHECKPOINT_FIELD.name) {
        message.first_checkpoint = epoch_info.start_checkpoint;
    }
    if read_mask.contains(Epoch::LAST_CHECKPOINT_FIELD.name) {
        message.last_checkpoint = epoch_info.end_checkpoint;
    }
    if read_mask.contains(Epoch::START_FIELD.name) {
        message.start = epoch_info.start_timestamp_ms.map(timestamp_ms_to_proto);
    }
    if read_mask.contains(Epoch::END_FIELD.name) {
        message.end = epoch_info.end_timestamp_ms.map(timestamp_ms_to_proto);
    }
    if read_mask.contains(Epoch::REFERENCE_GAS_PRICE_FIELD.name) {
        message.reference_gas_price = epoch_info.reference_gas_price;
    }
    if let (Some(submask), Some(version)) = (
        read_mask.subtree(Epoch::PROTOCOL_CONFIG_FIELD.name),
        epoch_info.protocol_version,
    ) {
        let protocol_config = ProtocolConfig::get_for_version_if_supported(version.into(), chain);
        message.protocol_config = protocol_config.map(|config| {
            RpcProtocolConfig::merge_from(protocol_config_to_proto(config), &submask)
        });
    }
    if read_mask.contains(Epoch::COMMITTEE_FIELD.name) {
        message.committee = epoch_info.system_state.as_ref().map(|system_state| {
            system_state
                .get_current_epoch_committee()
                .committee()
                .clone()
                .into()
        });
    }
    if read_mask.contains(Epoch::SYSTEM_STATE_FIELD.name) {
        message.system_state = epoch_info
            .system_state
            .map(|system_state| Box::new(system_state.into()));
    }
    Ok(GetEpochResponse::new(message))
}

/// Compute the set of BigTable columns needed for the given read mask.
/// Always includes `ep` (epoch number, small).
fn epoch_columns(mask: &FieldMaskTree) -> Vec<&'static str> {
    let mut columns = vec![col::EPOCH];

    if mask.contains(Epoch::FIRST_CHECKPOINT_FIELD.name) {
        columns.push(col::START_CHECKPOINT);
    }
    if mask.contains(Epoch::LAST_CHECKPOINT_FIELD.name) {
        columns.push(col::END_CHECKPOINT);
    }
    if mask.contains(Epoch::START_FIELD.name) {
        columns.push(col::START_TIMESTAMP);
    }
    if mask.contains(Epoch::END_FIELD.name) {
        columns.push(col::END_TIMESTAMP);
    }
    if mask.contains(Epoch::REFERENCE_GAS_PRICE_FIELD.name) {
        columns.push(col::REFERENCE_GAS_PRICE);
    }
    if mask.subtree(Epoch::PROTOCOL_CONFIG_FIELD.name).is_some() {
        columns.push(col::PROTOCOL_VERSION);
    }
    if mask.contains(Epoch::COMMITTEE_FIELD.name) || mask.contains(Epoch::SYSTEM_STATE_FIELD.name) {
        columns.push(col::SYSTEM_STATE);
    }

    columns
}

fn admit_epoch(
    info: &mut rtd_kvstore::EpochData,
    height: u64,
    current_epoch: u64,
    requested_epoch: u64,
) -> rtd_rpc_api::Result<()> {
    let unavailable = || {
        rtd_rpc_api::RpcError::new(
            tonic::Code::Unavailable,
            "Admitted epoch metadata unavailable",
        )
    };
    let start = info.start_checkpoint.ok_or_else(unavailable)?;
    if info.epoch != Some(requested_epoch)
        || start > height
        || info.end_checkpoint.is_some_and(|end| end < start)
        || (requested_epoch < current_epoch && info.end_checkpoint.is_none_or(|end| end > height))
    {
        return Err(unavailable());
    }
    if info.end_checkpoint.is_none_or(|end| end > height) {
        info.end_checkpoint = None;
        info.end_timestamp_ms = None;
    }
    Ok(())
}

#[cfg(test)]
mod admission_tests {
    use super::*;
    use bytes::Bytes;
    use rtd_kvstore::tables::epochs;

    fn epoch() -> rtd_kvstore::EpochData {
        let cells: Vec<_> = [("ep", 2u64), ("sc", 5), ("ec", 10), ("et", 1000)]
            .into_iter()
            .map(|(key, value)| {
                (
                    Bytes::from_static(key.as_bytes()),
                    Bytes::copy_from_slice(&value.to_be_bytes()),
                )
            })
            .collect();
        epochs::decode(&cells).unwrap()
    }

    #[tokio::test]
    async fn epoch_end_is_hidden_until_its_checkpoint_is_admitted() {
        let mut value = epoch();
        admit_epoch(&mut value, 9, 2, 2).unwrap();
        assert_eq!(value.end_checkpoint, None);
        assert_eq!(value.end_timestamp_ms, None);
        let mut value = epoch();
        admit_epoch(&mut value, 10, 2, 2).unwrap();
        assert_eq!(value.end_checkpoint, Some(10));
        assert_eq!(value.end_timestamp_ms, Some(1000));
        admit_epoch(&mut value, 11, 3, 2).unwrap();
    }

    #[tokio::test]
    async fn missing_or_inconsistent_epoch_provenance_is_rejected() {
        assert!(admit_epoch(&mut epoch(), 4, 2, 2).is_err());
        assert!(admit_epoch(&mut epoch(), 9, 3, 2).is_err());
        assert!(admit_epoch(&mut epoch(), 10, 2, 1).is_err());
        let mut value = epoch();
        value.start_checkpoint = None;
        assert!(admit_epoch(&mut value, 10, 2, 2).is_err());
        let mut value = epoch();
        value.end_checkpoint = Some(4);
        assert!(admit_epoch(&mut value, 10, 2, 2).is_err());
        let mut value = epoch();
        value.end_checkpoint = None;
        admit_epoch(&mut value, 9, 2, 2).unwrap();
        assert_eq!(value.end_timestamp_ms, None);
    }
}
