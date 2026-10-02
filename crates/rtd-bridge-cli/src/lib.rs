// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use alloy::dyn_abi::DynSolValue;
use alloy::primitives::{Address as EthAddress, Bytes, U256};
use alloy::providers::{Provider, WalletProvider};
use anyhow::anyhow;
use clap::*;
use fastcrypto::encoding::Encoding;
use fastcrypto::encoding::Hex;
use fastcrypto::hash::{HashFunction, Keccak256};
use linku_common::ZipDebugEqIteratorExt;
use move_core_types::ident_str;
use rtd_bridge::abi::EthBridgeCommittee;
use rtd_bridge::abi::{EthRtdBridge, eth_rtd_bridge};
use rtd_bridge::crypto::BridgeAuthorityPublicKeyBytes;
use rtd_bridge::encoding::TOKEN_TRANSFER_MESSAGE_VERSION_V2;
use rtd_bridge::rtd_client::RtdBridgeClient;
use rtd_bridge::types::BridgeAction;
use rtd_bridge::types::{
    AddTokensOnEvmAction, AddTokensOnRtdAction, AssetPriceUpdateAction, BlocklistCommitteeAction,
    BlocklistType, EmergencyAction, EmergencyActionType, EvmContractUpgradeAction,
    LimitUpdateAction,
};
use rtd_bridge::utils::{EthSignerProvider, get_eth_signer_provider};
use rtd_config::Config;
use rtd_keys::keypair_file::read_key;
use rtd_rpc::field::{FieldMask, FieldMaskUtil};
use rtd_rpc::proto::rtd::rpc::v2::GetObjectRequest;
use rtd_rpc_api::Client;
use rtd_types::base_types::RtdAddress;
use rtd_types::base_types::{ObjectID, ObjectRef};
use rtd_types::bridge::{BRIDGE_MODULE_NAME, BridgeChainId};
use rtd_types::crypto::{RtdKeyPair, Signature};
use rtd_types::gas_coin::{GAS, GasCoin};
use rtd_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use rtd_types::transaction::{CallArg, ObjectArg, Transaction, TransactionData};
use rtd_types::{BRIDGE_PACKAGE_ID, TypeTag};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use shared_crypto::intent::Intent;
use shared_crypto::intent::IntentMessage;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

pub const SEPOLIA_BRIDGE_PROXY_ADDR: &str = "0xAE68F87938439afEEDd6552B0E83D2CbC2473623";

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct Args {
    #[clap(subcommand)]
    pub command: BridgeCommand,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
pub enum Network {
    Testnet,
}

/// Bridge message version. V2 adds timestamp-awareness for limiter bypass on mature messages.
#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum BridgeVersion {
    V1,
    V2,
}

