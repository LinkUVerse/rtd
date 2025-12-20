// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use rtd_sdk::RtdClientBuilder;

// This example shows the few basic ways to connect to a Rtd network.
// There are several in-built methods for connecting to the
// Rtd devnet, tesnet, and localnet (running locally),
// as well as a custom way for connecting to custom URLs.
// The example prints out the API versions of the different networks,
// and finally, it prints the list of available RPC methods
// and the list of subscriptions.
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

    // Rtd devnet -- https://fullnode.devnet.rtd.io:443
    let rtd_devnet = RtdClientBuilder::default().build_devnet().await?;
    println!("Rtd devnet version: {}", rtd_devnet.api_version());

    // Rtd testnet -- https://fullnode.testnet.rtd.io:443
    let rtd_testnet = RtdClientBuilder::default().build_testnet().await?;
    println!("Rtd testnet version: {}", rtd_testnet.api_version());

    // Rtd mainnet -- https://fullnode.mainnet.rtd.io:443
    let rtd_mainnet = RtdClientBuilder::default().build_mainnet().await?;
    println!("Rtd mainnet version: {}", rtd_mainnet.api_version());

    println!("rpc methods: {:?}", rtd_testnet.available_rpc_methods());
    println!(
        "available subscriptions: {:?}",
        rtd_testnet.available_subscriptions()
    );

    Ok(())
}
