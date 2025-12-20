// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use fastcrypto::encoding::Base64;
use jsonrpsee::RpcModule;
use jsonrpsee::core::RpcResult;
use jsonrpsee::http_client::HttpClient;

use rtd_json_rpc::RtdRpcModule;
use rtd_json_rpc_api::{WriteApiClient, WriteApiServer};
use rtd_json_rpc_types::{
    DevInspectArgs, DevInspectResults, DryRunTransactionBlockResponse, RtdTransactionBlockResponse,
    RtdTransactionBlockResponseOptions,
};
use rtd_open_rpc::Module;
use rtd_types::base_types::RtdAddress;
use rtd_types::quorum_driver_types::ExecuteTransactionRequestType;
use rtd_types::rtd_serde::BigInt;

use crate::types::RtdTransactionBlockResponseWithOptions;

pub(crate) struct WriteApi {
    fullnode: HttpClient,
}

impl WriteApi {
    pub fn new(fullnode_client: HttpClient) -> Self {
        Self {
            fullnode: fullnode_client,
        }
    }
}

#[async_trait]
impl WriteApiServer for WriteApi {
    async fn execute_transaction_block(
        &self,
        tx_bytes: Base64,
        signatures: Vec<Base64>,
        options: Option<RtdTransactionBlockResponseOptions>,
        request_type: Option<ExecuteTransactionRequestType>,
    ) -> RpcResult<RtdTransactionBlockResponse> {
        let rtd_transaction_response = self
            .fullnode
            .execute_transaction_block(tx_bytes, signatures, options.clone(), request_type)
            .await
            .map_err(crate::errors::client_error_to_error_object)?;
        Ok(RtdTransactionBlockResponseWithOptions {
            response: rtd_transaction_response,
            options: options.unwrap_or_default(),
        }
        .into())
    }

    async fn dev_inspect_transaction_block(
        &self,
        sender_address: RtdAddress,
        tx_bytes: Base64,
        gas_price: Option<BigInt<u64>>,
        epoch: Option<BigInt<u64>>,
        additional_args: Option<DevInspectArgs>,
    ) -> RpcResult<DevInspectResults> {
        self.fullnode
            .dev_inspect_transaction_block(
                sender_address,
                tx_bytes,
                gas_price,
                epoch,
                additional_args,
            )
            .await
            .map_err(crate::errors::client_error_to_error_object)
    }

    async fn dry_run_transaction_block(
        &self,
        tx_bytes: Base64,
    ) -> RpcResult<DryRunTransactionBlockResponse> {
        self.fullnode
            .dry_run_transaction_block(tx_bytes)
            .await
            .map_err(crate::errors::client_error_to_error_object)
    }
}

impl RtdRpcModule for WriteApi {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }

    fn rpc_doc_module() -> Module {
        rtd_json_rpc_api::WriteApiOpenRpc::module_doc()
    }
}