impl std::fmt::Display for BridgeVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeVersion::V1 => write!(f, "v1"),
            BridgeVersion::V2 => write!(f, "v2"),
        }
    }
}

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum BridgeCommand {
    #[clap(name = "create-bridge-validator-key")]
    CreateBridgeValidatorKey { path: PathBuf },
    #[clap(name = "create-bridge-client-key")]
    CreateBridgeClientKey {
        path: PathBuf,
        #[clap(long = "use-ecdsa", default_value = "false")]
        use_ecdsa: bool,
    },
    /// Read bridge key from a file and print related information
    /// If `is-validator-key` is true, the key must be a secp256k1 key
    #[clap(name = "examine-key")]
    ExamineKey {
        path: PathBuf,
        #[clap(long = "is-validator-key")]
        is_validator_key: bool,
    },
    #[clap(name = "create-bridge-node-config-template")]
    CreateBridgeNodeConfigTemplate {
        path: PathBuf,
        #[clap(long = "run-client")]
        run_client: bool,
    },
    /// Governance client to facilitate and execute Bridge governance actions
    #[clap(name = "governance")]
    Governance {
        /// Path of BridgeCliConfig
        #[clap(long = "config-path")]
        config_path: PathBuf,
        #[clap(long = "chain-id")]
        chain_id: u8,
        #[clap(subcommand)]
        cmd: GovernanceClientCommands,
        /// If true, only collect signatures but not execute on chain
        #[clap(long = "dry-run")]
        dry_run: bool,
    },
    /// View current status of Eth bridge
    #[clap(name = "view-eth-bridge")]
    ViewEthBridge {
        #[clap(long = "network")]
        network: Option<Network>,
        #[clap(long = "bridge-proxy")]
        bridge_proxy: Option<EthAddress>,
        #[clap(long = "eth-rpc-url")]
        eth_rpc_url: String,
    },
    /// View current list of registered validators
    #[clap(name = "view-bridge-registration")]
    ViewBridgeRegistration {
        #[clap(long = "rtd-rpc-url")]
        rtd_rpc_url: String,
    },
    /// View current status of Rtd bridge
    #[clap(name = "view-rtd-bridge")]
    ViewRtdBridge {
        #[clap(long = "rtd-rpc-url")]
        rtd_rpc_url: String,
        #[clap(long, default_value = "false")]
        hex: bool,
        #[clap(long, default_value = "false")]
        ping: bool,
    },
    /// Client to facilitate and execute Bridge actions
    #[clap(name = "client")]
    Client {
        /// Path of BridgeCliConfig
        #[clap(long = "config-path")]
        config_path: PathBuf,
        #[clap(subcommand)]
        cmd: BridgeClientCommands,
    },
}

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum GovernanceClientCommands {
    #[clap(name = "emergency-button")]
    EmergencyButton {
        #[clap(name = "nonce", long)]
        nonce: u64,
        #[clap(name = "action-type", long)]
        action_type: EmergencyActionType,
    },
    #[clap(name = "update-committee-blocklist")]
    UpdateCommitteeBlocklist {
        #[clap(name = "nonce", long)]
        nonce: u64,
        #[clap(name = "blocklist-type", long)]
        blocklist_type: BlocklistType,
        #[clap(name = "pubkey-hex", use_value_delimiter = true, long)]
        pubkeys_hex: Vec<BridgeAuthorityPublicKeyBytes>,
    },
    #[clap(name = "update-limit")]
    UpdateLimit {
        #[clap(name = "nonce", long)]
        nonce: u64,
        #[clap(name = "sending-chain", long)]
        sending_chain: u8,
        #[clap(name = "new-usd-limit", long)]
        new_usd_limit: u64,
    },
    #[clap(name = "update-asset-price")]
    UpdateAssetPrice {
        #[clap(name = "nonce", long)]
        nonce: u64,
        #[clap(name = "token-id", long)]
        token_id: u8,
        #[clap(name = "new-usd-price", long)]
        new_usd_price: u64,
    },
    #[clap(name = "add-tokens-on-rtd")]
    AddTokensOnRtd {
        #[clap(name = "nonce", long)]
        nonce: u64,
        #[clap(name = "token-ids", use_value_delimiter = true, long)]
        token_ids: Vec<u8>,
        #[clap(name = "token-type-names", use_value_delimiter = true, long)]
        token_type_names: Vec<TypeTag>,
        #[clap(name = "token-prices", use_value_delimiter = true, long)]
        token_prices: Vec<u64>,
    },
    #[clap(name = "add-tokens-on-evm")]
    AddTokensOnEvm {
        #[clap(name = "nonce", long)]
        nonce: u64,
        #[clap(name = "token-ids", use_value_delimiter = true, long)]
        token_ids: Vec<u8>,
        #[clap(name = "token-type-names", use_value_delimiter = true, long)]
        token_addresses: Vec<EthAddress>,
        #[clap(name = "token-prices", use_value_delimiter = true, long)]
        token_prices: Vec<u64>,
        #[clap(name = "token-rtd-decimals", use_value_delimiter = true, long)]
        token_rtd_decimals: Vec<u8>,
    },
    #[clap(name = "upgrade-evm-contract")]
    UpgradeEVMContract {
        #[clap(name = "nonce", long)]
        nonce: u64,
        #[clap(name = "proxy-address", long)]
        proxy_address: EthAddress,
        /// The address of the new implementation contract
        #[clap(name = "implementation-address", long)]
        implementation_address: EthAddress,
        /// Function selector with params types, e.g. `foo(uint256,bool,string)`
        #[clap(name = "function-selector", long)]
        function_selector: Option<String>,
        /// Params to be passed to the function, e.g. `420,false,hello`
        #[clap(name = "params", use_value_delimiter = true, long)]
        params: Vec<String>,
    },
}

