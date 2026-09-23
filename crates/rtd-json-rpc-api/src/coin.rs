// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use rtd_json_rpc_types::{Balance, CoinPage, RtdCoinMetadata};
use rtd_open_rpc_macros::open_rpc;
use rtd_types::balance::Supply;
use rtd_types::base_types::RtdAddress;

#[open_rpc(namespace = "rtdx", tag = "Coin Query API")]
#[rpc(server, client, namespace = "rtdx")]
pub trait CoinReadApi {
    /// Return all Coin<`coin_type`> objects owned by an address.
    #[method(name = "getCoins")]
    async fn get_coins(
        &self,
        /// the owner's Rtd address
        owner: RtdAddress,
        /// optional type name for the coin (e.g., 0x168da5bf1f48dafc111b0a488fa454aca95e0b5e::usdc::USDC), default to 0x2::rtd::RTD if not specified.
        coin_type: Option<String>,
        /// optional paging cursor
        cursor: Option<String>,
        /// maximum number of items per page
        limit: Option<usize>,
    ) -> RpcResult<CoinPage>;

    /// Return all Coin objects owned by an address.
    #[method(name = "getAllCoins")]
    async fn get_all_coins(
        &self,
        /// the owner's Rtd address
        owner: RtdAddress,
        /// optional paging cursor
        cursor: Option<String>,
        /// maximum number of items per page
        limit: Option<usize>,
    ) -> RpcResult<CoinPage>;

    /// Return the total coin balance for one coin type, owned by the address owner.
    #[method(name = "getBalance")]
    async fn get_balance(
        &self,
        /// the owner's Rtd address
        owner: RtdAddress,
        /// optional type names for the coin (e.g., 0x168da5bf1f48dafc111b0a488fa454aca95e0b5e::usdc::USDC), default to 0x2::rtd::RTD if not specified.
        coin_type: Option<String>,
    ) -> RpcResult<Balance>;

    /// Return the total coin balance for all coin type, owned by the address owner.
    #[method(name = "getAllBalances")]
    async fn get_all_balances(
        &self,
        /// the owner's Rtd address
        owner: RtdAddress,
    ) -> RpcResult<Vec<Balance>>;

    /// Return metadata (e.g., symbol, decimals) for a coin. Note that if the coin's metadata was
    /// wrapped in the transaction that published its marker type, or the latest version of the
    /// metadata object is wrapped or deleted, it will not be found.
    #[method(name = "getCoinMetadata")]
    async fn get_coin_metadata(
        &self,
        /// type name for the coin (e.g., 0x168da5bf1f48dafc111b0a488fa454aca95e0b5e::usdc::USDC)
        coin_type: String,
    ) -> RpcResult<Option<RtdCoinMetadata>>;

    /// Return total supply for a coin
    #[method(name = "getTotalSupply")]
    async fn get_total_supply(
        &self,
        /// type name for the coin (e.g., 0x168da5bf1f48dafc111b0a488fa454aca95e0b5e::usdc::USDC)
        coin_type: String,
    ) -> RpcResult<Supply>;
}
