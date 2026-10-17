// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
#![allow(clippy::all)]
#![allow(deprecated)]
#![allow(unused)]

#[path = "generated"]
pub mod bigtable {
    #[rustfmt::skip]
    #[path = "google.bigtable.v2.rs"]
    pub mod v2;
}

#[rustfmt::skip]
#[path = "generated/google.rpc.rs"]
pub mod rpc;

#[rustfmt::skip]
#[path = "generated/google.api.rs"]
pub mod api;

#[rustfmt::skip]
#[path = "generated/google.r#type.rs"]
pub mod r#type;
