// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use anyhow::Context as _;
use anyhow::ensure;
use async_graphql::Context;
use async_graphql::Enum;
use async_graphql::SimpleObject;
use async_graphql::Union;
use async_graphql::dataloader::DataLoader;
use fastcrypto::encoding::Base58;
use fastcrypto::encoding::Encoding;
use fastcrypto::encoding::Hex;
use fastcrypto::hash::HashFunction;
use fastcrypto::hash::Sha256;
use move_core_types::ident_str;
use move_core_types::language_storage::StructTag;
use rtd_indexer_alt_reader::historical_displays::DisplayHistoryCoverageKey;
use rtd_indexer_alt_reader::historical_displays::HistoricalDisplay;
use rtd_indexer_alt_reader::historical_displays::HistoricalDisplayKey;
use rtd_indexer_alt_reader::object_versions::VersionedObjectVersionKey;
use rtd_indexer_alt_reader::pg_reader::PgReader;
use rtd_indexer_alt_schema::display_history::StoredDisplayCoverage;
use rtd_types::RTD_FRAMEWORK_ADDRESS;
use rtd_types::base_types::ObjectID;
use rtd_types::display::DisplayVersionUpdatedEvent;
use rtd_types::display_registry;
use tokio::join;

use crate::api::scalars::json::Json;
use crate::api::scalars::rtd_address::RtdAddress;
use crate::api::scalars::uint53::UInt53;
use crate::api::types::object::Object;
use crate::error::RpcError;
use crate::scope::Scope;

#[cfg(test)]
mod tests;

const MAX_UINT53: u64 = (1 << 53) - 1;

/// A rendered JSON blob based on an on-chain template.
#[derive(SimpleObject)]
pub(crate) struct Display {
    /// Output for successfully substituted fields. Failed fields are null and described in errors.
    pub(crate) output: Option<Json>,
    /// Mapping from failed field names to error messages, or null if all fields succeeded.
    pub(crate) errors: Option<Json>,
    /// The checkpoint-bound template used to render this value.
    pub(crate) basis: DisplayBasis,
}

/// Identity and continuous coverage of the immutable v1 publication history.
/// These fields describe the indexer's certified source binding; they are not a portable proof.
#[derive(Clone, SimpleObject)]
pub(crate) struct DisplayHistoryCoverage {
    pub(crate) genesis_digest: String,
    pub(crate) source_id: String,
    pub(crate) generation: String,
    pub(crate) first_checkpoint: UInt53,
    pub(crate) last_checkpoint: UInt53,
    pub(crate) last_checkpoint_digest: String,
    pub(crate) snapshot_checkpoint: UInt53,
    pub(crate) snapshot_checkpoint_digest: String,
}

#[derive(Clone, Copy, Eq, PartialEq, Enum)]
pub(crate) enum DisplayTemplateKind {
    V1,
    V2,
}

/// Snapshot-bound capability. Missing continuous history is an error, never a supported empty result.
#[derive(SimpleObject)]
pub(crate) struct DisplayHistoryCapability {
    pub(crate) api_version: UInt53,
    pub(crate) supported_template_kinds: Vec<DisplayTemplateKind>,
    pub(crate) coverage: DisplayHistoryCoverage,
}

/// V1 event publication or V2 registry object identity. The versions have distinct semantics.
#[derive(Union)]
pub(crate) enum DisplayBasis {
    V1(DisplayV1Basis),
    V2(DisplayV2Basis),
}

#[derive(SimpleObject)]
pub(crate) struct DisplayV1Basis {
    pub(crate) snapshot_checkpoint: UInt53,
    /// Canonical type of the value whose Display was requested.
    pub(crate) object_type: String,
    pub(crate) template_id: RtdAddress,
    /// The published u16 version from VersionUpdated, not a Move object version.
    pub(crate) published_version: UInt53,
    pub(crate) source_checkpoint: UInt53,
    pub(crate) source_checkpoint_digest: String,
    pub(crate) transaction_digest: String,
    pub(crate) checkpoint_transaction_index: UInt53,
    pub(crate) event_index: UInt53,
    /// Lowercase hexadecimal SHA256 of the raw event BCS. Content identity only, not a chain digest.
    pub(crate) event_bcs_sha256: String,
    pub(crate) history_coverage: DisplayHistoryCoverage,
}

#[derive(SimpleObject)]
pub(crate) struct DisplayV2Basis {
    pub(crate) snapshot_checkpoint: UInt53,
    pub(crate) object_type: String,
    pub(crate) template_id: RtdAddress,
    /// Registry Move object version, not a v1 publication version.
    pub(crate) object_version: UInt53,
    pub(crate) object_digest: String,
    pub(crate) source_checkpoint: UInt53,
    pub(crate) previous_transaction_digest: String,
    /// Lowercase hexadecimal SHA256 of registry Move contents BCS. Content identity only.
    pub(crate) contents_bcs_sha256: String,
    pub(crate) history_coverage: DisplayHistoryCoverage,
}

