// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use rtd_enum_compat_util::*;

use crate::{RtdMoveStruct, RtdMoveValue};

#[test]
fn enforce_order_test() {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.extend(["tests", "staged", "rtd_move_struct.yaml"]);
    check_enum_compat_order::<RtdMoveStruct>(path);

    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.extend(["tests", "staged", "rtd_move_value.yaml"]);
    check_enum_compat_order::<RtdMoveValue>(path);
}
