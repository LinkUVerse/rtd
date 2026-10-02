// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Progress reader interface for RTD self-hosted analytics integrations.

use anyhow::Result;

#[async_trait::async_trait]
pub trait MaxCheckpointReader: Send + Sync + 'static {
    async fn max_checkpoint(&self) -> Result<i64>;
}
