// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! This module contains the transactional test runner instantiation for the Rtd adapter

pub mod args;
pub mod cursor;
pub mod offchain_state;
pub mod programmable_transaction_test_parser;
mod simulator_persisted_store;
pub mod test_adapter;

pub use move_transactional_test_runner::framework::{
    create_adapter, run_tasks_with_adapter, run_test_impl,
};
use rand::rngs::StdRng;
use simulacrum::AdvanceEpochConfig;
use simulacrum::Simulacrum;
use simulacrum::SimulatorStore;
use simulator_persisted_store::PersistedStore;
use std::path::Path;
use std::sync::Arc;
use rtd_core::authority::AuthorityState;
use rtd_core::authority::authority_per_epoch_store::CertLockGuard;
use rtd_core::authority::authority_test_utils::send_and_confirm_transaction_with_execution_error;
use rtd_core::authority::shared_object_version_manager::AssignedVersions;
use rtd_json_rpc::authority_state::StateRead;
use rtd_json_rpc_types::EventFilter;
use rtd_json_rpc_types::{DevInspectResults, DryRunTransactionBlockResponse};
use rtd_storage::key_value_store::TransactionKeyValueStore;
use rtd_types::base_types::ObjectID;
use rtd_types::base_types::RtdAddress;
use rtd_types::base_types::VersionNumber;
use rtd_types::committee::EpochId;
use rtd_types::digests::TransactionDigest;
use rtd_types::effects::TransactionEffects;
use rtd_types::effects::TransactionEvents;
use rtd_types::error::ExecutionError;
use rtd_types::error::RtdErrorKind;
use rtd_types::error::RtdResult;
use rtd_types::event::Event;
use rtd_types::executable_transaction::{ExecutableTransaction, VerifiedExecutableTransaction};
use rtd_types::messages_checkpoint::CheckpointContentsDigest;
use rtd_types::messages_checkpoint::VerifiedCheckpoint;
use rtd_types::object::Object;
use rtd_types::storage::ObjectStore;
use rtd_types::storage::ReadStore;
use rtd_types::rtd_system_state::RtdSystemStateTrait;
use rtd_types::rtd_system_state::epoch_start_rtd_system_state::EpochStartSystemStateTrait;
use rtd_types::transaction::Transaction;
use rtd_types::transaction::TransactionKind;
use rtd_types::transaction::{InputObjects, TransactionData};
use test_adapter::{PRE_COMPILED, RtdTestAdapter};

#[cfg_attr(not(msim), tokio::main)]
#[cfg_attr(msim, msim::main)]
pub async fn run_test(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let (_guard, _filter_handle) = telemetry_subscribers::TelemetryConfig::new()
        .with_env()
        .init();
    run_test_impl::<RtdTestAdapter>(path, Some(std::sync::Arc::new(PRE_COMPILED.clone())), None)
        .await?;
    Ok(())
}

pub struct ValidatorWithFullnode {
    pub validator: Arc<AuthorityState>,
    pub fullnode: Arc<AuthorityState>,
    pub kv_store: Arc<TransactionKeyValueStore>,
}

#[allow(unused_variables)]
/// TODO: better name?
#[async_trait::async_trait]
pub trait TransactionalAdapter: Send + Sync + ReadStore {
    async fn execute_txn(
        &mut self,
        transaction: Transaction,
    ) -> anyhow::Result<(TransactionEffects, Option<ExecutionError>)>;

    async fn read_input_objects(
        &self,
        transaction: Transaction,
        assigned_versions: AssignedVersions,
    ) -> RtdResult<InputObjects>;

    fn prepare_txn(
        &self,
        transaction: Transaction,
        input_objects: InputObjects,
    ) -> anyhow::Result<(TransactionEffects, Option<ExecutionError>)>;

    async fn create_checkpoint(&mut self) -> anyhow::Result<VerifiedCheckpoint>;

    async fn advance_clock(
        &mut self,
        duration: std::time::Duration,
    ) -> anyhow::Result<TransactionEffects>;

    async fn advance_epoch(&mut self, config: AdvanceEpochConfig) -> anyhow::Result<()>;

