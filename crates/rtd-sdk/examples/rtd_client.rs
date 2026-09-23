// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use rtd_sdk::RtdClientBuilder;

// This example shows the few basic ways to connect to a Rtd network.
// The example connects to localnet and optionally to RTD_RPC_URL.
// Public networks require an explicitly configured endpoint.
// Note that running this code will fail if there is no Rtd network
// running locally on the default address: 127.0.0.1:9000

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let rtd = RtdClientBuilder::default()
        .build("http://127.0.0.1:9000") // local network address
        .await?;
    println!("Rtd local network version: {}", rtd.api_version());

    // local Rtd network, like the above one but using the dedicated function
    let rtd_local = RtdClientBuilder::default().build_localnet().await?;
    println!("Rtd local network version: {}", rtd_local.api_version());

    if let Ok(url) = std::env::var("RTD_RPC_URL") {
        let custom = RtdClientBuilder::default().build(&url).await?;
        println!("Configured RTD network version: {}", custom.api_version());
    }

    println!("rpc methods: {:?}", rtd_local.available_rpc_methods());
    println!(
        "available subscriptions: {:?}",
        rtd_local.available_subscriptions()
    );

    Ok(())
}
