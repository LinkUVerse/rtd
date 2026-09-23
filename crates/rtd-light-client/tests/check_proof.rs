// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

mod common;

use common::test_chain;

use rtd_light_client::proof::{
    base::{Proof, ProofBuilder, ProofContents, ProofTarget, ProofVerifier},
    committee::{CommitteeProof, extract_new_committee_info},
    objects::ObjectsTarget,
};

use rtd_types::event::{Event, EventID};

use rtd_types::{committee::Committee, effects::TransactionEffectsAPI, object::Object};

use rtd_types::full_checkpoint_content::CheckpointData;

fn read_data() -> (Committee, CheckpointData) {
    let chain = test_chain();
    (chain.committee.clone(), chain.end_of_epoch.clone())
}

fn read_transaction_data() -> (Committee, CheckpointData) {
    let chain = test_chain();
    (chain.committee.clone(), chain.transaction.clone())
}

fn sample_event(checkpoint: &CheckpointData) -> (EventID, Event) {
    let tx = checkpoint
        .transactions
        .iter()
        .find(|tx| {
            tx.events
                .as_ref()
                .is_some_and(|events| !events.data.is_empty())
        })
        .expect("RTD epoch change emits an event");
    (
        EventID::from((*tx.effects.transaction_digest(), 0)),
        tx.events.as_ref().unwrap().data[0].clone(),
    )
}

#[tokio::test]
async fn check_can_read_test_data() {
    let (_committee, full_checkpoint) = read_data();
    assert!(
        full_checkpoint
            .checkpoint_summary
            .end_of_epoch_data
            .is_some()
    );
}

#[tokio::test]
async fn test_new_committee() {
    let (committee, full_checkpoint) = read_data();

    // Make a committee object using this
    let new_committee = extract_new_committee_info(&full_checkpoint.checkpoint_summary).unwrap();

    let target = ProofTarget::new_committee(new_committee.clone());
    let committee_proof = target.construct(&full_checkpoint).unwrap();

    assert!(committee_proof.verify(&committee).is_ok());
}

// Fail if the new committee does not match the target of the proof
#[tokio::test]
async fn test_incorrect_new_committee() {
    let (committee, full_checkpoint) = read_data();

    let committee_proof = Proof {
        checkpoint_summary: full_checkpoint.checkpoint_summary.clone(),
        proof_contents: ProofContents::CommitteeProof(CommitteeProof {}),
        targets: ProofTarget::new_committee(committee.clone()), // WRONG
    };

    assert!(committee_proof.verify(&committee).is_err());
}

// Fail if the certificate is incorrect even if no proof targets are given
#[tokio::test]
async fn test_fail_incorrect_cert() {
    let (_committee, full_checkpoint) = read_data();

    // Make a committee object using this
    let new_committee = extract_new_committee_info(&full_checkpoint.checkpoint_summary).unwrap();

    let target = ProofTarget::new_committee(new_committee.clone());
    let committee_proof = target.construct(&full_checkpoint).unwrap();

    assert!(committee_proof.verify(&new_committee).is_err());
}

#[tokio::test]
async fn test_object_target_fail_no_data() {
    let (committee, full_checkpoint) = read_transaction_data();

    let sample_object: Object = full_checkpoint.transactions[0].output_objects[0].clone();
    let sample_ref = sample_object.compute_object_reference();

    let bad_proof = Proof {
        checkpoint_summary: full_checkpoint.checkpoint_summary.clone(),
        proof_contents: ProofContents::CommitteeProof(CommitteeProof {}), // WRONG
        targets: ProofTarget::Objects(ObjectsTarget {
            objects: vec![(sample_ref, sample_object)],
        }),
    };

    assert!(bad_proof.verify(&committee).is_err());
}

#[tokio::test]
async fn test_object_target_success() {
    let (committee, full_checkpoint) = read_transaction_data();

    let sample_object: Object = full_checkpoint.transactions[0].output_objects[0].clone();
    let sample_ref = sample_object.compute_object_reference();

    let target = ProofTarget::Objects(ObjectsTarget {
        objects: vec![(sample_ref, sample_object)],
    });
    let object_proof = target.construct(&full_checkpoint).unwrap();

    assert!(object_proof.verify(&committee).is_ok());
}

#[tokio::test]
async fn test_object_target_fail_wrong_object() {
    let (committee, full_checkpoint) = read_transaction_data();

    let sample_object: Object = full_checkpoint.transactions[0].output_objects[0].clone();
    let wrong_object: Object = full_checkpoint.transactions[0].output_objects[1].clone();
    let mut sample_ref = sample_object.compute_object_reference();
    let wrong_ref = wrong_object.compute_object_reference();

    let target = ProofTarget::new_objects(vec![(wrong_ref, sample_object.clone())]); // WRONG
    let object_proof = target.construct(&full_checkpoint).unwrap();
    assert!(object_proof.verify(&committee).is_err());

    // Does not exist
    sample_ref.1 = sample_ref.1.next(); // WRONG

    let target = ProofTarget::new_objects(vec![(sample_ref, sample_object)]);
    let object_proof = target.construct(&full_checkpoint).unwrap();
    assert!(object_proof.verify(&committee).is_err());
}

#[tokio::test]
async fn test_event_target_fail_no_data() {
    let (committee, full_checkpoint) = read_data();
    let (sample_eid, sample_event) = sample_event(&full_checkpoint);

    let bad_proof = Proof {
        checkpoint_summary: full_checkpoint.checkpoint_summary.clone(),
        proof_contents: ProofContents::CommitteeProof(CommitteeProof {}), // WRONG
        targets: ProofTarget::new_events(vec![(sample_eid, sample_event)]),
    };

    assert!(bad_proof.verify(&committee).is_err());
}

#[tokio::test]
async fn test_event_target_success() {
    let (committee, full_checkpoint) = read_data();
    let (sample_eid, sample_event) = sample_event(&full_checkpoint);

    let target = ProofTarget::new_events(vec![(sample_eid, sample_event)]);
    let event_proof = target.construct(&full_checkpoint).unwrap();

    assert!(event_proof.verify(&committee).is_ok());
}

#[tokio::test]
async fn test_event_target_fail_bad_event() {
    let (committee, full_checkpoint) = read_data();
    let (mut sample_eid, sample_event) = sample_event(&full_checkpoint);
    sample_eid.event_seq += 1; // WRONG

    let target = ProofTarget::new_events(vec![(sample_eid, sample_event)]);
    let event_proof = target.construct(&full_checkpoint).unwrap();

    assert!(event_proof.verify(&committee).is_err());
}
