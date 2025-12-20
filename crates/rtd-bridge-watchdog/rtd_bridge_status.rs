// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! The RtdBridgeStatus observable monitors whether the Rtd Bridge is paused.

use crate::Observable;
use async_trait::async_trait;
use prometheus::IntGauge;
use std::sync::Arc;
use rtd_bridge::rtd_client::RtdBridgeClient;

use tokio::time::Duration;
use tracing::{error, info};

pub struct RtdBridgeStatus {
    rtd_client: Arc<RtdBridgeClient>,
    metric: IntGauge,
}

impl RtdBridgeStatus {
    pub fn new(rtd_client: Arc<RtdBridgeClient>, metric: IntGauge) -> Self {
        Self { rtd_client, metric }
    }
}

#[async_trait]
impl Observable for RtdBridgeStatus {
    fn name(&self) -> &str {
        "RtdBridgeStatus"
    }

    async fn observe_and_report(&self) {
        let status = self.rtd_client.is_bridge_paused().await;
        match status {
            Ok(status) => {
                self.metric.set(status as i64);
                info!("Rtd Bridge Status: {:?}", status);
            }
            Err(e) => {
                error!("Error getting rtd bridge status: {:?}", e);
            }
        }
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(2)
    }
}
