// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use rtd_types::storage::{ObjectFundsResolver, RuntimeObjectResolver, Storage};

/// Interface with the store necessary to execute a programmable transaction
pub trait ExecutionState: Storage + RuntimeObjectResolver + ObjectFundsResolver {}

impl<T> ExecutionState for T where T: Storage + RuntimeObjectResolver + ObjectFundsResolver {}
