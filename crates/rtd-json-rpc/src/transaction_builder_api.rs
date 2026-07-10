// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use async_trait::async_trait;
use fastcrypto::encoding::Base64;
use jsonrpsee::RpcModule;
use jsonrpsee::core::RpcResult;
use move_core_types::language_storage::StructTag;

use rtd_core::authority::AuthorityState;
use rtd_core::node_readiness::FullnodeReadiness;
use rtd_json::RtdJsonValue;
use rtd_json_rpc_api::{TransactionBuilderOpenRpc, TransactionBuilderServer};
use rtd_json_rpc_types::{RPCTransactionRequestParams, RtdObjectDataFilter};
use rtd_json_rpc_types::{
    RtdObjectDataOptions, RtdObjectResponse, RtdTransactionBlockBuilderMode, RtdTypeTag,
    TransactionBlockBytes,
};
use rtd_open_rpc::Module;
use rtd_transaction_builder::{DataReader, TransactionBuilder};
use rtd_types::base_types::ObjectInfo;
use rtd_types::base_types::{ObjectID, RtdAddress};
use rtd_types::quorum_driver_types::QuorumDriverError;
use rtd_types::rtd_serde::BigInt;

use crate::RtdRpcModule;
use crate::authority_state::StateRead;

pub struct TransactionBuilderApi(TransactionBuilder, Option<Arc<FullnodeReadiness>>);

impl TransactionBuilderApi {
    pub fn new(state: Arc<AuthorityState>) -> Self {
        let reader = Arc::new(AuthorityStateDataReader::new(state));
        Self(TransactionBuilder::new(reader), None)
    }

    pub fn new_with_readiness(
        state: Arc<AuthorityState>,
        readiness: Arc<FullnodeReadiness>,
    ) -> Self {
        let reader = Arc::new(AuthorityStateDataReader::new(state));
        Self(TransactionBuilder::new(reader), Some(readiness))
    }

    pub fn new_with_data_reader(data_reader: Arc<dyn DataReader + Sync + Send>) -> Self {
        Self(TransactionBuilder::new(data_reader), None)
    }

    pub fn new_with_data_reader_and_readiness(
        data_reader: Arc<dyn DataReader + Sync + Send>,
        readiness: Arc<FullnodeReadiness>,
    ) -> Self {
        Self(TransactionBuilder::new(data_reader), Some(readiness))
    }

    fn ensure_ready(&self) -> RpcResult<()> {
        if let Some(readiness) = &self.1 {
            readiness
                .ensure_ready()
                .map_err(|error| -> jsonrpsee::types::ErrorObjectOwned {
                    crate::Error::QuorumDriverError(QuorumDriverError::FullnodeCatchingUp {
                        details: error.to_string(),
                    })
                    .into()
                })?;
        }
        Ok(())
    }
}

pub struct AuthorityStateDataReader(Arc<dyn StateRead>);

impl AuthorityStateDataReader {
    pub fn new(state: Arc<AuthorityState>) -> Self {
        Self(state)
    }
}

#[async_trait]
impl DataReader for AuthorityStateDataReader {
    async fn get_owned_objects(
        &self,
        address: RtdAddress,
        object_type: StructTag,
    ) -> Result<Vec<ObjectInfo>, anyhow::Error> {
        Ok(self
            .0
            // DataReader is used internally, don't need a limit
            .get_owner_objects(
                address,
                None,
                Some(RtdObjectDataFilter::StructType(object_type)),
            )?)
    }

    async fn get_object_with_options(
        &self,
        object_id: ObjectID,
        options: RtdObjectDataOptions,
    ) -> Result<RtdObjectResponse, anyhow::Error> {
        let result = self.0.get_object_read(&object_id)?;
        Ok((result, options).try_into()?)
    }

    async fn get_reference_gas_price(&self) -> Result<u64, anyhow::Error> {
        let epoch_store = self.0.load_epoch_store_one_call_per_task();
        Ok(epoch_store.reference_gas_price())
    }
}

#[async_trait]
impl TransactionBuilderServer for TransactionBuilderApi {
    async fn transfer_object(
        &self,
        signer: RtdAddress,
        object_id: ObjectID,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
        recipient: RtdAddress,
    ) -> RpcResult<TransactionBlockBytes> {
        self.ensure_ready()?;
        let data = self
            .0
            .transfer_object(signer, object_id, gas, *gas_budget, recipient)
            .await
            .map_err(crate::Error::from)?;
        Ok(TransactionBlockBytes::from_data(data).map_err(crate::Error::from)?)
    }

