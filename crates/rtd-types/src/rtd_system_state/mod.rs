// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use self::rtd_system_state_inner_v1::{RtdSystemStateInnerV1, ValidatorV1};
use self::rtd_system_state_summary::{RtdSystemStateSummary, RtdValidatorSummary};
use crate::base_types::ObjectID;
use crate::committee::CommitteeWithNetworkMetadata;
use crate::dynamic_field::{
    Field, get_dynamic_field_from_store, get_dynamic_field_object_from_store,
};
use crate::error::{RtdError, RtdErrorKind};
use crate::gas::GasCostSummary;
use crate::object::{MoveObject, Object};
use crate::storage::ObjectStore;
use crate::rtd_system_state::epoch_start_rtd_system_state::EpochStartSystemState;
use crate::rtd_system_state::rtd_system_state_inner_v2::RtdSystemStateInnerV2;
use crate::versioned::Versioned;
use crate::{MoveTypeTagTrait, RTD_SYSTEM_ADDRESS, RTD_SYSTEM_STATE_OBJECT_ID, id::UID};
use anyhow::Result;
use enum_dispatch::enum_dispatch;
use move_core_types::{ident_str, identifier::IdentStr, language_storage::StructTag};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt;
use rtd_protocol_config::{ProtocolConfig, ProtocolVersion};

pub mod epoch_start_rtd_system_state;
pub mod mock;
pub mod rtd_system_state_inner_v1;
pub mod rtd_system_state_inner_v2;
pub mod rtd_system_state_summary;

#[cfg(msim)]
mod simtest_rtd_system_state_inner;
#[cfg(msim)]
use self::simtest_rtd_system_state_inner::{
    SimTestRtdSystemStateInnerDeepV2, SimTestRtdSystemStateInnerShallowV2,
    SimTestRtdSystemStateInnerV1, SimTestValidatorDeepV2, SimTestValidatorV1,
};

const RTD_SYSTEM_STATE_WRAPPER_STRUCT_NAME: &IdentStr = ident_str!("RtdSystemState");

pub const RTD_SYSTEM_MODULE_NAME: &IdentStr = ident_str!("rtd_system");
pub const ADVANCE_EPOCH_FUNCTION_NAME: &IdentStr = ident_str!("advance_epoch");
pub const ADVANCE_EPOCH_SAFE_MODE_FUNCTION_NAME: &IdentStr = ident_str!("advance_epoch_safe_mode");

#[cfg(msim)]
pub const RTD_SYSTEM_STATE_SIM_TEST_V1: u64 = 18446744073709551605; // u64::MAX - 10
#[cfg(msim)]
pub const RTD_SYSTEM_STATE_SIM_TEST_SHALLOW_V2: u64 = 18446744073709551606; // u64::MAX - 9
#[cfg(msim)]
pub const RTD_SYSTEM_STATE_SIM_TEST_DEEP_V2: u64 = 18446744073709551607; // u64::MAX - 8

/// Rust version of the Move rtd::rtd_system::RtdSystemState type
/// This repreents the object with 0x5 ID.
/// In Rust, this type should be rarely used since it's just a thin
/// wrapper used to access the inner object.
/// Within this module, we use it to determine the current version of the system state inner object type,
/// so that we could deserialize the inner object correctly.
/// Outside of this module, we only use it in genesis snapshot and testing.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RtdSystemStateWrapper {
    pub id: UID,
    pub version: u64,
}

impl RtdSystemStateWrapper {
    pub fn type_() -> StructTag {
        StructTag {
            address: RTD_SYSTEM_ADDRESS,
            name: RTD_SYSTEM_STATE_WRAPPER_STRUCT_NAME.to_owned(),
            module: RTD_SYSTEM_MODULE_NAME.to_owned(),
            type_params: vec![],
        }
    }