pub fn make_action(chain_id: BridgeChainId, cmd: &GovernanceClientCommands) -> BridgeAction {
    match cmd {
        GovernanceClientCommands::EmergencyButton { nonce, action_type } => {
            BridgeAction::EmergencyAction(EmergencyAction {
                nonce: *nonce,
                chain_id,
                action_type: *action_type,
            })
        }
        GovernanceClientCommands::UpdateCommitteeBlocklist {
            nonce,
            blocklist_type,
            pubkeys_hex,
        } => BridgeAction::BlocklistCommitteeAction(BlocklistCommitteeAction {
            nonce: *nonce,
            chain_id,
            blocklist_type: *blocklist_type,
            members_to_update: pubkeys_hex.clone(),
        }),
        GovernanceClientCommands::UpdateLimit {
            nonce,
            sending_chain,
            new_usd_limit,
        } => {
            let sending_chain_id =
                BridgeChainId::try_from(*sending_chain).expect("Invalid sending chain id");
            BridgeAction::LimitUpdateAction(LimitUpdateAction {
                nonce: *nonce,
                chain_id,
                sending_chain_id,
                new_usd_limit: *new_usd_limit,
            })
        }
        GovernanceClientCommands::UpdateAssetPrice {
            nonce,
            token_id,
            new_usd_price,
        } => BridgeAction::AssetPriceUpdateAction(AssetPriceUpdateAction {
            nonce: *nonce,
            chain_id,
            token_id: *token_id,
            new_usd_price: *new_usd_price,
        }),
        GovernanceClientCommands::AddTokensOnRtd {
            nonce,
            token_ids,
            token_type_names,
            token_prices,
        } => {
            assert_eq!(token_ids.len(), token_type_names.len());
            assert_eq!(token_ids.len(), token_prices.len());
            BridgeAction::AddTokensOnRtdAction(AddTokensOnRtdAction {
                nonce: *nonce,
                chain_id,
                native: false, // only foreign tokens are supported now
                token_ids: token_ids.clone(),
                token_type_names: token_type_names.clone(),
                token_prices: token_prices.clone(),
            })
        }
        GovernanceClientCommands::AddTokensOnEvm {
            nonce,
            token_ids,
            token_addresses,
            token_prices,
            token_rtd_decimals,
        } => {
            assert_eq!(token_ids.len(), token_addresses.len());
            assert_eq!(token_ids.len(), token_prices.len());
            assert_eq!(token_ids.len(), token_rtd_decimals.len());
            BridgeAction::AddTokensOnEvmAction(AddTokensOnEvmAction {
                nonce: *nonce,
                native: true, // only eth native tokens are supported now
                chain_id,
                token_ids: token_ids.clone(),
                token_addresses: token_addresses.clone(),
                token_prices: token_prices.clone(),
                token_rtd_decimals: token_rtd_decimals.clone(),
            })
        }
        GovernanceClientCommands::UpgradeEVMContract {
            nonce,
            proxy_address,
            implementation_address,
            function_selector,
            params,
        } => {
            let call_data = match function_selector {
                Some(function_selector) => encode_call_data(function_selector, params),
                None => vec![],
            };
            BridgeAction::EvmContractUpgradeAction(EvmContractUpgradeAction {
                nonce: *nonce,
                chain_id,
                proxy_address: *proxy_address,
                new_impl_address: *implementation_address,
                call_data,
            })
        }
    }
}

fn encode_call_data(function_selector: &str, params: &[String]) -> Vec<u8> {
    let left = function_selector
        .find('(')
        .expect("Invalid function selector, no left parentheses");
    let right = function_selector
        .find(')')
        .expect("Invalid function selector, no right parentheses");
    let param_types = function_selector[left + 1..right]
        .split(',')
        .map(|x| x.trim())
        .collect::<Vec<&str>>();

    assert_eq!(param_types.len(), params.len(), "Invalid number of params");

    let mut tokens = vec![];
    for (param, param_type) in params.iter().zip_debug_eq(param_types.iter()) {
        let token = match param_type.to_lowercase().as_str() {
            "uint256" => {
                DynSolValue::Uint(U256::from_str_radix(param, 10).expect("Invalid U256"), 256)
            }
            "bool" => DynSolValue::Bool(match param.as_str() {
                "true" => true,
                "false" => false,
                _ => panic!("Invalid bool in params"),
            }),
            "string" => DynSolValue::String(param.clone()),
            // TODO: need to support more types if needed
            _ => panic!("Invalid param type"),
        };
        tokens.push(token);
    }

    let mut call_data = Keccak256::digest(function_selector).digest[0..4].to_vec();
    if !tokens.is_empty() {
        call_data.extend(DynSolValue::Tuple(tokens).abi_encode());
    }
    call_data
}

