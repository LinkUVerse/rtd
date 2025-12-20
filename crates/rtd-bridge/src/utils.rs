// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::abi::{
    EthBridgeCommittee, EthBridgeConfig, EthBridgeLimiter, EthBridgeVault, EthRtdBridge,
};
use crate::config::{
    BridgeNodeConfig, EthConfig, MetricsConfig, RtdConfig, WatchdogConfig, default_ed25519_key_pair,
};
use crate::crypto::BridgeAuthorityKeyPair;
use crate::crypto::BridgeAuthorityPublicKeyBytes;
use crate::server::APPLICATION_JSON;
use crate::types::BridgeCommittee;
use crate::types::{AddTokensOnRtdAction, BridgeAction};
use anyhow::anyhow;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::signers::Wallet;
use ethers::types::Address as EthAddress;
use fastcrypto::ed25519::Ed25519KeyPair;
use fastcrypto::encoding::{Encoding, Hex};
use fastcrypto::secp256k1::Secp256k1KeyPair;
use fastcrypto::traits::EncodeDecodeBase64;
use fastcrypto::traits::KeyPair;
use futures::future::join_all;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use rtd_config::Config;
use rtd_json_rpc_types::RtdExecutionStatus;
use rtd_json_rpc_types::RtdTransactionBlockEffectsAPI;
use rtd_json_rpc_types::RtdTransactionBlockResponseOptions;
use rtd_keys::keypair_file::read_key;
use rtd_sdk::wallet_context::WalletContext;
use rtd_test_transaction_builder::TestTransactionBuilder;
use rtd_types::BRIDGE_PACKAGE_ID;
use rtd_types::base_types::RtdAddress;
use rtd_types::bridge::BridgeChainId;
use rtd_types::bridge::{BRIDGE_MODULE_NAME, BRIDGE_REGISTER_FOREIGN_TOKEN_FUNCTION_NAME};
use rtd_types::committee::StakeUnit;
use rtd_types::crypto::RtdKeyPair;
use rtd_types::crypto::ToFromBytes;
use rtd_types::crypto::get_key_pair;
use rtd_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use rtd_types::rtd_system_state::rtd_system_state_summary::RtdSystemStateSummary;
use rtd_types::transaction::{ObjectArg, TransactionData};

pub type EthSigner = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;

pub struct EthBridgeContracts<P> {
    pub bridge: EthRtdBridge<Provider<P>>,
    pub committee: EthBridgeCommittee<Provider<P>>,
    pub limiter: EthBridgeLimiter<Provider<P>>,
    pub vault: EthBridgeVault<Provider<P>>,
    pub config: EthBridgeConfig<Provider<P>>,
}

/// Generate Bridge Authority key (Secp256k1KeyPair) and write to a file as base64 encoded `privkey`.
pub fn generate_bridge_authority_key_and_write_to_file(
    path: &PathBuf,
) -> Result<(), anyhow::Error> {
    let (_, kp): (_, BridgeAuthorityKeyPair) = get_key_pair();
    let eth_address = BridgeAuthorityPublicKeyBytes::from(&kp.public).to_eth_address();
    println!(
        "Corresponding Ethereum address by this ecdsa key: {:?}",
        eth_address
    );
    let rtd_address = RtdAddress::from(&kp.public);
    println!(
        "Corresponding Rtd address by this ecdsa key: {:?}",
        rtd_address
    );
    let base64_encoded = kp.encode_base64();
    std::fs::write(path, base64_encoded)
        .map_err(|err| anyhow!("Failed to write encoded key to path: {:?}", err))
}

/// Generate Bridge Client key (Secp256k1KeyPair or Ed25519KeyPair) and write to a file as base64 encoded `flag || privkey`.
pub fn generate_bridge_client_key_and_write_to_file(
    path: &PathBuf,
    use_ecdsa: bool,
) -> Result<(), anyhow::Error> {
    let kp = if use_ecdsa {
        let (_, kp): (_, Secp256k1KeyPair) = get_key_pair();
        let eth_address = BridgeAuthorityPublicKeyBytes::from(&kp.public).to_eth_address();
        println!(
            "Corresponding Ethereum address by this ecdsa key: {:?}",
            eth_address
        );
        RtdKeyPair::from(kp)
    } else {
        let (_, kp): (_, Ed25519KeyPair) = get_key_pair();
        RtdKeyPair::from(kp)
    };
    let rtd_address = RtdAddress::from(&kp.public());
    println!("Corresponding Rtd address by this key: {:?}", rtd_address);

    let contents = kp.encode_base64();
    std::fs::write(path, contents)
        .map_err(|err| anyhow!("Failed to write encoded key to path: {:?}", err))
}

