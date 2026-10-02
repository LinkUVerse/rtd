// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::checkpoint::{CheckpointsList, read_checkpoint, read_checkpoint_list};
use crate::committee::extract_new_committee_info;
use crate::config::Config;
use crate::object_store::RtdObjectStore;
use anyhow::{Result, anyhow};
use linku_common::ZipDebugEqIteratorExt;
use rtd_config::genesis::Genesis;
use rtd_rpc_api::Client;
use rtd_types::base_types::{ObjectID, TransactionDigest};
use rtd_types::committee::Committee;
use rtd_types::effects::{TransactionEffects, TransactionEvents};
use rtd_types::full_checkpoint_content::CheckpointData;
use rtd_types::messages_checkpoint::CheckpointSequenceNumber;
use rtd_types::object::Object;
use tracing::info;

use rtd_types::effects::TransactionEffectsAPI;

pub fn extract_verified_effects_and_events(
    checkpoint: &CheckpointData,
    committee: &Committee,
    tid: TransactionDigest,
) -> Result<(TransactionEffects, Option<TransactionEvents>)> {
    let summary = &checkpoint.checkpoint_summary;

    // Verify the checkpoint summary using the committee
    summary.verify_with_contents(committee, Some(&checkpoint.checkpoint_contents))?;

    // Check the validity of the transaction
    let contents = &checkpoint.checkpoint_contents;
    let (matching_tx, _) = checkpoint
        .transactions
        .iter()
        .zip_debug_eq(contents.iter())
        // Note that we get the digest of the effects to ensure this is
        // indeed the correct effects that are authenticated in the contents.
        .find(|(tx, digest)| {
            tx.effects.execution_digests() == **digest && digest.transaction == tid
        })
        .ok_or(anyhow!("Transaction not found in checkpoint contents"))?;

    // Check the events are all correct.
    let events_digest = matching_tx.events.as_ref().map(|events| events.digest());
    anyhow::ensure!(
        events_digest.as_ref() == matching_tx.effects.events_digest(),
        "Events digest does not match"
    );

    // Since we do not check objects we do not return them
    Ok((matching_tx.effects.clone(), matching_tx.events.clone()))
}

pub async fn get_verified_object(config: &Config, id: ObjectID) -> Result<Object> {
    let mut client = Client::new(config.full_node_url.as_str())?;

    info!("Getting object: {}", id);

    let object = client.get_object(id).await?;

    // Need to authenticate this object
    let (effects, _) = get_verified_effects_and_events(config, object.previous_transaction)
        .await
        .expect("Cannot get effects and events");

    // check that this object ID, version and hash is in the effects
    let target_object_ref = object.compute_object_reference();
    effects
        .all_changed_objects()
        .iter()
        .find(|object_ref| object_ref.0 == target_object_ref)
        .ok_or(anyhow!("Object not found"))
        .expect("Object not found");

    Ok(object)
}

pub async fn get_verified_effects_and_events(
    config: &Config,
    tid: TransactionDigest,
) -> Result<(TransactionEffects, Option<TransactionEvents>)> {
    let mut client = Client::new(config.full_node_url.as_str())?;

    info!("Getting effects and events for TID: {}", tid);

    // Lookup the transaction id and get the checkpoint sequence number
    let seq = client
        .get_transaction(&tid)
        .await?
        .checkpoint
        .ok_or(anyhow!("Transaction not found"))?;

    // Create object store
    let object_store = RtdObjectStore::new(config)?;

    // Download the full checkpoint for this sequence number
    let full_check_point = object_store
        .get_full_checkpoint(seq)
        .await
        .map_err(|e| anyhow!(format!("Cannot get full checkpoint: {e}")))?;

    // Load the list of stored checkpoints
    let checkpoints_list: CheckpointsList = read_checkpoint_list(config)?;

    // find the stored checkpoint before the seq checkpoint
    let prev_ckp_id = checkpoints_list
        .checkpoints
        .iter()
        .rfind(|ckp_id| **ckp_id < seq);

    let committee = if let Some(prev_ckp_id) = prev_ckp_id {
        // Read it from the store
        let prev_ckp = read_checkpoint(config, *prev_ckp_id)?;

        // Check we have the right checkpoint
        anyhow::ensure!(
            prev_ckp.epoch().checked_add(1).unwrap() == full_check_point.checkpoint_summary.epoch(),
            "Checkpoint sequence number does not match. Need to Sync."
        );

        // Get the committee from the previous checkpoint
        extract_new_committee_info(&prev_ckp)?
    } else {
        // Since we did not find a small committee checkpoint we use the genesis
        let mut genesis_path = config.checkpoint_summary_dir.clone();
        genesis_path.push(&config.genesis_filename);
        Genesis::load(&genesis_path)?.committee()
    };

    info!("Extracting effects and events for TID: {}", tid);
    extract_verified_effects_and_events(&full_check_point, &committee, tid)
        .map_err(|e| anyhow!(format!("Cannot extract effects and events: {e}")))
}