    async fn request_gas(
        &mut self,
        address: RtdAddress,
        amount: u64,
    ) -> anyhow::Result<TransactionEffects>;

    async fn dry_run_transaction_block(
        &self,
        transaction_block: TransactionData,
        transaction_digest: TransactionDigest,
    ) -> RtdResult<DryRunTransactionBlockResponse>;

    async fn dev_inspect_transaction_block(
        &self,
        sender: RtdAddress,
        transaction_kind: TransactionKind,
        gas_price: Option<u64>,
    ) -> RtdResult<DevInspectResults>;

    async fn query_tx_events_asc(
        &self,
        tx_digest: &TransactionDigest,
        limit: usize,
    ) -> RtdResult<Vec<Event>>;

    async fn get_active_validator_addresses(&self) -> RtdResult<Vec<RtdAddress>>;

    fn get_object(&self, object_id: &ObjectID) -> Option<Object>;
}

#[async_trait::async_trait]
impl TransactionalAdapter for ValidatorWithFullnode {
    async fn execute_txn(
        &mut self,
        transaction: Transaction,
    ) -> anyhow::Result<(TransactionEffects, Option<ExecutionError>)> {
        let is_consensus_tx = transaction.is_consensus_tx();
        let (_, effects, execution_error) = send_and_confirm_transaction_with_execution_error(
            &self.validator,
            Some(&self.fullnode),
            transaction,
            is_consensus_tx,
            false,
        )
        .await?;
        Ok((effects.into_data(), execution_error))
    }

    async fn read_input_objects(
        &self,
        transaction: Transaction,
        assigned_versions: AssignedVersions,
    ) -> RtdResult<InputObjects> {
        let tx = VerifiedExecutableTransaction::new_unchecked(
            ExecutableTransaction::new_from_data_and_sig(
                transaction.data().clone(),
                rtd_types::executable_transaction::CertificateProof::Checkpoint(0, 0),
            ),
        );

        let epoch_store = self.validator.load_epoch_store_one_call_per_task().clone();
        self.validator.read_objects_for_execution(
            &CertLockGuard::dummy_for_tests(),
            &tx,
            &assigned_versions,
            &epoch_store,
        )
    }

    fn prepare_txn(
        &self,
        transaction: Transaction,
        input_objects: InputObjects,
    ) -> anyhow::Result<(TransactionEffects, Option<ExecutionError>)> {
        let tx = VerifiedExecutableTransaction::new_unchecked(
            ExecutableTransaction::new_from_data_and_sig(
                transaction.data().clone(),
                rtd_types::executable_transaction::CertificateProof::Checkpoint(0, 0),
            ),
        );

        let epoch_store = self.validator.load_epoch_store_one_call_per_task().clone();
        let (transaction_outputs, error) =
            self.validator
                .prepare_certificate_for_benchmark(&tx, input_objects, &epoch_store)?;
        Ok((transaction_outputs.effects, error))
    }

    async fn dry_run_transaction_block(
        &self,
        transaction_block: TransactionData,
        transaction_digest: TransactionDigest,
    ) -> RtdResult<DryRunTransactionBlockResponse> {
        self.fullnode
            .dry_exec_transaction(transaction_block, transaction_digest)
            .await
            .map(|result| result.0)
    }

    async fn dev_inspect_transaction_block(
        &self,
        sender: RtdAddress,
        transaction_kind: TransactionKind,
        gas_price: Option<u64>,
    ) -> RtdResult<DevInspectResults> {
        self.fullnode
            .dev_inspect_transaction_block(
                sender,
                transaction_kind,
                gas_price,
                None,
                None,
                None,
                None,
                None,
            )
            .await
    }