pub fn select_contract_address(
    config: &LoadedBridgeCliConfig,
    cmd: &GovernanceClientCommands,
) -> EthAddress {
    match cmd {
        GovernanceClientCommands::EmergencyButton { .. } => config.eth_bridge_proxy_address,
        GovernanceClientCommands::UpdateCommitteeBlocklist { .. } => {
            config.eth_bridge_committee_proxy_address
        }
        GovernanceClientCommands::UpdateLimit { .. } => config.eth_bridge_limiter_proxy_address,
        GovernanceClientCommands::UpdateAssetPrice { .. } => config.eth_bridge_config_proxy_address,
        GovernanceClientCommands::UpgradeEVMContract { proxy_address, .. } => *proxy_address,
        GovernanceClientCommands::AddTokensOnRtd { .. } => unreachable!(),
        GovernanceClientCommands::AddTokensOnEvm { .. } => config.eth_bridge_config_proxy_address,
    }
}

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct BridgeCliConfig {
    /// Rpc url for Rtd fullnode, used for query stuff and submit transactions.
    pub rtd_rpc_url: String,
    /// Rpc url for Eth fullnode, used for query stuff.
    pub eth_rpc_url: String,
    /// Proxy address for RtdBridge deployed on Eth
    pub eth_bridge_proxy_address: EthAddress,
    /// Path of the file where private key is stored. The content could be any of the following:
    /// - Base64 encoded `flag || privkey` for ECDSA key
    /// - Base64 encoded `privkey` for Raw key
    /// - Hex encoded `privkey` for Raw key
    /// At leaset one of `rtd_key_path` or `eth_key_path` must be provided.
    /// If only one is provided, it will be used for both Rtd and Eth.
    pub rtd_key_path: Option<PathBuf>,
    /// See `rtd_key_path`. Must be Secp256k1 key.
    pub eth_key_path: Option<PathBuf>,
}

impl Config for BridgeCliConfig {}

pub struct LoadedBridgeCliConfig {
    /// Rpc url for Rtd fullnode, used for query stuff and submit transactions.
    pub rtd_rpc_url: String,
    /// Rpc url for Eth fullnode, used for query stuff.
    pub eth_rpc_url: String,
    /// Proxy address for RtdBridge deployed on Eth
    pub eth_bridge_proxy_address: EthAddress,
    /// Proxy address for BridgeCommittee deployed on Eth
    pub eth_bridge_committee_proxy_address: EthAddress,
    /// Proxy address for BridgeConfig deployed on Eth
    pub eth_bridge_config_proxy_address: EthAddress,
    /// Proxy address for BridgeLimiter deployed on Eth
    pub eth_bridge_limiter_proxy_address: EthAddress,
    /// Key pair for Rtd operations
    rtd_key: RtdKeyPair,
    /// Key pair for Eth operations, must be Secp256k1 key
    eth_signer_provider: EthSignerProvider,
}

impl LoadedBridgeCliConfig {
    pub async fn load(cli_config: BridgeCliConfig) -> anyhow::Result<Self> {
        if cli_config.eth_key_path.is_none() && cli_config.rtd_key_path.is_none() {
            return Err(anyhow!(
                "At least one of `rtd_key_path` or `eth_key_path` must be provided"
            ));
        }
        let rtd_key = if let Some(rtd_key_path) = &cli_config.rtd_key_path {
            Some(read_key(rtd_key_path, false)?)
        } else {
            None
        };
        let eth_key = if let Some(eth_key_path) = &cli_config.eth_key_path {
            let eth_key = read_key(eth_key_path, true)?;
            Some(eth_key)
        } else {
            None
        };
        let (eth_key, rtd_key) = match (eth_key, rtd_key) {
            (None, Some(rtd_key)) => {
                if !matches!(rtd_key, RtdKeyPair::Secp256k1(_)) {
                    return Err(anyhow!("Eth key must be an ECDSA key"));
                }
                (rtd_key.copy(), rtd_key)
            }
            (Some(eth_key), None) => (eth_key.copy(), eth_key),
            (Some(eth_key), Some(rtd_key)) => (eth_key, rtd_key),
            (None, None) => unreachable!(),
        };

        let private_key_hex = Hex::encode(eth_key.to_bytes_no_flag());
        let eth_signer_provider =
            get_eth_signer_provider(&cli_config.eth_rpc_url, &private_key_hex)?;
        let rtd_bridge = EthRtdBridge::new(
            cli_config.eth_bridge_proxy_address,
            eth_signer_provider.clone(),
        );
        let eth_bridge_committee_proxy_address: EthAddress = rtd_bridge.committee().call().await?;
        let eth_bridge_limiter_proxy_address: EthAddress = rtd_bridge.limiter().call().await?;
        let eth_committee = EthBridgeCommittee::new(
            eth_bridge_committee_proxy_address,
            eth_signer_provider.clone(),
        );
        let eth_bridge_committee_proxy_address: EthAddress = rtd_bridge.committee().call().await?;
        let eth_bridge_config_proxy_address: EthAddress = eth_committee.config().call().await?;

        let eth_address = eth_signer_provider.default_signer_address();
        let eth_chain_id = eth_signer_provider.get_chain_id().await?;
        let rtd_address = RtdAddress::from(&rtd_key.public());
        println!("Using Rtd address: {:?}", rtd_address);
        println!("Using Eth address: {:?}", eth_address);
        println!("Using Eth chain: {:?}", eth_chain_id);

        Ok(Self {
            rtd_rpc_url: cli_config.rtd_rpc_url,
            eth_rpc_url: cli_config.eth_rpc_url,
            eth_bridge_proxy_address: cli_config.eth_bridge_proxy_address,
            eth_bridge_committee_proxy_address,
            eth_bridge_limiter_proxy_address,
            eth_bridge_config_proxy_address,
            rtd_key,
            eth_signer_provider,
        })
    }
}

