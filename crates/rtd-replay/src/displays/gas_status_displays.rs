// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::displays::Pretty;
use std::fmt::{Display, Formatter};
use rtd_types::gas::RtdGasStatus;
use rtd_types::gas_model::gas_v2::RtdGasStatus as GasStatusV2;
use tabled::{
    builder::Builder as TableBuilder,
    settings::{Style as TableStyle, style::HorizontalLine},
};

impl Display for Pretty<'_, RtdGasStatus> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Pretty(rtd_gas_status) = self;
        match rtd_gas_status {
            RtdGasStatus::V2(s) => {
                display_info(f, s)?;
                per_object_storage_table(f, s)?;
            }
        };
        Ok(())
    }
}

fn per_object_storage_table(f: &mut Formatter, rtd_gas_status: &GasStatusV2) -> std::fmt::Result {
    let mut builder = TableBuilder::default();
    builder.push_record(vec!["Object ID", "Bytes", "Old Rebate", "New Rebate"]);
    for (object_id, per_obj_storage) in rtd_gas_status.per_object_storage() {
        builder.push_record(vec![
            object_id.to_string(),
            per_obj_storage.new_size.to_string(),
            per_obj_storage.storage_rebate.to_string(),
            per_obj_storage.storage_cost.to_string(),
        ]);
    }
    let mut table = builder.build();

    table.with(TableStyle::rounded().horizontals([HorizontalLine::new(
        1,
        TableStyle::modern().get_horizontal(),
    )]));
    write!(f, "\n{}\n", table)
}

fn display_info(f: &mut Formatter<'_>, rtd_gas_status: &GasStatusV2) -> std::fmt::Result {
    let mut builder = TableBuilder::default();
    builder.push_record(vec!["Gas Info".to_string()]);
    builder.push_record(vec![format!(
        "Reference Gas Price: {}",
        rtd_gas_status.reference_gas_price()
    )]);
    builder.push_record(vec![format!(
        "Gas Price: {}",
        rtd_gas_status.gas_status.gas_price()
    )]);

    builder.push_record(vec![format!(
        "Max Gas Stack Height: {}",
        rtd_gas_status.gas_status.stack_height_high_water_mark()
    )]);

    builder.push_record(vec![format!(
        "Max Gas Stack Size: {}",
        rtd_gas_status.gas_status.stack_size_high_water_mark()
    )]);

    builder.push_record(vec![format!(
        "Number of Bytecode Instructions Executed: {}",
        rtd_gas_status.gas_status.instructions_executed()
    )]);

    let mut table = builder.build();
    table.with(TableStyle::rounded());

    write!(f, "\n{}\n", table)
}