/// Get the verified checkpoint sequence number for an object.
/// This function will verify that the object is in the transaction's effects,
/// and that the transaction is in the checkpoint
/// and that the checkpoint is signed by the committee
/// and the committee is read from the verified checkpoint summary
/// which is signed by the previous committee.
pub async fn get_verified_checkpoint(
    id: ObjectID,
    config: &Config,
) -> Result<CheckpointSequenceNumber> {
    let mut client = Client::new(config.full_node_url.as_str())?;
    let object = client.get_object(id).await?;

    // Lookup the transaction id and get the checkpoint sequence number
    let seq = client
        .get_transaction(&object.previous_transaction)
        .await?
        .checkpoint
        .ok_or(anyhow!("Transaction not found"))?;

    // Need to authenticate this object
    let (effects, _) = get_verified_effects_and_events(config, object.previous_transaction)
        .await
        .expect("Cannot get effects and events");

    // check that this object ID, version and hash is in the effects
    let target_object_ref = object.compute_object_reference();
    effects
        .all_changed_objects()
        .iter()
        .find(|object_ref| object_ref.0 == target_object_ref)
        .ok_or(anyhow!("Object not found"))
        .expect("Object not found");

    // Create object store
    let object_store = RtdObjectStore::new(config)?;

    // Download the full checkpoint for this sequence number
    let full_check_point = object_store
        .get_full_checkpoint(seq)
        .await
        .map_err(|e| anyhow!(format!("Cannot get full checkpoint: {e}")))?;

    // Load the list of stored checkpoints
    let checkpoints_list: CheckpointsList = read_checkpoint_list(config)?;

    // find the stored checkpoint before the seq checkpoint
    let prev_ckp_id = checkpoints_list
        .checkpoints
        .iter()
        .rfind(|ckp_id| **ckp_id < seq);

    let committee = if let Some(prev_ckp_id) = prev_ckp_id {
        // Read it from the store
        let prev_ckp = read_checkpoint(config, *prev_ckp_id)?;

        // Check we have the right checkpoint
        anyhow::ensure!(
            prev_ckp.epoch().checked_add(1).unwrap() == full_check_point.checkpoint_summary.epoch(),
            "Checkpoint sequence number does not match. Need to Sync."
        );

        // Get the committee from the previous checkpoint
        extract_new_committee_info(&prev_ckp)?
    } else {
        // Since we did not find a small committee checkpoint we use the genesis
        let mut genesis_path = config.checkpoint_summary_dir.clone();
        genesis_path.push(&config.genesis_filename);
        Genesis::load(&genesis_path)?.committee()
    };

    // Verify that committee signed this checkpoint and checkpoint contents with digest
    full_check_point
        .checkpoint_summary
        .verify_with_contents(&committee, Some(&full_check_point.checkpoint_contents))?;

    if full_check_point
        .transactions
        .iter()
        .any(|t| *t.transaction.digest() == object.previous_transaction)
    {
        Ok(seq)
    } else {
        Err(anyhow!("Transaction not found in checkpoint"))
    }
}