    async fn transfer_rtd(
        &self,
        signer: RtdAddress,
        rtd_object_id: ObjectID,
        gas_budget: BigInt<u64>,
        recipient: RtdAddress,
        amount: Option<BigInt<u64>>,
    ) -> RpcResult<TransactionBlockBytes> {
        self.ensure_ready()?;
        let data = self
            .0
            .transfer_rtd(
                signer,
                rtd_object_id,
                *gas_budget,
                recipient,
                amount.map(|a| *a),
            )
            .await
            .map_err(crate::Error::from)?;
        Ok(TransactionBlockBytes::from_data(data).map_err(crate::Error::from)?)
    }

    async fn pay(
        &self,
        signer: RtdAddress,
        input_coins: Vec<ObjectID>,
        recipients: Vec<RtdAddress>,
        amounts: Vec<BigInt<u64>>,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        self.ensure_ready()?;
        let data = self
            .0
            .pay(
                signer,
                input_coins,
                recipients,
                amounts.into_iter().map(|a| *a).collect(),
                gas,
                *gas_budget,
            )
            .await
            .map_err(crate::Error::from)?;
        Ok(TransactionBlockBytes::from_data(data).map_err(crate::Error::from)?)
    }

    async fn pay_rtd(
        &self,
        signer: RtdAddress,
        input_coins: Vec<ObjectID>,
        recipients: Vec<RtdAddress>,
        amounts: Vec<BigInt<u64>>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        self.ensure_ready()?;
        let data = self
            .0
            .pay_rtd(
                signer,
                input_coins,
                recipients,
                amounts.into_iter().map(|a| *a).collect(),
                *gas_budget,
            )
            .await
            .map_err(crate::Error::from)?;
        Ok(TransactionBlockBytes::from_data(data).map_err(crate::Error::from)?)
    }

    async fn pay_all_rtd(
        &self,
        signer: RtdAddress,
        input_coins: Vec<ObjectID>,
        recipient: RtdAddress,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        self.ensure_ready()?;
        let data = self
            .0
            .pay_all_rtd(signer, input_coins, recipient, *gas_budget)
            .await
            .map_err(crate::Error::from)?;
        Ok(TransactionBlockBytes::from_data(data).map_err(crate::Error::from)?)
    }

