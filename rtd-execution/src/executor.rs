// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use move_trace_format::format::MoveTraceBuilder;
use std::sync::Arc;
use rtd_protocol_config::ProtocolConfig;
use rtd_types::execution::ExecutionTiming;
use rtd_types::execution_params::ExecutionOrEarlyError;
use rtd_types::storage::BackingStore;
use rtd_types::transaction::GasData;
use rtd_types::{
    base_types::RtdAddress,
    committee::EpochId,
    digests::TransactionDigest,
    effects::TransactionEffects,
    error::ExecutionError,
    execution::{ExecutionResult, TypeLayoutStore},
    gas::RtdGasStatus,
    inner_temporary_store::InnerTemporaryStore,
    layout_resolver::LayoutResolver,
    metrics::LimitsMetrics,
    transaction::{CheckedInputObjects, ProgrammableTransaction, TransactionKind},
};

/// Abstracts over access to the VM across versions of the execution layer.
pub trait Executor {
    fn execute_transaction_to_effects(
        &self,
        store: &dyn BackingStore,
        // Configuration
        protocol_config: &ProtocolConfig,
        metrics: Arc<LimitsMetrics>,
        enable_expensive_checks: bool,
        execution_params: ExecutionOrEarlyError,
        // Epoch
        epoch_id: &EpochId,
        epoch_timestamp_ms: u64,
        // Transaction Inputs
        input_objects: CheckedInputObjects,
        // Gas related
        gas: GasData,
        gas_status: RtdGasStatus,
        // Transaction
        transaction_kind: TransactionKind,
        transaction_signer: RtdAddress,
        transaction_digest: TransactionDigest,
        trace_builder_opt: &mut Option<MoveTraceBuilder>,
    ) -> (
        InnerTemporaryStore,
        RtdGasStatus,
        TransactionEffects,
        Vec<ExecutionTiming>,
        Result<(), ExecutionError>,
    );

    fn dev_inspect_transaction(
        &self,
        store: &dyn BackingStore,
        // Configuration
        protocol_config: &ProtocolConfig,
        metrics: Arc<LimitsMetrics>,
        enable_expensive_checks: bool,
        execution_params: ExecutionOrEarlyError,
        // Epoch
        epoch_id: &EpochId,
        epoch_timestamp_ms: u64,
        // Transaction Inputs
        input_objects: CheckedInputObjects,
        // Gas related
        gas: GasData,
        gas_status: RtdGasStatus,
        // Transaction
        transaction_kind: TransactionKind,
        transaction_signer: RtdAddress,
        transaction_digest: TransactionDigest,
        skip_all_checks: bool,
    ) -> (
        InnerTemporaryStore,
        RtdGasStatus,
        TransactionEffects,
        Result<Vec<ExecutionResult>, ExecutionError>,
    );

    fn update_genesis_state(
        &self,
        store: &dyn BackingStore,
        // Configuration
        protocol_config: &ProtocolConfig,
        metrics: Arc<LimitsMetrics>,
        // Epoch
        epoch_id: EpochId,
        epoch_timestamp_ms: u64,
        // Genesis Digest
        transaction_digest: &TransactionDigest,
        // Transaction
        input_objects: CheckedInputObjects,
        pt: ProgrammableTransaction,
    ) -> Result<InnerTemporaryStore, ExecutionError>;

    fn type_layout_resolver<'r, 'vm: 'r, 'store: 'r>(
        &'vm self,
        store: Box<dyn TypeLayoutStore + 'store>,
    ) -> Box<dyn LayoutResolver + 'r>;
}
