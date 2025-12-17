// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Pipeline definitions for the analytics indexer.

use std::sync::Arc;

use anyhow::Result;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use sui_indexer_alt_framework::Indexer;
use sui_indexer_alt_framework::pipeline::Processor;
use sui_indexer_alt_framework::pipeline::sequential::SequentialConfig;

use crate::config::PipelineConfig;
use crate::handlers::tables::{
    CheckpointProcessor, DynamicFieldProcessor, EventProcessor, MoveCallProcessor, ObjectProcessor,
    PackageBCSProcessor, PackageProcessor, TransactionBCSProcessor, TransactionObjectsProcessor,
    TransactionProcessor, WrappedObjectProcessor,
};
use crate::handlers::{AnalyticsHandler, Row};
use crate::metrics::Metrics;
use crate::package_store::PackageCache;
use crate::schema::RowSchema;
use crate::store::AnalyticsStore;
use crate::tables::{
    CheckpointRow, DynamicFieldRow, EventRow, MoveCallRow, MovePackageRow, ObjectRow,
    PackageBCSRow, TransactionBCSRow, TransactionObjectRow, TransactionRow, WrappedObjectRow,
};

/// Register a sequential pipeline with the analytics handler.
async fn register_sequential_pipeline<P, T>(
    indexer: &mut Indexer<AnalyticsStore>,
    processor: P,
    sequential_config: SequentialConfig,
) -> Result<()>
where
    P: Processor<Value = T> + Send + Sync,
    T: Row + 'static,
{
    let handler = AnalyticsHandler::new(processor);
    indexer
        .sequential_pipeline(handler, sequential_config)
        .await?;
    Ok(())
}

/// Available analytics pipelines.
#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    strum_macros::Display,
    Serialize,
    Deserialize,
    TryFromPrimitive,
    IntoPrimitive,
    EnumIter,
)]
#[repr(u8)]
pub enum Pipeline {
    Checkpoint = 0,
    Object,
    Transaction,
    TransactionBCS,
    TransactionObjects,
    Event,
    MoveCall,
    MovePackage,
    MovePackageBCS,
    DynamicField,
    WrappedObject,
}

impl Pipeline {
    /// Returns the pipeline name used for both watermarks and output directory.
    /// This must match the corresponding `Processor::NAME` constant.
    pub fn name(&self) -> &'static str {
        match self {
            Pipeline::Checkpoint => "checkpoints",
            Pipeline::Transaction => "transactions",
            Pipeline::TransactionBCS => "transaction_bcs",
            Pipeline::TransactionObjects => "transaction_objects",
            Pipeline::Object => "objects",
            Pipeline::Event => "events",
            Pipeline::MoveCall => "move_call",
            Pipeline::MovePackage => "move_package",
            Pipeline::MovePackageBCS => "move_package_bcs",
            Pipeline::DynamicField => "dynamic_field",
            Pipeline::WrappedObject => "wrapped_object",
        }
    }

    /// Returns the schema (column names) for this pipeline's row type.
    pub fn schema(&self) -> Vec<String> {
        match self {
            Pipeline::Checkpoint => CheckpointRow::schema(),
            Pipeline::Transaction => TransactionRow::schema(),
            Pipeline::TransactionBCS => TransactionBCSRow::schema(),
            Pipeline::TransactionObjects => TransactionObjectRow::schema(),
            Pipeline::Object => ObjectRow::schema(),
            Pipeline::Event => EventRow::schema(),
            Pipeline::MoveCall => MoveCallRow::schema(),
            Pipeline::MovePackage => MovePackageRow::schema(),
            Pipeline::MovePackageBCS => PackageBCSRow::schema(),
            Pipeline::DynamicField => DynamicFieldRow::schema(),
            Pipeline::WrappedObject => WrappedObjectRow::schema(),
        }
    }

    /// Registers this pipeline with the indexer.
    pub async fn register(
        &self,
        indexer: &mut Indexer<AnalyticsStore>,
        pipeline_config: &PipelineConfig,
        package_cache: Arc<PackageCache>,
        metrics: Metrics,
        sequential_config: SequentialConfig,
    ) -> Result<()> {
        match self {
            Pipeline::Checkpoint => {
                register_sequential_pipeline(indexer, CheckpointProcessor, sequential_config).await
            }
            Pipeline::Transaction => {
                register_sequential_pipeline(indexer, TransactionProcessor, sequential_config).await
            }
            Pipeline::TransactionBCS => {
                register_sequential_pipeline(indexer, TransactionBCSProcessor, sequential_config)
                    .await
            }
            Pipeline::Event => {
                register_sequential_pipeline(
                    indexer,
                    EventProcessor::new(package_cache.clone()),
                    sequential_config,
                )
                .await
            }
            Pipeline::MoveCall => {
                register_sequential_pipeline(indexer, MoveCallProcessor, sequential_config).await
            }
            Pipeline::Object => {
                register_sequential_pipeline(
                    indexer,
                    ObjectProcessor::new(
                        package_cache.clone(),
                        &pipeline_config.package_id_filter,
                        metrics,
                    ),
                    sequential_config,
                )
                .await
            }
            Pipeline::DynamicField => {
                register_sequential_pipeline(
                    indexer,
                    DynamicFieldProcessor::new(package_cache.clone()),
                    sequential_config,
                )
                .await
            }
            Pipeline::TransactionObjects => {
                register_sequential_pipeline(
                    indexer,
                    TransactionObjectsProcessor,
                    sequential_config,
                )
                .await
            }
            Pipeline::MovePackage => {
                register_sequential_pipeline(indexer, PackageProcessor, sequential_config).await
            }
            Pipeline::MovePackageBCS => {
                register_sequential_pipeline(indexer, PackageBCSProcessor, sequential_config).await
            }
            Pipeline::WrappedObject => {
                register_sequential_pipeline(
                    indexer,
                    WrappedObjectProcessor::new(package_cache.clone()),
                    sequential_config,
                )
                .await
            }
        }
    }
}
