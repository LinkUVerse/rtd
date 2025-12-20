// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::cluster::new_wallet_context_from_cluster;

use super::Cluster;
use shared_crypto::intent::Intent;
use rtd_keys::keystore::AccountKeystore;
use rtd_sdk::wallet_context::WalletContext;
use rtd_sdk::{RtdClient, RtdClientBuilder};
use rtd_types::base_types::RtdAddress;
use rtd_types::crypto::{KeypairTraits, Signature};
use rtd_types::transaction::TransactionData;
use tracing::{Instrument, info, info_span};

pub struct WalletClient {
    wallet_context: WalletContext,
    address: RtdAddress,
    fullnode_client: RtdClient,
}

#[allow(clippy::borrowed_box)]
impl WalletClient {
    pub async fn new_from_cluster(cluster: &(dyn Cluster + Sync + Send)) -> Self {
        let key = cluster.user_key();
        let address: RtdAddress = key.public().into();
        let wallet_context = new_wallet_context_from_cluster(cluster, key)
            .await
            .instrument(info_span!("init_wallet_context_for_test_user"));

        let rpc_url = String::from(cluster.fullnode_url());
        info!("Use fullnode rpc: {}", &rpc_url);
        let fullnode_client = RtdClientBuilder::default().build(rpc_url).await.unwrap();

        Self {
            wallet_context: wallet_context.into_inner(),
            address,
            fullnode_client,
        }
    }

    pub fn get_wallet(&self) -> &WalletContext {
        &self.wallet_context
    }

    pub fn get_wallet_mut(&mut self) -> &mut WalletContext {
        &mut self.wallet_context
    }

    pub fn get_wallet_address(&self) -> RtdAddress {
        self.address
    }

    pub fn get_fullnode_client(&self) -> &RtdClient {
        &self.fullnode_client
    }

    pub async fn sign(&self, txn_data: &TransactionData, desc: &str) -> Signature {
        self.get_wallet()
            .config
            .keystore
            .sign_secure(&self.address, txn_data, Intent::rtd_transaction())
            .await
            .unwrap_or_else(|e| panic!("Failed to sign transaction for {}. {}", desc, e))
    }
}