    /// Advances epoch in safe mode natively in Rust, without involking Move.
    /// This ensures that there cannot be any failure from Move and is guaranteed to succeed.
    /// Returns the old and new inner system state object.
    pub fn advance_epoch_safe_mode(
        &self,
        params: &AdvanceEpochParams,
        object_store: &dyn ObjectStore,
        protocol_config: &ProtocolConfig,
    ) -> (Object, Object) {
        let id = self.id.id.bytes;
        let old_field_object = get_dynamic_field_object_from_store(object_store, id, &self.version)
            .expect("Dynamic field object of wrapper should always be present in the object store");
        let mut new_field_object = old_field_object.clone();
        let move_object = new_field_object
            .data
            .try_as_move_mut()
            .expect("Dynamic field object must be a Move object");
        match self.version {
            1 => {
                Self::advance_epoch_safe_mode_impl::<RtdSystemStateInnerV1>(
                    move_object,
                    params,
                    protocol_config,
                );
            }
            2 => {
                Self::advance_epoch_safe_mode_impl::<RtdSystemStateInnerV2>(
                    move_object,
                    params,
                    protocol_config,
                );
            }
            #[cfg(msim)]
            RTD_SYSTEM_STATE_SIM_TEST_V1 => {
                Self::advance_epoch_safe_mode_impl::<SimTestRtdSystemStateInnerV1>(
                    move_object,
                    params,
                    protocol_config,
                );
            }
            #[cfg(msim)]
            RTD_SYSTEM_STATE_SIM_TEST_SHALLOW_V2 => {
                Self::advance_epoch_safe_mode_impl::<SimTestRtdSystemStateInnerShallowV2>(
                    move_object,
                    params,
                    protocol_config,
                );
            }
            #[cfg(msim)]
            RTD_SYSTEM_STATE_SIM_TEST_DEEP_V2 => {
                Self::advance_epoch_safe_mode_impl::<SimTestRtdSystemStateInnerDeepV2>(
                    move_object,
                    params,
                    protocol_config,
                );
            }
            _ => unreachable!(),
        }
        (old_field_object, new_field_object)
    }

    fn advance_epoch_safe_mode_impl<T>(
        move_object: &mut MoveObject,
        params: &AdvanceEpochParams,
        protocol_config: &ProtocolConfig,
    ) where
        T: Serialize + DeserializeOwned + RtdSystemStateTrait,
    {
        let mut field: Field<u64, T> =
            bcs::from_bytes(move_object.contents()).expect("bcs deserialization should never fail");
        tracing::info!(
            "Advance epoch safe mode: current epoch: {}, protocol_version: {}, system_state_version: {}",
            field.value.epoch(),
            field.value.protocol_version(),
            field.value.system_state_version()
        );
        field.value.advance_epoch_safe_mode(params);
        tracing::info!(
            "Safe mode activated. New epoch: {}, protocol_version: {}, system_state_version: {}",
            field.value.epoch(),
            field.value.protocol_version(),
            field.value.system_state_version()
        );
        let new_contents = bcs::to_bytes(&field).expect("bcs serialization should never fail");
        move_object
            .update_contents_advance_epoch_safe_mode(new_contents, protocol_config)
            .expect("Update rtd system object content cannot fail since it should be small or unbounded");
    }
}

/// This is the standard API that all inner system state object type should implement.
#[enum_dispatch]
pub trait RtdSystemStateTrait {
    fn epoch(&self) -> u64;
    fn reference_gas_price(&self) -> u64;
    fn protocol_version(&self) -> u64;
    fn system_state_version(&self) -> u64;
    fn epoch_start_timestamp_ms(&self) -> u64;
    fn epoch_duration_ms(&self) -> u64;
    fn safe_mode(&self) -> bool;
    fn safe_mode_gas_cost_summary(&self) -> GasCostSummary;
    fn advance_epoch_safe_mode(&mut self, params: &AdvanceEpochParams);
    fn get_current_epoch_committee(&self) -> CommitteeWithNetworkMetadata;
    fn get_pending_active_validators<S: ObjectStore + ?Sized>(
        &self,
        object_store: &S,
    ) -> Result<Vec<RtdValidatorSummary>, RtdError>;
    fn into_epoch_start_state(self) -> EpochStartSystemState;
    fn into_rtd_system_state_summary(self) -> RtdSystemStateSummary;
}

