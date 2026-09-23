// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

mod common;

use common::test_chain;
use rtd_light_client::verifier::extract_verified_effects_and_events;
use rtd_types::digests::TransactionDigest;
use rtd_types::effects::TransactionEffectsAPI;

fn event_transaction_digest() -> TransactionDigest {
    let checkpoint = &test_chain().end_of_epoch;
    *checkpoint
        .transactions
        .iter()
        .find(|tx| {
            tx.events
                .as_ref()
                .is_some_and(|events| !events.data.is_empty())
        })
        .expect("RTD epoch event transaction")
        .transaction
        .digest()
}

#[test]
fn test_checkpoint_all_good() {
    let chain = test_chain();
    let (effects, events) = extract_verified_effects_and_events(
        &chain.end_of_epoch,
        &chain.committee,
        event_transaction_digest(),
    )
    .unwrap();
    assert_eq!(effects.transaction_digest(), &event_transaction_digest());
    assert!(events.is_some_and(|events| !events.data.is_empty()));
}

#[test]
fn test_checkpoint_bad_committee() {
    let chain = test_chain();
    let mut committee = chain.committee.clone();
    committee.epoch += 10;
    assert!(
        extract_verified_effects_and_events(
            &chain.end_of_epoch,
            &committee,
            event_transaction_digest(),
        )
        .is_err()
    );
}

#[test]
fn test_checkpoint_no_transaction() {
    let chain = test_chain();
    let missing_digest = TransactionDigest::random();
    assert!(
        chain
            .end_of_epoch
            .transactions
            .iter()
            .all(|tx| tx.transaction.digest() != &missing_digest)
    );
    assert!(
        extract_verified_effects_and_events(&chain.end_of_epoch, &chain.committee, missing_digest,)
            .is_err()
    );
}

#[test]
fn test_checkpoint_bad_contents() {
    let chain = test_chain();
    let mut checkpoint = chain.end_of_epoch.clone();
    checkpoint.checkpoint_contents = chain.transaction.checkpoint_contents.clone();
    assert!(
        extract_verified_effects_and_events(
            &checkpoint,
            &chain.committee,
            event_transaction_digest(),
        )
        .is_err()
    );
}

#[test]
fn test_checkpoint_bad_events() {
    let chain = test_chain();
    let mut checkpoint = chain.end_of_epoch.clone();
    let transaction = checkpoint
        .transactions
        .iter_mut()
        .find(|tx| tx.transaction.digest() == &event_transaction_digest())
        .expect("RTD epoch event transaction");
    let events = transaction.events.as_mut().unwrap();
    events.data.push(events.data[0].clone());
    assert!(
        extract_verified_effects_and_events(
            &checkpoint,
            &chain.committee,
            event_transaction_digest(),
        )
        .is_err()
    );
}