impl LoadedBridgeCliConfig {
    pub fn eth_signer_provider(self: &LoadedBridgeCliConfig) -> EthSignerProvider {
        self.eth_signer_provider.clone()
    }

    pub async fn get_rtd_account_info(
        self: &LoadedBridgeCliConfig,
    ) -> anyhow::Result<(RtdKeyPair, RtdAddress, ObjectRef)> {
        let pubkey = self.rtd_key.public();
        let rtd_client_address = RtdAddress::from(&pubkey);
        let rtd_client = Client::new(&self.rtd_rpc_url)?;
        let gases = rtd_client
            .get_owned_objects(rtd_client_address, Some(GasCoin::type_()), None, None)
            .await?
            .items;
        // TODO: is 5 Rtd a good number?
        let gas = gases
            .into_iter()
            .find(|coin| {
                GasCoin::try_from(coin)
                    .ok()
                    .map(|coin| coin.value() >= 5_000_000_000)
                    .unwrap_or(false)
            })
            .ok_or(anyhow!(
                "Did not find gas object with enough balance for {}",
                rtd_client_address
            ))?;
        println!("Using Gas object: {}", gas.id());
        Ok((
            self.rtd_key.copy(),
            rtd_client_address,
            gas.compute_object_reference(),
        ))
    }
}
#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum BridgeClientCommands {
    #[clap(name = "deposit-native-ether-on-eth")]
    DepositNativeEtherOnEth {
        #[clap(long)]
        ether_amount: String,
        #[clap(long)]
        target_chain: u8,
        #[clap(long)]
        rtd_recipient_address: RtdAddress,
        /// Bridge message version (v1 = original, v2 = timestamp-aware with limiter bypass)
        #[clap(long, default_value_t = BridgeVersion::V1, value_enum)]
        bridge_version: BridgeVersion,
    },
    #[clap(name = "deposit-on-rtd")]
    DepositOnRtd {
        #[clap(long)]
        coin_object_id: ObjectID,
        #[clap(long)]
        coin_type: String,
        #[clap(long)]
        target_chain: u8,
        #[clap(long)]
        recipient_address: EthAddress,
        /// Bridge message version (v1 = original, v2 = timestamp-aware with limiter bypass)
        #[clap(long, default_value_t = BridgeVersion::V1, value_enum)]
        bridge_version: BridgeVersion,
    },
    /// Claim bridged tokens on Eth for a Rtd→ETH transfer.
    /// Auto-detects V1/V2 from the on-chain record and calls the appropriate EVM function.
    #[clap(name = "claim-on-eth")]
    ClaimOnEth {
        #[clap(long)]
        seq_num: u64,
        #[clap(long, default_value_t = true, action = clap::ArgAction::Set)]
        dry_run: bool,
    },
    /// Claim bridged tokens on Rtd for an ETH→Rtd transfer that has been approved but not yet claimed.
    /// Auto-detects V1/V2 from the on-chain record (V2 messages >48h old bypass the rate limiter).
    #[clap(name = "claim-on-rtd")]
    ClaimOnRtd {
        /// The bridge sequence number of the ETH→Rtd transfer
        #[clap(long)]
        seq_num: u64,
        /// The source chain ID (the ETH chain from which the transfer originated)
        #[clap(long)]
        source_chain: u8,
        #[clap(long, default_value_t = true, action = clap::ArgAction::Set)]
        dry_run: bool,
    },
}

impl BridgeClientCommands {
    pub async fn handle(
        self,
        config: &LoadedBridgeCliConfig,
        rtd_bridge_client: RtdBridgeClient,
    ) -> anyhow::Result<()> {
        match self {
            BridgeClientCommands::DepositNativeEtherOnEth {
                ether_amount,
                target_chain,
                rtd_recipient_address,
                bridge_version,
            } => {
                deposit_native_ether_on_eth(
                    &ether_amount,
                    target_chain,
                    rtd_recipient_address,
                    config,
                    bridge_version,
                )
                .await
            }
            BridgeClientCommands::DepositOnRtd {
                coin_object_id,
                coin_type,
                target_chain,
                recipient_address,
                bridge_version,
            } => {
                let target_chain = BridgeChainId::try_from(target_chain).expect("Invalid chain id");
                let coin_type = TypeTag::from_str(&coin_type).expect("Invalid coin type");
                deposit_on_rtd(
                    coin_object_id,
                    coin_type,
                    target_chain,
                    recipient_address,
                    config,
                    rtd_bridge_client,
                    bridge_version,
                )
                .await
            }
            BridgeClientCommands::ClaimOnEth { seq_num, dry_run } => {
                claim_on_eth(seq_num, config, rtd_bridge_client, dry_run).await
            }
            BridgeClientCommands::ClaimOnRtd {
                seq_num,
                source_chain,
                dry_run,
            } => claim_on_rtd(seq_num, source_chain, config, rtd_bridge_client, dry_run).await,
        }
    }
}

