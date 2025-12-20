// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use move_core_types::ident_str;
use move_core_types::identifier::IdentStr;
use linku_common::{fatal, in_test_configuration};

use crate::TypeTag;
use crate::accumulator_root::AccumulatorObjId;
use crate::balance::Balance;
use crate::base_types::RtdAddress;
use crate::effects::{
    AccumulatorAddress, AccumulatorOperation, AccumulatorValue, AccumulatorWriteV1,
};
use crate::error::{RtdError, RtdErrorKind};
use crate::gas_coin::GasCoin;

pub const ACCUMULATOR_MODULE_NAME: &IdentStr = ident_str!("accumulator");

#[derive(Debug, Clone)]
pub struct AccumulatorEvent {
    pub accumulator_obj: AccumulatorObjId,
    pub write: AccumulatorWriteV1,
}

impl AccumulatorEvent {
    pub fn new(accumulator_obj: AccumulatorObjId, write: AccumulatorWriteV1) -> Self {
        if in_test_configuration()
            && let Ok(expected_obj) = crate::accumulator_root::AccumulatorValue::get_field_id(
                write.address.address,
                &write.address.ty,
            )
        {
            debug_assert_eq!(
                *accumulator_obj.inner(),
                *expected_obj.inner(),
                "Accumulator object ID {:?} does not match expected ID {:?} for address {:?} and type {:?}",
                accumulator_obj.inner(),
                expected_obj.inner(),
                write.address.address,
                write.address.ty
            );
        }
        Self {
            accumulator_obj,
            write,
        }
    }

    pub fn from_balance_change(
        address: RtdAddress,
        balance_type: TypeTag,
        net_change: i64,
    ) -> Result<Self, RtdError> {
        if !Balance::is_balance_type(&balance_type) {
            return Err(RtdErrorKind::TypeError {
                error: "only Balance<T> is supported".to_string(),
            }
            .into());
        }
        let accumulator_obj =
            crate::accumulator_root::AccumulatorValue::get_field_id(address, &balance_type)?;

        let accumulator_address = AccumulatorAddress::new(address, balance_type);

        let (operation, amount) = if net_change > 0 {
            (AccumulatorOperation::Split, net_change as u64)
        } else {
            (AccumulatorOperation::Merge, (-net_change) as u64)
        };

        let accumulator_write = AccumulatorWriteV1 {
            address: accumulator_address,
            operation,
            value: AccumulatorValue::Integer(amount),
        };

        Ok(Self::new(accumulator_obj, accumulator_write))
    }

    pub fn total_rtd_in_event(&self) -> (u64 /* input */, u64 /* output */) {
        let Self {
            write:
                AccumulatorWriteV1 {
                    address: AccumulatorAddress { ty, .. },
                    operation,
                    value,
                },
            ..
        } = self;

        let rtd = match ty {
            TypeTag::Struct(struct_tag) => {
                if !GasCoin::is_gas_balance(struct_tag) {
                    0
                } else {
                    match value {
                        AccumulatorValue::Integer(v) => *v,
                        AccumulatorValue::IntegerTuple(_, _) => fatal!("invalid accumulator value"),
                        AccumulatorValue::EventDigest(_) => fatal!("invalid accumulator value"),
                    }
                }
            }
            _ => 0,
        };

        match operation {
            AccumulatorOperation::Merge => (0, rtd),
            AccumulatorOperation::Split => (rtd, 0),
        }
    }
}