pub(crate) enum LoadedDisplay {
    V1(DisplayVersionUpdatedEvent, DisplayV1Basis),
    V2(display_registry::Display, DisplayV2Basis),
}

struct V2Template {
    display: display_registry::Display,
    object_version: u64,
    object_digest: String,
    source_checkpoint: u64,
    previous_transaction_digest: String,
    contents_bcs_sha256: String,
}

impl DisplayHistoryCoverage {
    fn checked(
        stored: StoredDisplayCoverage,
        checkpoint: u64,
        snapshot_digest: Vec<u8>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            genesis_digest: checked_digest(stored.genesis_digest.as_slice())?,
            source_id: stored.source_id,
            generation: stored.generation,
            first_checkpoint: checked_uint53(u64::try_from(stored.reader_lo)?)?,
            last_checkpoint: checked_uint53(u64::try_from(stored.checkpoint_hi)?)?,
            last_checkpoint_digest: checked_digest(
                stored
                    .checkpoint_digest
                    .context("Display history endpoint is unavailable")?
                    .as_slice(),
            )?,
            snapshot_checkpoint: checked_uint53(checkpoint)?,
            snapshot_checkpoint_digest: checked_digest(&snapshot_digest)?,
        })
    }
}

/// History is required even when v2 wins: an incomplete index cannot claim a complete asset view.
/// A fresh execution/stream scope has no indexed snapshot and must not borrow a current template.
pub(crate) async fn load_display(
    ctx: &Context<'_>,
    scope: Scope,
    type_: StructTag,
) -> Result<Option<LoadedDisplay>, RpcError> {
    let checkpoint = snapshot_checkpoint(&scope)?;
    let pg_loader: &Arc<DataLoader<PgReader>> = ctx.data()?;
    let (history, v2) = join!(
        pg_loader.load_one(HistoricalDisplayKey(type_.clone(), checkpoint)),
        display_v2(ctx, scope, type_.clone())
    );
    let history = history
        .context("Failed to fetch historical Display v1")?
        .context("Display history loader omitted the requested key")?;
    Ok(select_template(type_, checkpoint, history, v2?)?)
}

fn snapshot_checkpoint(scope: &Scope) -> anyhow::Result<u64> {
    let checkpoint = scope
        .checkpoint_viewed_at()
        .context("Display requires an indexed checkpoint scope")?;
    checked_uint53(checkpoint)?;
    Ok(checkpoint)
}

fn checked_uint53(value: u64) -> anyhow::Result<UInt53> {
    ensure!(value <= MAX_UINT53, "Display basis integer exceeds UInt53");
    Ok(value.into())
}

fn checked_digest(bytes: &[u8]) -> anyhow::Result<String> {
    ensure!(
        bytes.len() == 32,
        "Display basis digest must contain 32 bytes"
    );
    Ok(Base58::encode(bytes))
}

fn select_template(
    type_: StructTag,
    checkpoint: u64,
    history: HistoricalDisplay,
    v2: Option<V2Template>,
) -> anyhow::Result<Option<LoadedDisplay>> {
    ensure!(
        history.coverage.reader_lo == 0
            && history.coverage.checkpoint_hi >= i64::try_from(checkpoint)?,
        "Display history does not cover the snapshot"
    );
    let coverage = DisplayHistoryCoverage::checked(
        history.coverage,
        checkpoint,
        history.snapshot_checkpoint_digest,
    )?;
    let snapshot_checkpoint = checked_uint53(checkpoint)?;
    let object_type = type_.to_canonical_string(true);
    if let Some(v2) = v2 {
        ensure!(
            v2.source_checkpoint <= checkpoint,
            "Display v2 source is in the future"
        );
        let basis = DisplayV2Basis {
            snapshot_checkpoint,
            object_type,
            template_id: v2.display.id.into(),
            object_version: checked_uint53(v2.object_version)?,
            object_digest: v2.object_digest,
            source_checkpoint: checked_uint53(v2.source_checkpoint)?,
            previous_transaction_digest: v2.previous_transaction_digest,
            contents_bcs_sha256: v2.contents_bcs_sha256,
            history_coverage: coverage,
        };
        return Ok(Some(LoadedDisplay::V2(v2.display, basis)));
    }
    let Some(stored) = history.published else {
        return Ok(None);
    };
    let event: DisplayVersionUpdatedEvent = bcs::from_bytes(&stored.display)
        .context("Failed to deserialize historical DisplayVersionUpdatedEvent")?;
    let source_checkpoint = u64::try_from(stored.cp_sequence_number)?;
    ensure!(
        source_checkpoint <= checkpoint
            && stored.object_type == bcs::to_bytes(&type_)?
            && event.id.bytes.as_ref() == stored.display_id.as_slice()
            && i32::from(event.version) == stored.display_version,
        "Display v1 publication basis differs"
    );
    let basis = DisplayV1Basis {
        snapshot_checkpoint,
        object_type,
        template_id: ObjectID::from_bytes(&stored.display_id)?.into(),
        published_version: u64::from(event.version).into(),
        source_checkpoint: checked_uint53(source_checkpoint)?,
        source_checkpoint_digest: checked_digest(
            history
                .event_checkpoint_digest
                .context("Display publication checkpoint digest is unavailable")?
                .as_slice(),
        )?,
        transaction_digest: checked_digest(stored.transaction_digest.as_slice())?,
        checkpoint_transaction_index: checked_uint53(u64::try_from(stored.checkpoint_tx_index)?)?,
        event_index: checked_uint53(u64::try_from(stored.event_index)?)?,
        event_bcs_sha256: Hex::encode(Sha256::digest(&stored.display)),
        history_coverage: coverage,
    };
    Ok(Some(LoadedDisplay::V1(event, basis)))
}