async fn deposit_native_ether_on_eth(
    ether_amount: &str,
    target_chain: u8,
    rtd_recipient_address: RtdAddress,
    config: &LoadedBridgeCliConfig,
    version: BridgeVersion,
) -> anyhow::Result<()> {
    let eth_rtd_bridge = EthRtdBridge::new(
        config.eth_bridge_proxy_address,
        config.eth_signer_provider().clone(),
    );
    let amount: U256 = alloy::primitives::utils::parse_units(ether_amount, "ether")?.into();
    let pending_tx = match version {
        BridgeVersion::V2 => {
            eth_rtd_bridge
                .bridgeETHV2(rtd_recipient_address.to_vec().into(), target_chain)
                .value(amount)
                .send()
                .await?
        }
        BridgeVersion::V1 => {
            eth_rtd_bridge
                .bridgeETH(rtd_recipient_address.to_vec().into(), target_chain)
                .value(amount)
                .send()
                .await?
        }
    };
    let tx_receipt = pending_tx.get_receipt().await?;
    info!(
        "Deposited {ether_amount} Ethers ({version:?}) to {:?} (target chain {target_chain}). Receipt: {:?}",
        rtd_recipient_address, tx_receipt,
    );
    Ok(())
}

async fn deposit_on_rtd(
    coin_object_id: ObjectID,
    coin_type: TypeTag,
    target_chain: BridgeChainId,
    recipient_address: EthAddress,
    config: &LoadedBridgeCliConfig,
    rtd_bridge_client: RtdBridgeClient,
    version: BridgeVersion,
) -> anyhow::Result<()> {
    let target_chain = target_chain as u8;
    let mut rtd_client = rtd_bridge_client.grpc_client().clone();
    let bridge_object_arg = rtd_bridge_client
        .get_mutable_bridge_object_arg_must_succeed()
        .await;
    let rgp = rtd_client.get_reference_gas_price().await.unwrap();
    let sender = RtdAddress::from(&config.rtd_key.public());
    let gas_type = rtd_sdk_types::TypeTag::from_str(&GAS::type_().to_canonical_string(true))?;
    let gas_obj_ref = rtd_client
        .inner_mut()
        .select_coins(&sender.into(), &gas_type, 1_000_000_000, &[])
        .await?
        .into_iter()
        .map(|coin| {
            (
                coin.object_id().parse().unwrap(),
                coin.version().into(),
                coin.digest().parse().unwrap(),
            )
        })
        .collect();
    let coin_obj = rtd_client
        .inner_mut()
        .ledger_client()
        .get_object(
            GetObjectRequest::new(&(coin_object_id.into()))
                .with_read_mask(FieldMask::from_paths(["object_id", "version", "digest"])),
        )
        .await?
        .into_inner()
        .object
        .unwrap_or_default();
    let coin_obj_ref = (
        coin_obj.object_id().parse()?,
        coin_obj.version().into(),
        coin_obj.digest().parse()?,
    );

    let mut builder = ProgrammableTransactionBuilder::new();
    let arg_target_chain = builder.pure(target_chain).unwrap();
    let arg_target_address = builder.pure(recipient_address.as_slice()).unwrap();
    let arg_token = builder
        .obj(ObjectArg::ImmOrOwnedObject(coin_obj_ref))
        .unwrap();
    let arg_bridge = builder.obj(bridge_object_arg).unwrap();

    match version {
        BridgeVersion::V2 => {
            let arg_clock = builder.input(CallArg::CLOCK_IMM).unwrap();
            builder.programmable_move_call(
                BRIDGE_PACKAGE_ID,
                BRIDGE_MODULE_NAME.to_owned(),
                ident_str!("send_token_v2").to_owned(),
                vec![coin_type],
                vec![
                    arg_bridge,
                    arg_target_chain,
                    arg_target_address,
                    arg_token,
                    arg_clock,
                ],
            );
        }
        BridgeVersion::V1 => {
            builder.programmable_move_call(
                BRIDGE_PACKAGE_ID,
                BRIDGE_MODULE_NAME.to_owned(),
                ident_str!("send_token").to_owned(),
                vec![coin_type],
                vec![arg_bridge, arg_target_chain, arg_target_address, arg_token],
            );
        }
    }
    let pt = builder.finish();
    let tx_data = TransactionData::new_programmable(sender, gas_obj_ref, pt, 500_000_000, rgp);
    let sig = Signature::new_secure(
        &IntentMessage::new(Intent::rtd_transaction(), tx_data.clone()),
        &config.rtd_key,
    );
    let signed_tx = Transaction::from_data(tx_data, vec![sig]);
    let tx_digest = *signed_tx.digest();
    info!(
        ?tx_digest,
        "Sending deposit transaction ({version:?}) to Rtd."
    );
    let resp = rtd_bridge_client
        .execute_transaction_block_with_effects(signed_tx)
        .await
        .expect("Failed to execute transaction block");
    match &resp.status {
        rtd_json_rpc_types::RtdExecutionStatus::Success => {
            info!(
                ?tx_digest,
                "Deposit transaction ({version:?}) succeeded. Events: {:?}", resp.events
            );
            Ok(())
        }
        rtd_json_rpc_types::RtdExecutionStatus::Failure { error } => Err(anyhow!(
            "Deposit ({version:?}) transaction {:?} failed: {:?}",
            tx_digest,
            error
        )),
    }
}

