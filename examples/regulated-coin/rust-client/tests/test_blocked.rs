// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use anyhow::{anyhow, Result};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{StructTag, TypeTag};
use rust_client::tx_run::{execute_command, AppCommand, AppConfig};
use rtd_config::{rtd_config_dir, RTD_CLIENT_CONFIG, RTD_KEYSTORE_FILENAME};
use rtd_keys::keystore::{AccountKeystore, FileBasedKeystore};
use rtd_sdk::rpc_types::{ObjectChange, RtdTransactionBlockResponse};
use rtd_sdk::types::base_types::{ObjectID, ObjectRef, RtdAddress};
use rtd_sdk::wallet_context::WalletContext;

// Change from here
const PACKAGE_ID: &'static str =
    "0x5da522e939ce9fdcb15d4b3d03a16aa408706105cf90114cedc9613809f04c20";
const MODULE: &'static str = "regulated_coin";
// To here

fn cmd_rtd_client_switch(new_addr: RtdAddress) -> Result<()> {
    println!("SWITCHING TO ADDRESS: {new_addr}");
    let rtd_client_switch = format!("rtd client switch --address {new_addr}");
    let _ = std::process::Command::new("sh")
        .arg("-c")
        .arg(rtd_client_switch)
        .output()?;
    Ok(())
}

fn get_other_address(different_from: RtdAddress) -> Result<RtdAddress> {
    let keystore =
        FileBasedKeystore::load_or_create(&rtd_config_dir()?.join(RTD_KEYSTORE_FILENAME))?;
    Ok(keystore
        .keys()
        .into_iter()
        .find(|pub_key| RtdAddress::from(pub_key) != different_from)
        .map(|pub_key| RtdAddress::from(&pub_key))
        .ok_or(anyhow!("No other address found"))?)
}

async fn get_config() -> Result<AppConfig> {
    let package_id = ObjectID::from_hex_literal(PACKAGE_ID)?;
    let otw = MODULE.to_uppercase();
    let type_tag = TypeTag::Struct(Box::new(StructTag {
        address: AccountAddress::new(package_id.as_ref().try_into()?),
        module: Identifier::from_str(MODULE)?,
        name: Identifier::from_str(&otw)?,
        type_params: vec![],
    }));
    let wallet_context =
        WalletContext::new(&rtd_config_dir()?.join(RTD_CLIENT_CONFIG), None, None).await?;

    Ok(AppConfig {
        client: wallet_context.get_client().await?,
        wallet_context,
        type_tag,
    })
}

#[tokio::test]
async fn test_is_blocked() -> Result<()> {
    let mut config = get_config().await?;
    let admin_addr = config.wallet_context.active_address()?;
    let deny_addr = get_other_address(admin_addr)?;

    let command = AppCommand::DenyListAdd(deny_addr);
    println!("CURRENT_ADDRESS: {admin_addr}");
    let _ = execute_command(command, config).await?;

    let command = AppCommand::MintAndTransfer(10000, deny_addr);
    let resp_mint = execute_command(command, get_config().await?).await?;
    let coin: ObjectRef = resp_mint
        .object_changes
        .unwrap()
        .into_iter()
        .find(|obj_chng| match obj_chng {
            ObjectChange::Created { .. } => true,
            _ => false,
        })
        .map(|created| created.object_ref())
        .ok_or(anyhow!("No coin created"))?;

    cmd_rtd_client_switch(deny_addr)?;
    // Wrap in function to ensure client will switch to initial
    async fn run_as_deny_addr(
        coin_id: ObjectID,
        transfer_to: RtdAddress,
    ) -> Result<RtdTransactionBlockResponse> {
        let config = get_config().await?;
        let command = AppCommand::Transfer(coin_id, transfer_to);
        execute_command(command, config).await
    }
    let resp2 = run_as_deny_addr(coin.0, admin_addr).await; // Notice we do not use '?' so that
                                                            // cmd_rtd_client_switch runs again
    cmd_rtd_client_switch(admin_addr)?;
    assert!(resp2.is_err());
    assert!(get_config().await?.wallet_context.active_address()? == admin_addr);

    Ok(())
}