/// Given the address of RtdBridge Proxy, return the addresses of the committee, limiter, vault, and config.
pub async fn get_eth_contract_addresses<P: ethers::providers::JsonRpcClient + 'static>(
    bridge_proxy_address: EthAddress,
    provider: &Arc<Provider<P>>,
) -> anyhow::Result<(
    EthAddress,
    EthAddress,
    EthAddress,
    EthAddress,
    EthAddress,
    EthAddress,
    EthAddress,
    EthAddress,
)> {
    let rtd_bridge = EthRtdBridge::new(bridge_proxy_address, provider.clone());
    let committee_address: EthAddress = rtd_bridge.committee().call().await?;
    let committee = EthBridgeCommittee::new(committee_address, provider.clone());
    let config_address: EthAddress = committee.config().call().await?;
    let bridge_config = EthBridgeConfig::new(config_address, provider.clone());
    let limiter_address: EthAddress = rtd_bridge.limiter().call().await?;
    let vault_address: EthAddress = rtd_bridge.vault().call().await?;
    let vault = EthBridgeVault::new(vault_address, provider.clone());
    let weth_address: EthAddress = vault.w_eth().call().await?;
    let usdt_address: EthAddress = bridge_config.token_address_of(4).call().await?;
    let wbtc_address: EthAddress = bridge_config.token_address_of(1).call().await?;
    let lbtc_address: EthAddress = bridge_config.token_address_of(6).call().await?;

    Ok((
        committee_address,
        limiter_address,
        vault_address,
        config_address,
        weth_address,
        usdt_address,
        wbtc_address,
        lbtc_address,
    ))
}

/// Given the address of RtdBridge Proxy, return the contracts of the committee, limiter, vault, and config.
pub async fn get_eth_contracts<P: ethers::providers::JsonRpcClient + 'static>(
    bridge_proxy_address: EthAddress,
    provider: &Arc<Provider<P>>,
) -> anyhow::Result<EthBridgeContracts<P>> {
    let rtd_bridge = EthRtdBridge::new(bridge_proxy_address, provider.clone());
    let committee_address: EthAddress = rtd_bridge.committee().call().await?;
    let limiter_address: EthAddress = rtd_bridge.limiter().call().await?;
    let vault_address: EthAddress = rtd_bridge.vault().call().await?;
    let committee = EthBridgeCommittee::new(committee_address, provider.clone());
    let config_address: EthAddress = committee.config().call().await?;

    let limiter = EthBridgeLimiter::new(limiter_address, provider.clone());
    let vault = EthBridgeVault::new(vault_address, provider.clone());
    let config = EthBridgeConfig::new(config_address, provider.clone());
    Ok(EthBridgeContracts {
        bridge: rtd_bridge,
        committee,
        limiter,
        vault,
        config,
    })
}

/// Read bridge key from a file and print the corresponding information.
/// If `is_validator_key` is true, the key must be a Secp256k1 key.
pub fn examine_key(path: &PathBuf, is_validator_key: bool) -> Result<(), anyhow::Error> {
    let key = read_key(path, is_validator_key)?;
    let rtd_address = RtdAddress::from(&key.public());
    let pubkey = match key {
        RtdKeyPair::Secp256k1(kp) => {
            println!("Secp256k1 key:");
            let eth_address = BridgeAuthorityPublicKeyBytes::from(&kp.public).to_eth_address();
            println!("Corresponding Ethereum address: {:x}", eth_address);
            kp.public.as_bytes().to_vec()
        }
        RtdKeyPair::Ed25519(kp) => {
            println!("Ed25519 key:");
            kp.public().as_bytes().to_vec()
        }
        RtdKeyPair::Secp256r1(kp) => {
            println!("Secp256r1 key:");
            kp.public().as_bytes().to_vec()
        }
    };
    println!("Corresponding Rtd address: {:?}", rtd_address);
    println!("Corresponding PublicKey: {:?}", Hex::encode(pubkey));
    Ok(())
}