async fn claim_on_eth(
    seq_num: u64,
    config: &LoadedBridgeCliConfig,
    rtd_bridge_client: RtdBridgeClient,
    dry_run: bool,
) -> anyhow::Result<()> {
    let rtd_chain_id = rtd_bridge_client
        .get_bridge_summary()
        .await
        .map_err(|e| anyhow!("{:?}", e))?
        .chain_id;
    let parsed_message = rtd_bridge_client
        .get_parsed_token_transfer_message(rtd_chain_id, seq_num)
        .await
        .map_err(|e| anyhow!("{:?}", e))?;
    if parsed_message.is_none() {
        println!("No record found for seq_num: {seq_num}, chain id: {rtd_chain_id}");
        return Ok(());
    }
    let parsed_message = parsed_message.unwrap();
    let message_version = parsed_message.message_version;
    let version_label = if message_version == TOKEN_TRANSFER_MESSAGE_VERSION_V2 {
        "V2"
    } else {
        "V1"
    };

    let sigs = rtd_bridge_client
        .get_token_transfer_action_onchain_signatures_until_success(rtd_chain_id, seq_num)
        .await;
    if sigs.is_none() {
        println!("No signatures found for seq_num: {seq_num}, chain id: {rtd_chain_id}");
        return Ok(());
    }
    let signatures = sigs
        .unwrap()
        .into_iter()
        .map(|sig: Vec<u8>| Bytes::from(sig))
        .collect::<Vec<_>>();

    let eth_rtd_bridge = EthRtdBridge::new(
        config.eth_bridge_proxy_address,
        Arc::new(config.eth_signer_provider().clone()),
    );
    let message = eth_rtd_bridge::BridgeUtils::Message::from(parsed_message);
    let tx = if message_version == TOKEN_TRANSFER_MESSAGE_VERSION_V2 {
        eth_rtd_bridge
            .transferBridgedTokensWithSignaturesV2(signatures, message)
            .into_transaction_request()
    } else {
        eth_rtd_bridge
            .transferBridgedTokensWithSignatures(signatures, message)
            .into_transaction_request()
    };

    if dry_run {
        let resp = config.eth_signer_provider.estimate_gas(tx).await?;
        println!(
            "Rtd to Eth bridge transfer ({version_label}) claim dry run result: {:?}",
            resp
        );
    } else {
        let eth_claim_tx_receipt = config
            .eth_signer_provider
            .send_transaction(tx)
            .await?
            .get_receipt()
            .await?;
        println!(
            "Rtd to Eth bridge transfer ({version_label}) claimed: {:?}",
            eth_claim_tx_receipt
        );
    }
    Ok(())
}

