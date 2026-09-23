// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::OnceLock;

use rtd_types::committee::Committee;
use rtd_types::crypto::{AuthoritySignInfo, AuthoritySignature, RtdAuthoritySignature};
use rtd_types::full_checkpoint_content::CheckpointData;
use rtd_types::gas_coin::MIST_PER_RTD;
use rtd_types::messages_checkpoint::{
    CertifiedCheckpointSummary, CheckpointArtifacts, CheckpointCommitment, VerifiedCheckpoint,
};
use rtd_types::storage::ReadStore;
use shared_crypto::intent::{Intent, IntentMessage, IntentScope};
use simulacrum::Simulacrum;

#[allow(dead_code)] // Each integration-test binary uses a different subset of this shared fixture.
pub struct TestChain {
    pub committee: Committee,
    pub transaction: CheckpointData,
    pub end_of_epoch: CheckpointData,
    pub ocs: CheckpointData,
}

pub fn test_chain() -> &'static TestChain {
    static CHAIN: OnceLock<TestChain> = OnceLock::new();
    CHAIN.get_or_init(TestChain::new)
}

impl TestChain {
    fn new() -> Self {
        let mut sim = Simulacrum::new();
        let committee = sim
            .store()
            .get_committee_by_epoch(0)
            .expect("RTD genesis committee")
            .clone();

        sim.funded_account(MIST_PER_RTD)
            .expect("fund an RTD account");
        let transaction_checkpoint = sim.create_checkpoint();
        let transaction = checkpoint_data(&sim, transaction_checkpoint);

        sim.advance_epoch(Default::default());
        let end_of_epoch_checkpoint = sim
            .store()
            .get_highest_checkpint()
            .expect("end-of-epoch checkpoint")
            .clone();
        let end_of_epoch = checkpoint_data(&sim, end_of_epoch_checkpoint);
        assert!(end_of_epoch.checkpoint_summary.end_of_epoch_data.is_some());
        assert!(end_of_epoch.transactions.iter().any(|tx| {
            tx.events
                .as_ref()
                .is_some_and(|events| !events.data.is_empty())
        }));

        let mut ocs = transaction.clone();
        let artifacts_digest = CheckpointArtifacts::from(&ocs)
            .digest()
            .expect("RTD checkpoint artifacts digest");
        let mut summary = ocs.checkpoint_summary.data().clone();
        summary
            .checkpoint_commitments
            .push(CheckpointCommitment::from(artifacts_digest));
        let signatures = committee
            .voting_rights
            .iter()
            .map(|(name, _)| {
                let message =
                    IntentMessage::new(Intent::rtd_app(IntentScope::CheckpointSummary), &summary);
                let key = sim.keystore().validator(name).expect("RTD validator key");
                AuthoritySignInfo {
                    epoch: summary.epoch,
                    authority: *name,
                    signature: AuthoritySignature::new_secure(&message, &summary.epoch, key),
                }
            })
            .collect();
        ocs.checkpoint_summary = CertifiedCheckpointSummary::new(summary, signatures, &committee)
            .expect("certified RTD checkpoint with artifacts commitment");

        Self {
            committee,
            transaction,
            end_of_epoch,
            ocs,
        }
    }
}

fn checkpoint_data(sim: &Simulacrum, checkpoint: VerifiedCheckpoint) -> CheckpointData {
    let contents = sim
        .store()
        .get_checkpoint_contents(&checkpoint.content_digest)
        .expect("RTD checkpoint contents")
        .clone();
    sim.get_checkpoint_data(checkpoint, contents)
        .expect("complete RTD checkpoint data")
        .into()
}