/// Generate Bridge Node Config template and write to a file.
pub fn generate_bridge_node_config_and_write_to_file(
    path: &PathBuf,
    run_client: bool,
) -> Result<(), anyhow::Error> {
    let mut config = BridgeNodeConfig {
        server_listen_port: 9191,
        metrics_port: 9184,
        bridge_authority_key_path: PathBuf::from("/path/to/your/bridge_authority_key"),
        rtd: RtdConfig {
            rtd_rpc_url: "your_rtd_rpc_url".to_string(),
            rtd_bridge_chain_id: BridgeChainId::RtdTestnet as u8,
            bridge_client_key_path: None,
            bridge_client_gas_object: None,
            rtd_bridge_module_last_processed_event_id_override: None,
        },
        eth: EthConfig {
            eth_rpc_url: "your_eth_rpc_url".to_string(),
            eth_bridge_proxy_address: "0x0000000000000000000000000000000000000000".to_string(),
            eth_bridge_chain_id: BridgeChainId::EthSepolia as u8,
            eth_contracts_start_block_fallback: Some(0),
            eth_contracts_start_block_override: None,
        },
        approved_governance_actions: vec![],
        run_client,
        db_path: None,
        metrics_key_pair: default_ed25519_key_pair(),
        metrics: Some(MetricsConfig {
            push_interval_seconds: None, // use default value
            push_url: "metrics_proxy_url".to_string(),
        }),
        watchdog_config: Some(WatchdogConfig {
            total_supplies: BTreeMap::from_iter(vec![(
                "eth".to_string(),
                "0xd0e89b2af5e4910726fbcd8b8dd37bb79b29e5f83f7491bca830e94f7f226d29::eth::ETH"
                    .to_string(),
            )]),
        }),
    };
    if run_client {
        config.rtd.bridge_client_key_path = Some(PathBuf::from("/path/to/your/bridge_client_key"));
        config.db_path = Some(PathBuf::from("/path/to/your/client_db"));
    }
    config.save(path)
}

pub async fn get_eth_signer_client(url: &str, private_key_hex: &str) -> anyhow::Result<EthSigner> {
    let provider = Provider::<Http>::try_from(url)
        .unwrap()
        .interval(std::time::Duration::from_millis(2000));
    let chain_id = provider.get_chainid().await?;
    let wallet = Wallet::from_str(private_key_hex)
        .unwrap()
        .with_chain_id(chain_id.as_u64());
    Ok(SignerMiddleware::new(provider, wallet))
}

pub async fn publish_and_register_coins_return_add_coins_on_rtd_action(
    wallet_context: &WalletContext,
    bridge_arg: ObjectArg,
    token_packages_dir: Vec<PathBuf>,
    token_ids: Vec<u8>,
    token_prices: Vec<u64>,
    nonce: u64,
) -> BridgeAction {
    assert!(token_ids.len() == token_packages_dir.len());
    assert!(token_prices.len() == token_packages_dir.len());
    let rtd_client = wallet_context.get_client().await.unwrap();
    let quorum_driver_api = Arc::new(rtd_client.quorum_driver_api().clone());
    let rgp = rtd_client
        .governance_api()
        .get_reference_gas_price()
        .await
        .unwrap();

    let senders = wallet_context.get_addresses();
    // We want each sender to deal with one coin
    assert!(senders.len() >= token_packages_dir.len());

    // publish coin packages
    let mut publish_tokens_tasks = vec![];

    for (token_package_dir, sender) in token_packages_dir.iter().zip(senders.clone()) {
        let gas = wallet_context
            .get_one_gas_object_owned_by_address(sender)
            .await
            .unwrap()
            .unwrap();
        let tx = TestTransactionBuilder::new(sender, gas, rgp)
            .publish(token_package_dir.to_path_buf())
            .build();
        let tx = wallet_context.sign_transaction(&tx).await;
        let api_clone = quorum_driver_api.clone();
        publish_tokens_tasks.push(tokio::spawn(async move {
            api_clone.execute_transaction_block(
                tx,
                RtdTransactionBlockResponseOptions::new()
                    .with_effects()
                    .with_input()
                    .with_events()
                    .with_object_changes()
                    .with_balance_changes(),
                Some(rtd_types::quorum_driver_types::ExecuteTransactionRequestType::WaitForLocalExecution),
            ).await
        }));
    }
    let publish_coin_responses = join_all(publish_tokens_tasks).await;

    let mut token_type_names = vec![];
    let mut register_tasks = vec![];
    for (response, sender) in publish_coin_responses.into_iter().zip(senders.clone()) {
        let response = response.unwrap().unwrap();
        assert_eq!(
            response.effects.unwrap().status(),
            &RtdExecutionStatus::Success
        );
        let object_changes = response.object_changes.unwrap();
        let mut tc = None;
        let mut type_ = None;
        let mut uc = None;
        let mut metadata = None;
        for object_change in &object_changes {
            if let o @ rtd_json_rpc_types::ObjectChange::Created { object_type, .. } = object_change
            {
                if object_type.name.as_str().starts_with("TreasuryCap") {
                    assert!(tc.is_none() && type_.is_none());
                    tc = Some(o.clone());
                    type_ = Some(object_type.type_params.first().unwrap().clone());
                } else if object_type.name.as_str().starts_with("UpgradeCap") {
                    assert!(uc.is_none());
                    uc = Some(o.clone());
                } else if object_type.name.as_str().starts_with("CoinMetadata") {
                    assert!(metadata.is_none());
                    metadata = Some(o.clone());
                }
            }
        }
        let (tc, type_, uc, metadata) =
            (tc.unwrap(), type_.unwrap(), uc.unwrap(), metadata.unwrap());

        // register with the bridge
        let mut builder = ProgrammableTransactionBuilder::new();
        let bridge_arg = builder.obj(bridge_arg).unwrap();
        let uc_arg = builder
            .obj(ObjectArg::ImmOrOwnedObject(uc.object_ref()))
            .unwrap();
        let tc_arg = builder
            .obj(ObjectArg::ImmOrOwnedObject(tc.object_ref()))
            .unwrap();
        let metadata_arg = builder
            .obj(ObjectArg::ImmOrOwnedObject(metadata.object_ref()))
            .unwrap();
        builder.programmable_move_call(
            BRIDGE_PACKAGE_ID,
            BRIDGE_MODULE_NAME.into(),
            BRIDGE_REGISTER_FOREIGN_TOKEN_FUNCTION_NAME.into(),
            vec![type_.clone()],
            vec![bridge_arg, tc_arg, uc_arg, metadata_arg],
        );
        let pt = builder.finish();
        let gas = wallet_context
            .get_one_gas_object_owned_by_address(sender)
            .await
            .unwrap()
            .unwrap();
        let tx = TransactionData::new_programmable(sender, vec![gas], pt, 1_000_000_000, rgp);
        let signed_tx = wallet_context.sign_transaction(&tx).await;
        let api_clone = quorum_driver_api.clone();
        register_tasks.push(async move {
            api_clone
                .execute_transaction_block(
                    signed_tx,
                    RtdTransactionBlockResponseOptions::new().with_effects(),
                    None,
                )
                .await
        });
        token_type_names.push(type_);
    }
    for response in join_all(register_tasks).await {
        assert_eq!(
            response.unwrap().effects.unwrap().status(),
            &RtdExecutionStatus::Success
        );
    }

    BridgeAction::AddTokensOnRtdAction(AddTokensOnRtdAction {
        nonce,
        chain_id: BridgeChainId::RtdCustom,
        native: false,
        token_ids,
        token_type_names,
        token_prices,
    })
}

