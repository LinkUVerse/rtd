// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;

use rtd_json_rpc_types::{DelegatedStake, RtdCommittee, ValidatorApys};
use rtd_open_rpc_macros::open_rpc;
use rtd_types::base_types::{ObjectID, RtdAddress};
use rtd_types::rtd_serde::BigInt;
use rtd_types::rtd_system_state::rtd_system_state_summary::RtdSystemStateSummary;

#[open_rpc(namespace = "rtdx", tag = "Governance Read API")]
#[rpc(server, client, namespace = "rtdx")]
pub trait GovernanceReadApi {
    /// Return one or more [DelegatedStake]. If a Stake was withdrawn its status will be Unstaked.
    #[method(name = "getStakesByIds")]
    async fn get_stakes_by_ids(
        &self,
        staked_rtd_ids: Vec<ObjectID>,
    ) -> RpcResult<Vec<DelegatedStake>>;

    /// Return all [DelegatedStake].
    #[method(name = "getStakes")]
    async fn get_stakes(&self, owner: RtdAddress) -> RpcResult<Vec<DelegatedStake>>;

    /// Return the committee information for the asked `epoch`.
    #[method(name = "getCommitteeInfo")]
    async fn get_committee_info(
        &self,
        /// The epoch of interest. If None, default to the latest epoch
        epoch: Option<BigInt<u64>>,
    ) -> RpcResult<RtdCommittee>;

    /// Return the latest RTD system state object on-chain.
    #[method(name = "getLatestRtdSystemState")]
    async fn get_latest_rtd_system_state(&self) -> RpcResult<RtdSystemStateSummary>;

    /// Return the reference gas price for the network
    #[method(name = "getReferenceGasPrice")]
    async fn get_reference_gas_price(&self) -> RpcResult<BigInt<u64>>;

    /// Return the validator APY
    #[method(name = "getValidatorsApy")]
    async fn get_validators_apy(&self) -> RpcResult<ValidatorApys>;
}
