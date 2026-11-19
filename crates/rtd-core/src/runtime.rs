// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use rtd_config::NodeConfig;
use tokio::runtime::Runtime;

pub struct RtdRuntimes {
    // Order in this struct is the order in which runtimes are stopped
    pub rtd_node: Runtime,
    pub metrics: Runtime,
}

impl RtdRuntimes {
    pub fn new(_confg: &NodeConfig) -> Self {
        let rtd_node = tokio::runtime::Builder::new_multi_thread()
            .thread_name("rtd-node-runtime")
            .enable_all()
            .build()
            .unwrap();
        let metrics = tokio::runtime::Builder::new_multi_thread()
            .thread_name("metrics-runtime")
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();

        Self { rtd_node, metrics }
    }
}
