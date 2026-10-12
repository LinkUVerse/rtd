// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use diesel::Insertable;
use diesel::Queryable;
use diesel::QueryableByName;
use diesel::Selectable;
use rtd_field_count::FieldCount;
use serde::Serialize;

use crate::schema::hist_display_checkpoints;
use crate::schema::hist_display_coverage;
use crate::schema::hist_displays;

#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    Insertable,
    Queryable,
    QueryableByName,
    Selectable,
    FieldCount,
    Serialize,
)]
#[diesel(table_name = hist_displays)]
pub struct StoredHistoricalDisplay {
    pub object_type: Vec<u8>,
    pub cp_sequence_number: i64,
    pub checkpoint_tx_index: i64,
    pub event_index: i64,
    pub transaction_digest: Vec<u8>,
    pub display_id: Vec<u8>,
    pub display_version: i32,
    pub display: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Insertable, Queryable, Selectable)]
#[diesel(table_name = hist_display_checkpoints)]
pub struct StoredDisplayCheckpoint {
    pub cp_sequence_number: i64,
    pub checkpoint_digest: Vec<u8>,
    pub previous_digest: Option<Vec<u8>>,
    pub event_count: i64,
    pub events_sha256: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Insertable, Queryable, QueryableByName, Selectable)]
#[diesel(table_name = hist_display_coverage)]
pub struct StoredDisplayCoverage {
    pub id: i16,
    pub genesis_digest: Vec<u8>,
    pub source_id: String,
    pub generation: String,
    pub reader_lo: i64,
    pub checkpoint_hi: i64,
    pub checkpoint_digest: Option<Vec<u8>>,
    pub writer_epoch: i64,
}