pub async fn wait_for_server_to_be_up(server_url: String, timeout_sec: u64) -> anyhow::Result<()> {
    let now = std::time::Instant::now();
    loop {
        if let Ok(true) = reqwest::Client::new()
            .get(server_url.clone())
            .header(reqwest::header::ACCEPT, APPLICATION_JSON)
            .send()
            .await
            .map(|res| res.status().is_success())
        {
            break;
        }
        if now.elapsed().as_secs() > timeout_sec {
            anyhow::bail!("Server is not up and running after {} seconds", timeout_sec);
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    Ok(())
}

/// Return a mappping from validator name to their bridge voting power.
/// If a validator is not in the Rtd committee, we will use its base URL as the name.
pub async fn get_committee_voting_power_by_name(
    bridge_committee: &Arc<BridgeCommittee>,
    system_state: &RtdSystemStateSummary,
) -> BTreeMap<String, StakeUnit> {
    let mut rtd_committee: BTreeMap<_, _> = system_state
        .active_validators
        .iter()
        .map(|v| (v.rtd_address, v.name.clone()))
        .collect();
    bridge_committee
        .members()
        .iter()
        .map(|v| {
            (
                rtd_committee
                    .remove(&v.1.rtd_address)
                    .unwrap_or(v.1.base_url.clone()),
                v.1.voting_power,
            )
        })
        .collect()
}

/// Return a mappping from validator pub keys to their names.
/// If a validator is not in the Rtd committee, we will use its base URL as the name.
pub async fn get_validator_names_by_pub_keys(
    bridge_committee: &Arc<BridgeCommittee>,
    system_state: &RtdSystemStateSummary,
) -> BTreeMap<BridgeAuthorityPublicKeyBytes, String> {
    let mut rtd_committee: BTreeMap<_, _> = system_state
        .active_validators
        .iter()
        .map(|v| (v.rtd_address, v.name.clone()))
        .collect();
    bridge_committee
        .members()
        .iter()
        .map(|(name, validator)| {
            (
                name.clone(),
                rtd_committee
                    .remove(&validator.rtd_address)
                    .unwrap_or(validator.base_url.clone()),
            )
        })
        .collect()
}
