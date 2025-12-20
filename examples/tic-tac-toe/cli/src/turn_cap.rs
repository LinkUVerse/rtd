// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;
use rtd_types::base_types::ObjectID;

/// Rust representation of a Move `owned::TurnCap`, rtdtable for deserializing from their BCS
/// representation.
#[allow(dead_code)]
#[derive(Deserialize)]
pub(crate) struct TurnCap {
    pub id: ObjectID,
    pub game: ObjectID,
}