    async fn query_tx_events_asc(
        &self,
        tx_digest: &TransactionDigest,
        limit: usize,
    ) -> RtdResult<Vec<Event>> {
        Ok(self
            .validator
            .query_events(
                &self.kv_store,
                EventFilter::Transaction(*tx_digest),
                None,
                limit,
                false,
            )
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|rtd_event| rtd_event.into())
            .collect())
    }

    async fn create_checkpoint(&mut self) -> anyhow::Result<VerifiedCheckpoint> {
        unimplemented!("create_checkpoint not supported")
    }

    async fn advance_clock(
        &mut self,
        _duration: std::time::Duration,
    ) -> anyhow::Result<TransactionEffects> {
        unimplemented!("advance_clock not supported")
    }

    async fn advance_epoch(&mut self, _config: AdvanceEpochConfig) -> anyhow::Result<()> {
        self.validator.reconfigure_for_testing().await;
        self.fullnode.reconfigure_for_testing().await;
        Ok(())
    }

    async fn request_gas(
        &mut self,
        _address: RtdAddress,
        _amount: u64,
    ) -> anyhow::Result<TransactionEffects> {
        unimplemented!("request_gas not supported")
    }

    async fn get_active_validator_addresses(&self) -> RtdResult<Vec<RtdAddress>> {
        Ok(self
            .fullnode
            .get_system_state()
            .map_err(|e| {
                RtdErrorKind::RtdSystemStateReadError(format!(
                    "Failed to get system state from fullnode: {}",
                    e
                ))
            })?
            .into_rtd_system_state_summary()
            .active_validators
            .iter()
            .map(|x| x.rtd_address)
            .collect::<Vec<_>>())
    }

    fn get_object(&self, object_id: &ObjectID) -> Option<Object> {
        self.validator.get_object_store().get_object(object_id)
    }
}

impl ReadStore for ValidatorWithFullnode {
    fn get_committee(
        &self,
        _epoch: rtd_types::committee::EpochId,
    ) -> Option<Arc<rtd_types::committee::Committee>> {
        todo!()
    }

    fn get_latest_epoch_id(&self) -> rtd_types::storage::error::Result<EpochId> {
        Ok(self.validator.epoch_store_for_testing().epoch())
    }

    fn get_latest_checkpoint(&self) -> rtd_types::storage::error::Result<VerifiedCheckpoint> {
        let sequence_number = self
            .validator
            .get_latest_checkpoint_sequence_number()
            .unwrap();
        Ok(self
            .get_checkpoint_by_sequence_number(sequence_number)
            .unwrap())
    }

    fn get_highest_verified_checkpoint(
        &self,
    ) -> rtd_types::storage::error::Result<VerifiedCheckpoint> {
        todo!()
    }

    fn get_highest_synced_checkpoint(
        &self,
    ) -> rtd_types::storage::error::Result<VerifiedCheckpoint> {
        todo!()
    }

    fn get_lowest_available_checkpoint(
        &self,
    ) -> rtd_types::storage::error::Result<rtd_types::messages_checkpoint::CheckpointSequenceNumber>
    {
        todo!()
    }

    fn get_checkpoint_by_digest(
        &self,
        _digest: &rtd_types::messages_checkpoint::CheckpointDigest,
    ) -> Option<VerifiedCheckpoint> {
        todo!()
    }

    fn get_checkpoint_by_sequence_number(
        &self,
        sequence_number: rtd_types::messages_checkpoint::CheckpointSequenceNumber,
    ) -> Option<VerifiedCheckpoint> {
        self.validator
            .get_checkpoint_store()
            .get_checkpoint_by_sequence_number(sequence_number)
            .expect("db error")
    }

    fn get_checkpoint_contents_by_digest(
        &self,
        digest: &CheckpointContentsDigest,
    ) -> Option<rtd_types::messages_checkpoint::CheckpointContents> {
        self.validator
            .get_checkpoint_store()
            .get_checkpoint_contents(digest)
            .expect("db error")
    }

    fn get_checkpoint_contents_by_sequence_number(
        &self,
        _sequence_number: rtd_types::messages_checkpoint::CheckpointSequenceNumber,
    ) -> Option<rtd_types::messages_checkpoint::CheckpointContents> {
        todo!()
    }

    fn get_transaction(
        &self,
        tx_digest: &TransactionDigest,
    ) -> Option<Arc<rtd_types::transaction::VerifiedTransaction>> {
        self.validator
            .get_transaction_cache_reader()
            .get_transaction_block(tx_digest)
    }

    fn get_transaction_effects(&self, tx_digest: &TransactionDigest) -> Option<TransactionEffects> {
        self.validator
            .get_transaction_cache_reader()
            .get_executed_effects(tx_digest)
    }