    async fn publish(
        &self,
        sender: RtdAddress,
        compiled_modules: Vec<Base64>,
        dependencies: Vec<ObjectID>,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        self.ensure_ready()?;
        let compiled_modules = compiled_modules
            .into_iter()
            .map(|data| data.to_vec().map_err(|e| anyhow::anyhow!(e)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(crate::Error::from)?;
        let data = self
            .0
            .publish(sender, compiled_modules, dependencies, gas, *gas_budget)
            .await
            .map_err(crate::Error::from)?;
        Ok(TransactionBlockBytes::from_data(data).map_err(crate::Error::from)?)
    }

    async fn split_coin(
        &self,
        signer: RtdAddress,
        coin_object_id: ObjectID,
        split_amounts: Vec<BigInt<u64>>,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        self.ensure_ready()?;
        let split_amounts = split_amounts.into_iter().map(|a| *a).collect();
        let data = self
            .0
            .split_coin(signer, coin_object_id, split_amounts, gas, *gas_budget)
            .await
            .map_err(crate::Error::from)?;
        Ok(TransactionBlockBytes::from_data(data).map_err(crate::Error::from)?)
    }

    async fn split_coin_equal(
        &self,
        signer: RtdAddress,
        coin_object_id: ObjectID,
        split_count: BigInt<u64>,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        self.ensure_ready()?;
        let data = self
            .0
            .split_coin_equal(signer, coin_object_id, *split_count, gas, *gas_budget)
            .await
            .map_err(crate::Error::from)?;
        Ok(TransactionBlockBytes::from_data(data).map_err(crate::Error::from)?)
    }

    async fn merge_coin(
        &self,
        signer: RtdAddress,
        primary_coin: ObjectID,
        coin_to_merge: ObjectID,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        self.ensure_ready()?;
        let data = self
            .0
            .merge_coins(signer, primary_coin, coin_to_merge, gas, *gas_budget)
            .await
            .map_err(crate::Error::from)?;
        Ok(TransactionBlockBytes::from_data(data).map_err(crate::Error::from)?)
    }

    async fn move_call(
        &self,
        signer: RtdAddress,
        package_object_id: ObjectID,
        module: String,
        function: String,
        type_arguments: Vec<RtdTypeTag>,
        rpc_arguments: Vec<RtdJsonValue>,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
        _txn_builder_mode: Option<RtdTransactionBlockBuilderMode>,
    ) -> RpcResult<TransactionBlockBytes> {
        self.ensure_ready()?;
        Ok(TransactionBlockBytes::from_data(
            self.0
                .move_call(
                    signer,
                    package_object_id,
                    &module,
                    &function,
                    type_arguments,
                    rpc_arguments,
                    gas,
                    *gas_budget,
                    None,
                )
                .await
                .map_err(crate::Error::from)?,
        )
        .map_err(crate::Error::from)?)
    }

    async fn batch_transaction(
        &self,
        signer: RtdAddress,
        params: Vec<RPCTransactionRequestParams>,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
        _txn_builder_mode: Option<RtdTransactionBlockBuilderMode>,
    ) -> RpcResult<TransactionBlockBytes> {
        self.ensure_ready()?;
        Ok(TransactionBlockBytes::from_data(
            self.0
                .batch_transaction(signer, params, gas, *gas_budget)
                .await
                .map_err(crate::Error::from)?,
        )
        .map_err(crate::Error::from)?)
    }

    async fn request_add_stake(
        &self,
        signer: RtdAddress,
        coins: Vec<ObjectID>,
        amount: Option<BigInt<u64>>,
        validator: RtdAddress,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        self.ensure_ready()?;
        let amount = amount.map(|a| *a);
        Ok(TransactionBlockBytes::from_data(
            self.0
                .request_add_stake(signer, coins, amount, validator, gas, *gas_budget)
                .await
                .map_err(crate::Error::from)?,
        )
        .map_err(crate::Error::from)?)
    }

    async fn request_withdraw_stake(
        &self,
        signer: RtdAddress,
        staked_rtd: ObjectID,
        gas: Option<ObjectID>,
        gas_budget: BigInt<u64>,
    ) -> RpcResult<TransactionBlockBytes> {
        self.ensure_ready()?;
        Ok(TransactionBlockBytes::from_data(
            self.0
                .request_withdraw_stake(signer, staked_rtd, gas, *gas_budget)
                .await
                .map_err(crate::Error::from)?,
        )
        .map_err(crate::Error::from)?)
    }
}

impl RtdRpcModule for TransactionBuilderApi {
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }

    fn rpc_doc_module() -> Module {
        TransactionBuilderOpenRpc::module_doc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rtd_core::checkpoints::CheckpointStore;
    use rtd_core::node_readiness::FullnodeReadiness;
    use rtd_json_rpc_api::TRANSIENT_ERROR_CODE;

    struct PanicDataReader;

    #[async_trait]
    impl DataReader for PanicDataReader {
        async fn get_owned_objects(
            &self,
            _address: RtdAddress,
            _object_type: StructTag,
        ) -> Result<Vec<ObjectInfo>, anyhow::Error> {
            panic!("data reader must not be called while fullnode is catching up")
        }

        async fn get_object_with_options(
            &self,
            _object_id: ObjectID,
            _options: RtdObjectDataOptions,
        ) -> Result<RtdObjectResponse, anyhow::Error> {
            panic!("data reader must not be called while fullnode is catching up")
        }

        async fn get_reference_gas_price(&self) -> Result<u64, anyhow::Error> {
            panic!("data reader must not be called while fullnode is catching up")
        }
    }

    #[tokio::test]
    async fn transaction_builder_rejects_catching_up_before_reading_objects() {
        let readiness = Arc::new(FullnodeReadiness::new(
            42,
            CheckpointStore::new_for_tests(),
            None,
            false,
            false,
        ));
        let api = TransactionBuilderApi::new_with_data_reader_and_readiness(
            Arc::new(PanicDataReader),
            readiness,
        );

        let result = api
            .transfer_rtd(
                RtdAddress::ZERO,
                ObjectID::ZERO,
                BigInt::from(1),
                RtdAddress::ZERO,
                None,
            )
            .await;
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("transaction builder unexpectedly succeeded"),
        };

        assert_eq!(error.code(), TRANSIENT_ERROR_CODE);
    }
}
