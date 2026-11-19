// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Client-side utilities for interacting with a Rtd network from Rust.
//!
//! The JSON-RPC client this crate used to provide was removed together with
//! the fullnode JSON-RPC service. What remains is the wallet and configuration
//! layer shared by the Rtd CLI and the tooling in this repository:
//!
//! * [`wallet_context::WalletContext`] - the keystore-backed wallet used to
//!   sign and execute transactions against a network over gRPC.
//! * [`rtd_client_config::RtdClientConfig`] - the persisted client
//!   configuration (`client.yaml`) and its named network environments.
//! * [`verify_personal_message_signature`] - verification of personal message
//!   signatures through a fullnode.
//!
//! For a general-purpose Rust SDK, use the
//! [`rtd-rust-sdk`](https://github.com/LinkUVerse/rtd-rust-sdk) crate.

pub use rtd_crypto;
pub use rtd_rpc;
pub use rtd_sdk_types;
pub use rtd_types as types;

pub mod digests;
pub mod error;
pub mod rtd_client_config;
pub mod verify_personal_message_signature;
pub mod wallet_context;

pub const RTD_COIN_TYPE: &str = "0x2::rtd::RTD";
pub const RTD_LOCAL_NETWORK_URL: &str = "http://127.0.0.1:9000";
pub const RTD_LOCAL_NETWORK_URL_0: &str = "http://0.0.0.0:9000";
pub const RTD_LOCAL_NETWORK_GAS_URL: &str = "http://127.0.0.1:5003/v2/gas";
pub const RTD_DEVNET_URL: &str = "https://fullnode.devnet.rtd.io:443";
pub const RTD_TESTNET_URL: &str = "https://fullnode.testnet.rtd.io:443";
pub const RTD_MAINNET_URL: &str = "https://fullnode.mainnet.rtd.io:443";