pub(crate) async fn display_history_coverage(
    ctx: &Context<'_>,
    scope: &Scope,
) -> Result<DisplayHistoryCoverage, RpcError> {
    let checkpoint = snapshot_checkpoint(scope)?;
    let pg_loader: &Arc<DataLoader<PgReader>> = ctx.data()?;
    let value = pg_loader
        .load_one(DisplayHistoryCoverageKey(checkpoint))
        .await
        .context("Failed to fetch continuous Display history coverage")?
        .context("Display history coverage loader omitted the checkpoint")?;
    Ok(DisplayHistoryCoverage::checked(
        value.coverage,
        checkpoint,
        value.snapshot_checkpoint_digest,
    )?)
}

/// Global registry objects ignore inherited root bounds but retain the same checkpoint S.
async fn display_v2(
    ctx: &Context<'_>,
    scope: Scope,
    type_: StructTag,
) -> Result<Option<V2Template>, RpcError> {
    let checkpoint = snapshot_checkpoint(&scope)?;
    let object_id = display_registry::display_object_id(type_.clone().into())
        .context("Failed to derive Display V2 object ID")?;
    let Some(object) = Object::latest(ctx, scope.without_root_bound(), object_id.into()).await?
    else {
        return Ok(None);
    };
    let native = object
        .contents(ctx)
        .await?
        .context("Display V2 registry object contents are unavailable")?;
    let move_object = native
        .data
        .try_as_move()
        .context("Display V2 object is not a Move object")?;
    let display: display_registry::Display = bcs::from_bytes(move_object.contents())
        .context("Failed to deserialize Display V2 object")?;
    let pg_loader: &Arc<DataLoader<PgReader>> = ctx.data()?;
    let stored = pg_loader
        .load_one(VersionedObjectVersionKey(
            object_id,
            native.version().value(),
        ))
        .await
        .context("Failed to fetch Display V2 source checkpoint")?
        .context("Display V2 object version is unavailable in the index")?;
    let source_checkpoint = u64::try_from(stored.cp_sequence_number)
        .context("Display V2 source checkpoint is invalid")?;
    let native_digest = native.digest();
    let registry_type = StructTag {
        address: RTD_FRAMEWORK_ADDRESS,
        module: ident_str!("display_registry").to_owned(),
        name: ident_str!("Display").to_owned(),
        type_params: vec![type_.into()],
    };
    let native_type: StructTag = move_object.type_().clone().into();
    let identity_matches = native.id() == object_id
        && display.id == object_id
        && native_type == registry_type
        && source_checkpoint <= checkpoint
        && stored.object_id == object_id.as_ref()
        && u64::try_from(stored.object_version).ok() == Some(native.version().value())
        && stored.object_digest.as_deref() == Some(native_digest.as_ref());
    if !identity_matches {
        return Err(anyhow::anyhow!("Display V2 registry identity or checkpoint differs").into());
    }
    Ok(Some(V2Template {
        display,
        object_version: native.version().value(),
        object_digest: checked_digest(native_digest.as_ref())
            .context("Display V2 object digest is invalid")?,
        source_checkpoint,
        previous_transaction_digest: checked_digest(native.previous_transaction.as_ref())
            .context("Display V2 previous transaction digest is invalid")?,
        contents_bcs_sha256: Hex::encode(Sha256::digest(move_object.contents())),
    }))
}