/// RtdSystemState provides an abstraction over multiple versions of the inner RtdSystemStateInner object.
/// This should be the primary interface to the system state object in Rust.
/// We use enum dispatch to dispatch all methods defined in RtdSystemStateTrait to the actual
/// implementation in the inner types.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[enum_dispatch(RtdSystemStateTrait)]
pub enum RtdSystemState {
    V1(RtdSystemStateInnerV1),
    V2(RtdSystemStateInnerV2),
    #[cfg(msim)]
    SimTestV1(SimTestRtdSystemStateInnerV1),
    #[cfg(msim)]
    SimTestShallowV2(SimTestRtdSystemStateInnerShallowV2),
    #[cfg(msim)]
    SimTestDeepV2(SimTestRtdSystemStateInnerDeepV2),
}

/// This is the fixed type used by genesis.
pub type RtdSystemStateInnerGenesis = RtdSystemStateInnerV1;
pub type RtdValidatorGenesis = ValidatorV1;

impl RtdSystemState {
    /// Always return the version that we will be using for genesis.
    /// Genesis always uses this version regardless of the current version.
    /// Note that since it's possible for the actual genesis of the network to diverge from the
    /// genesis of the latest Rust code, it's important that we only use this for tooling purposes.
    pub fn into_genesis_version_for_tooling(self) -> RtdSystemStateInnerGenesis {
        match self {
            RtdSystemState::V1(inner) => inner,
            _ => unreachable!(),
        }
    }

    pub fn version(&self) -> u64 {
        self.system_state_version()
    }
}

pub fn get_rtd_system_state_wrapper(
    object_store: &dyn ObjectStore,
) -> Result<RtdSystemStateWrapper, RtdError> {
    let wrapper = object_store
        .get_object(&RTD_SYSTEM_STATE_OBJECT_ID)
        // Don't panic here on None because object_store is a generic store.
        .ok_or_else(|| {
            RtdErrorKind::RtdSystemStateReadError(
                "RtdSystemStateWrapper object not found".to_owned(),
            )
        })?;
    let move_object = wrapper.data.try_as_move().ok_or_else(|| {
        RtdErrorKind::RtdSystemStateReadError(
            "RtdSystemStateWrapper object must be a Move object".to_owned(),
        )
    })?;
    let result = bcs::from_bytes::<RtdSystemStateWrapper>(move_object.contents())
        .map_err(|err| RtdErrorKind::RtdSystemStateReadError(err.to_string()))?;
    Ok(result)
}

