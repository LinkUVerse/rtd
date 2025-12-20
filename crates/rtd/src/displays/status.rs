// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::displays::Pretty;
use std::fmt::{Display, Formatter};
use rtd_json_rpc_types::RtdExecutionStatus::{self, Failure, Success};

impl Display for Pretty<'_, RtdExecutionStatus> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Pretty(status) = self;

        let output = match status {
            Success => "success".to_string(),
            Failure { error } => format!("failed due to {error}"),
        };

        write!(f, "{}", output)
    }
}
