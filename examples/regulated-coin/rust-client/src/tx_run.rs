// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

mod coin;
mod deny;

use anyhow::Result;
use move_core_types::language_storage::TypeTag;
use rtd_keys::keystore::AccountKeystore;
use rtd_sdk::RtdClient;
use rtd_sdk::rpc_types::RtdTransactionBlockResponse;
use rtd_sdk::types::base_types::{RtdAddress, ObjectID};
use rtd_sdk::wallet_context::WalletContext;

#[derive(Debug)]
pub enum AppCommand {
    DenyListAdd(RtdAddress),
    DenyListRemove(RtdAddress),
    MintAndTransfer(u64, RtdAddress),
    Transfer(ObjectID, RtdAddress),
    Burn(ObjectID)
}

pub struct AppConfig {
    pub client: RtdClient,
    pub wallet_context: WalletContext,
    pub type_tag: TypeTag,
}

pub async fn execute_command(
    command: AppCommand,
    config: AppConfig,
) -> Result<RtdTransactionBlockResponse> {
    let AppConfig {
        client,
        mut wallet_context,
        type_tag,
    } = config;
    let active_addr = wallet_context.active_address()?;
    let signer = wallet_context.config.keystore.get_key(&active_addr)?;

    match command {
        AppCommand::DenyListAdd(address) => {
            let deny_list = deny::get_deny_list(&client).await?;
            let deny_cap = deny::get_deny_cap(&client, active_addr, type_tag.clone()).await?;
            deny::deny_list_add(&client, signer, type_tag, deny_list, deny_cap, address).await
        }
        AppCommand::DenyListRemove(address) => {
            let deny_list = deny::get_deny_list(&client).await?;
            let deny_cap = deny::get_deny_cap(&client, active_addr, type_tag.clone()).await?;
            deny::deny_list_remove(&client, signer, type_tag, deny_list, deny_cap, address).await
        }
        AppCommand::MintAndTransfer(balance, to_address) => {
            let treasury_cap =
                coin::get_treasury_cap(&client, active_addr, type_tag.clone()).await?;
            coin::mint_and_transfer(&client, signer, type_tag, treasury_cap, to_address, balance)
                .await
        }
        AppCommand::Transfer(coin_id, to_address) => {
            let coin = coin::get_coin(&client, coin_id).await?;
            coin::transfer(&client, signer, coin, to_address).await
        }
        AppCommand::Burn(coin_id) => {
            let treasury_cap =
                coin::get_treasury_cap(&client, active_addr, type_tag.clone()).await?;
            let coin = coin::get_coin(&client, coin_id).await?;
            coin::burn(&client, signer, type_tag, treasury_cap, coin).await
        }
    }
}