pub fn get_rtd_system_state(object_store: &dyn ObjectStore) -> Result<RtdSystemState, RtdError> {
    let wrapper = get_rtd_system_state_wrapper(object_store)?;
    let id = wrapper.id.id.bytes;
    match wrapper.version {
        1 => {
            let result: RtdSystemStateInnerV1 =
                get_dynamic_field_from_store(object_store, id, &wrapper.version).map_err(
                    |err| {
                        RtdErrorKind::DynamicFieldReadError(format!(
                            "Failed to load rtd system state inner object with ID {:?} and version {:?}: {:?}",
                            id, wrapper.version, err
                        ))
                    },
                )?;
            Ok(RtdSystemState::V1(result))
        }
        2 => {
            let result: RtdSystemStateInnerV2 =
                get_dynamic_field_from_store(object_store, id, &wrapper.version).map_err(
                    |err| {
                        RtdErrorKind::DynamicFieldReadError(format!(
                            "Failed to load rtd system state inner object with ID {:?} and version {:?}: {:?}",
                            id, wrapper.version, err
                        ))
                    },
                )?;
            Ok(RtdSystemState::V2(result))
        }
        #[cfg(msim)]
        RTD_SYSTEM_STATE_SIM_TEST_V1 => {
            let result: SimTestRtdSystemStateInnerV1 =
                get_dynamic_field_from_store(object_store, id, &wrapper.version).map_err(
                    |err| {
                        RtdErrorKind::DynamicFieldReadError(format!(
                            "Failed to load rtd system state inner object with ID {:?} and version {:?}: {:?}",
                            id, wrapper.version, err
                        ))
                    },
                )?;
            Ok(RtdSystemState::SimTestV1(result))
        }
        #[cfg(msim)]
        RTD_SYSTEM_STATE_SIM_TEST_SHALLOW_V2 => {
            let result: SimTestRtdSystemStateInnerShallowV2 =
                get_dynamic_field_from_store(object_store, id, &wrapper.version).map_err(
                    |err| {
                        RtdErrorKind::DynamicFieldReadError(format!(
                            "Failed to load rtd system state inner object with ID {:?} and version {:?}: {:?}",
                            id, wrapper.version, err
                        ))
                    },
                )?;
            Ok(RtdSystemState::SimTestShallowV2(result))
        }
        #[cfg(msim)]
        RTD_SYSTEM_STATE_SIM_TEST_DEEP_V2 => {
            let result: SimTestRtdSystemStateInnerDeepV2 =
                get_dynamic_field_from_store(object_store, id, &wrapper.version).map_err(
                    |err| {
                        RtdErrorKind::DynamicFieldReadError(format!(
                            "Failed to load rtd system state inner object with ID {:?} and version {:?}: {:?}",
                            id, wrapper.version, err
                        ))
                    },
                )?;
            Ok(RtdSystemState::SimTestDeepV2(result))
        }
        _ => Err(RtdErrorKind::RtdSystemStateReadError(format!(
            "Unsupported RtdSystemState version: {}",
            wrapper.version
        ))
        .into()),
    }
}

/// Given a system state type version, and the ID of the table, along with a key, retrieve the
/// dynamic field as a Validator type. We need the version to determine which inner type to use for
/// the Validator type. This is assuming that the validator is stored in the table as
/// ValidatorWrapper type.
pub fn get_validator_from_table<K>(
    object_store: &dyn ObjectStore,
    table_id: ObjectID,
    key: &K,
) -> Result<RtdValidatorSummary, RtdError>
where
    K: Clone + MoveTypeTagTrait + Serialize + DeserializeOwned + fmt::Debug,
{
    let field: ValidatorWrapper = get_dynamic_field_from_store(object_store, table_id, key)
        .map_err(|err| {
            RtdErrorKind::RtdSystemStateReadError(format!(
                "Failed to load validator wrapper from table: {:?}",
                err
            ))
        })?;
    let versioned = field.inner;
    let version = versioned.version;
    match version {
        1 => {
            let validator: ValidatorV1 =
                get_dynamic_field_from_store(object_store, versioned.id.id.bytes, &version)
                    .map_err(|err| {
                        RtdErrorKind::RtdSystemStateReadError(format!(
                            "Failed to load inner validator from the wrapper: {:?}",
                            err
                        ))
                    })?;
            Ok(validator.into_rtd_validator_summary())
        }
        #[cfg(msim)]
        RTD_SYSTEM_STATE_SIM_TEST_V1 => {
            let validator: SimTestValidatorV1 =
                get_dynamic_field_from_store(object_store, versioned.id.id.bytes, &version)
                    .map_err(|err| {
                        RtdErrorKind::RtdSystemStateReadError(format!(
                            "Failed to load inner validator from the wrapper: {:?}",
                            err
                        ))
                    })?;
            Ok(validator.into_rtd_validator_summary())
        }
        #[cfg(msim)]
        RTD_SYSTEM_STATE_SIM_TEST_DEEP_V2 => {
            let validator: SimTestValidatorDeepV2 =
                get_dynamic_field_from_store(object_store, versioned.id.id.bytes, &version)
                    .map_err(|err| {
                        RtdErrorKind::RtdSystemStateReadError(format!(
                            "Failed to load inner validator from the wrapper: {:?}",
                            err
                        ))
                    })?;
            Ok(validator.into_rtd_validator_summary())
        }
        _ => Err(RtdErrorKind::RtdSystemStateReadError(format!(
            "Unsupported Validator version: {}",
            version
        ))
        .into()),
    }
}

