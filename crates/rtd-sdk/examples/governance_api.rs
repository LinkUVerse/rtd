// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

mod utils;
use utils::setup_for_read;

// This example connects to the Rtd testnet
// and collects information about the stakes in the network,
// the committee information,
// lists all the validators' name, description, and rtd address,
// and prints the reference gas price.

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let (rtd, active_address) = setup_for_read().await?;

    // ************ GOVERNANCE API ************ //

    // Stakes
    let stakes = rtd.governance_api().get_stakes(active_address).await?;

    println!(" *** Stakes ***");
    println!("{:?}", stakes);
    println!(" *** Stakes ***\n");

    // Committee Info
    let committee = rtd.governance_api().get_committee_info(None).await?; // None defaults to the latest epoch

    println!(" *** Committee Info ***");
    println!("{:?}", committee);
    println!(" *** Committee Info ***\n");

    // Latest Rtd System State
    let rtd_system_state = rtd.governance_api().get_latest_rtd_system_state().await?;

    println!(" *** Rtd System State ***");
    println!("{:?}", rtd_system_state);
    println!(" *** Rtd System State ***\n");

    // List all active validators

    println!(" *** List active validators *** ");
    rtd_system_state
        .active_validators
        .into_iter()
        .for_each(|validator| {
            println!(
                "Name: {}, Description: {}, RtdAddress: {:?}",
                validator.name, validator.description, validator.rtd_address
            )
        });

    println!(" *** List active validators ***\n");
    // Reference Gas Price
    let reference_gas_price = rtd.governance_api().get_reference_gas_price().await?;

    println!(" *** Reference Gas Price ***");
    println!("{:?}", reference_gas_price);
    println!(" *** Reference Gas Price ***\n");

    // ************ END OF GOVERNANCE API ************ //
    Ok(())
}