    fn get_events(&self, digest: &TransactionDigest) -> Option<TransactionEvents> {
        self.validator
            .get_transaction_cache_reader()
            .get_events(digest)
    }

    fn get_full_checkpoint_contents(
        &self,
        _sequence_number: Option<rtd_types::messages_checkpoint::CheckpointSequenceNumber>,
        _digest: &CheckpointContentsDigest,
    ) -> Option<rtd_types::messages_checkpoint::VersionedFullCheckpointContents> {
        todo!()
    }

    fn get_unchanged_loaded_runtime_objects(
        &self,
        _digest: &TransactionDigest,
    ) -> Option<Vec<rtd_types::storage::ObjectKey>> {
        None
    }

    fn get_transaction_checkpoint(
        &self,
        _digest: &TransactionDigest,
    ) -> Option<rtd_types::messages_checkpoint::CheckpointSequenceNumber> {
        None
    }
}

impl ObjectStore for ValidatorWithFullnode {
    fn get_object(&self, object_id: &ObjectID) -> Option<Object> {
        self.validator.get_object_store().get_object(object_id)
    }

    fn get_object_by_key(&self, object_id: &ObjectID, version: VersionNumber) -> Option<Object> {
        self.validator
            .get_object_store()
            .get_object_by_key(object_id, version)
    }
}

#[async_trait::async_trait]
impl TransactionalAdapter for Simulacrum<StdRng, PersistedStore> {
    async fn execute_txn(
        &mut self,
        transaction: Transaction,
    ) -> anyhow::Result<(TransactionEffects, Option<ExecutionError>)> {
        Ok(self.execute_transaction(transaction)?)
    }

    async fn read_input_objects(
        &self,
        _transaction: Transaction,
        _assigned_versions: AssignedVersions,
    ) -> RtdResult<InputObjects> {
        unimplemented!("read_input_objects not supported in simulator mode")
    }

    fn prepare_txn(
        &self,
        _transaction: Transaction,
        _input_objects: InputObjects,
    ) -> anyhow::Result<(TransactionEffects, Option<ExecutionError>)> {
        unimplemented!("prepare_txn not supported in simulator mode")
    }

    async fn dev_inspect_transaction_block(
        &self,
        _sender: RtdAddress,
        _transaction_kind: TransactionKind,
        _gas_price: Option<u64>,
    ) -> RtdResult<DevInspectResults> {
        unimplemented!("dev_inspect_transaction_block not supported in simulator mode")
    }

    async fn dry_run_transaction_block(
        &self,
        _transaction_block: TransactionData,
        _transaction_digest: TransactionDigest,
    ) -> RtdResult<DryRunTransactionBlockResponse> {
        unimplemented!("dry_run_transaction_block not supported in simulator mode")
    }

    async fn query_tx_events_asc(
        &self,
        tx_digest: &TransactionDigest,
        _limit: usize,
    ) -> RtdResult<Vec<Event>> {
        Ok(self
            .store()
            .get_transaction_events(tx_digest)
            .map(|x| x.data)
            .unwrap_or_default())
    }

    async fn create_checkpoint(&mut self) -> anyhow::Result<VerifiedCheckpoint> {
        Ok(self.create_checkpoint())
    }

    async fn advance_clock(
        &mut self,
        duration: std::time::Duration,
    ) -> anyhow::Result<TransactionEffects> {
        Ok(self.advance_clock(duration))
    }

    async fn advance_epoch(&mut self, config: AdvanceEpochConfig) -> anyhow::Result<()> {
        self.advance_epoch(config);
        Ok(())
    }

    async fn request_gas(
        &mut self,
        address: RtdAddress,
        amount: u64,
    ) -> anyhow::Result<TransactionEffects> {
        self.request_gas(address, amount)
    }

    async fn get_active_validator_addresses(&self) -> RtdResult<Vec<RtdAddress>> {
        // TODO: this is a hack to get the validator addresses. Currently using start state
        //       but we should have a better way to get this information after reconfig
        Ok(self.epoch_start_state().get_validator_addresses())
    }

    fn get_object(&self, object_id: &ObjectID) -> Option<Object> {
        ObjectStore::get_object(&self.store(), object_id)
    }
}