pub fn get_validators_from_table_vec<S, ValidatorType>(
    object_store: &S,
    table_id: ObjectID,
    table_size: u64,
) -> Result<Vec<ValidatorType>, RtdError>
where
    S: ObjectStore + ?Sized,
    ValidatorType: Serialize + DeserializeOwned,
{
    let mut validators = vec![];
    for i in 0..table_size {
        let validator: ValidatorType = get_dynamic_field_from_store(&object_store, table_id, &i)
            .map_err(|err| {
                RtdErrorKind::RtdSystemStateReadError(format!(
                    "Failed to load validator from table: {:?}",
                    err
                ))
            })?;
        validators.push(validator);
    }
    Ok(validators)
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Default)]
pub struct PoolTokenExchangeRate {
    rtd_amount: u64,
    pool_token_amount: u64,
}

impl PoolTokenExchangeRate {
    /// Rate of the staking pool, pool token amount : Rtd amount
    pub fn rate(&self) -> f64 {
        if self.rtd_amount == 0 {
            1_f64
        } else {
            self.pool_token_amount as f64 / self.rtd_amount as f64
        }
    }

    pub fn new(rtd_amount: u64, pool_token_amount: u64) -> Self {
        Self {
            rtd_amount,
            pool_token_amount,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct ValidatorWrapper {
    pub inner: Versioned,
}

#[derive(Debug)]
pub struct AdvanceEpochParams {
    pub epoch: u64,
    pub next_protocol_version: ProtocolVersion,
    pub storage_charge: u64,
    pub computation_charge: u64,
    pub storage_rebate: u64,
    pub non_refundable_storage_fee: u64,
    pub storage_fund_reinvest_rate: u64,
    pub reward_slashing_rate: u64,
    pub epoch_start_timestamp_ms: u64,
}

#[cfg(msim)]
pub mod advance_epoch_result_injection {
    use crate::{
        committee::EpochId,
        error::{ExecutionError, ExecutionErrorKind},
        execution::ResultWithTimings,
    };
    use std::cell::RefCell;

    thread_local! {
        /// Override the result of advance_epoch in the range [start, end).
        static OVERRIDE: RefCell<Option<(EpochId, EpochId)>>  = RefCell::new(None);
    }

    /// Override the result of advance_epoch transaction if new epoch is in the provided range [start, end).
    pub fn set_override(value: Option<(EpochId, EpochId)>) {
        OVERRIDE.with(|o| *o.borrow_mut() = value);
    }

    /// This function is used to modify the result of advance_epoch transaction for testing.
    /// If the override is set, the result will be an execution error, otherwise the original result will be returned.
    pub fn maybe_modify_result(
        result: ResultWithTimings<(), ExecutionError>,
        current_epoch: EpochId,
    ) -> ResultWithTimings<(), ExecutionError> {
        if let Some((start, end)) = OVERRIDE.with(|o| *o.borrow()) {
            if current_epoch >= start && current_epoch < end {
                return Err((
                    ExecutionError::new(ExecutionErrorKind::FunctionNotFound, None),
                    vec![],
                ));
            }
        }
        result
    }

    // For old execution versions that don't report timings
    pub fn maybe_modify_result_legacy(
        result: Result<(), ExecutionError>,
        current_epoch: EpochId,
    ) -> Result<(), ExecutionError> {
        if let Some((start, end)) = OVERRIDE.with(|o| *o.borrow()) {
            if current_epoch >= start && current_epoch < end {
                return Err(ExecutionError::new(
                    ExecutionErrorKind::FunctionNotFound,
                    None,
                ));
            }
        }
        result
    }
}