async fn claim_on_rtd(
    seq_num: u64,
    source_chain: u8,
    config: &LoadedBridgeCliConfig,
    rtd_bridge_client: RtdBridgeClient,
    dry_run: bool,
) -> anyhow::Result<()> {
    // Look up the on-chain bridge record to determine the token type
    let parsed_message = rtd_bridge_client
        .get_parsed_token_transfer_message(source_chain, seq_num)
        .await
        .map_err(|e| anyhow!("{:?}", e))?;
    let Some(parsed_message) = parsed_message else {
        println!("No record found for seq_num: {seq_num}, source chain: {source_chain}");
        return Ok(());
    };

    let message_version = parsed_message.message_version;
    let version_label = if message_version == TOKEN_TRANSFER_MESSAGE_VERSION_V2 {
        "V2"
    } else {
        "V1"
    };

    let token_type = parsed_message.parsed_payload.token_type;

    // Get the token type tag mapping
    let id_token_map = rtd_bridge_client
        .get_token_id_map()
        .await
        .map_err(|e| anyhow!("{:?}", e))?;
    let type_tag = id_token_map.get(&token_type).ok_or_else(|| {
        anyhow!(
            "Unknown token type {token_type} for seq_num {seq_num}, source chain {source_chain}"
        )
    })?;

    let bridge_object_arg = rtd_bridge_client
        .get_mutable_bridge_object_arg_must_succeed()
        .await;
    let (rtd_key, sender, gas_obj_ref) = config.get_rtd_account_info().await?;
    let rgp = rtd_bridge_client
        .get_reference_gas_price_until_success()
        .await;

    // Build the PTB: call bridge::claim_and_transfer_token<T>(bridge, clock, source_chain, seq_num)
    let mut builder = ProgrammableTransactionBuilder::new();
    let arg_bridge = builder.obj(bridge_object_arg).unwrap();
    let arg_clock = builder.input(CallArg::CLOCK_IMM).unwrap();
    let arg_source_chain = builder.pure(source_chain).unwrap();
    let arg_seq_num = builder.pure(seq_num).unwrap();

    builder.programmable_move_call(
        BRIDGE_PACKAGE_ID,
        BRIDGE_MODULE_NAME.to_owned(),
        ident_str!("claim_and_transfer_token").to_owned(),
        vec![type_tag.clone()],
        vec![arg_bridge, arg_clock, arg_source_chain, arg_seq_num],
    );

    let pt = builder.finish();
    let tx_data =
        TransactionData::new_programmable(sender, vec![gas_obj_ref], pt, 500_000_000, rgp);

    if dry_run {
        let rtd_client = rtd_bridge_client.grpc_client().clone();
        let resp = rtd_client
            .simulate_transaction(&tx_data, true, true)
            .await
            .map_err(|e| anyhow!("Dry run (simulate) failed: {:?}", e))?;
        println!(
            "Claim on Rtd ({version_label}) dry run result for seq_num {seq_num}, source chain {source_chain}: {:?}",
            resp
        );
    } else {
        let sig = Signature::new_secure(
            &IntentMessage::new(Intent::rtd_transaction(), tx_data.clone()),
            &rtd_key,
        );
        let signed_tx = Transaction::from_data(tx_data, vec![sig]);
        let tx_digest = *signed_tx.digest();
        info!(
            ?tx_digest,
            "Sending claim_and_transfer_token ({version_label}) transaction to Rtd for seq_num {seq_num}, source chain {source_chain}."
        );
        let resp = rtd_bridge_client
            .execute_transaction_block_with_effects(signed_tx)
            .await
            .map_err(|e| anyhow!("Failed to execute claim transaction: {:?}", e))?;
        match &resp.status {
            rtd_json_rpc_types::RtdExecutionStatus::Success => {
                info!(
                    ?tx_digest,
                    "Claim ({version_label}) transaction succeeded. Events: {:?}", resp.events
                );
                println!(
                    "Successfully claimed ({version_label}) tokens on Rtd for seq_num: {seq_num}, source chain: {source_chain}"
                );
            }
            rtd_json_rpc_types::RtdExecutionStatus::Failure { error } => {
                return Err(anyhow!(
                    "Claim ({version_label}) transaction {:?} failed: {:?}",
                    tx_digest,
                    error
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::{
        dyn_abi::{DynSolType, DynSolValue},
        json_abi::JsonAbi,
        primitives::U256,
    };

    #[tokio::test]
    async fn test_encode_call_data() {
        let abi_json =
            std::fs::read_to_string("../rtd-bridge/abi/tests/mock_rtd_bridge_v2.json").unwrap();
        let abi: JsonAbi = serde_json::from_str(&abi_json).unwrap();

        let function_selector = "initializeV2Params(uint256,bool,string)";
        let params = vec!["420".to_string(), "false".to_string(), "hello".to_string()];
        let call_data = encode_call_data(function_selector, &params);

        let function = abi
            .functions()
            .find(|f| {
                let selector = f.selector();
                call_data.starts_with(selector.as_ref())
            })
            .expect("Function not found");

        // Decode the data excluding the selector
        let input_types = function
            .inputs
            .iter()
            .map(|param| DynSolType::parse(&param.ty).unwrap())
            .collect::<Vec<_>>();
        let tuple_type = DynSolType::Tuple(input_types);
        let decoded = tuple_type
            .abi_decode(&call_data[4..])
            .expect("Decoding failed");
        let decoded_values = decoded.as_tuple().expect("Expected a tuple");

        assert_eq!(
            decoded_values,
            vec![
                DynSolValue::Uint(U256::from(420), 256),
                DynSolValue::Bool(false),
                DynSolValue::String("hello".to_string())
            ]
        )
    }
}
