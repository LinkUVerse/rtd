// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::mock_account::Account;
use crate::tx_generator::TxGenerator;
use move_core_types::identifier::Identifier;
use rtd_test_transaction_builder::TestTransactionBuilder;
use rtd_types::RTD_FRAMEWORK_PACKAGE_ID;
use rtd_types::digests::ChainIdentifier;
use rtd_types::gas_coin::GAS;
use rtd_types::transaction::{DEFAULT_VALIDATOR_GAS_PRICE, FundsWithdrawalArg, Transaction};
use std::sync::atomic::{AtomicU32, Ordering};

pub struct SendFundsTxGenerator {
    chain_identifier: ChainIdentifier,
    epoch: u64,
    transfer_amount: u64,
    nonce_counter: AtomicU32,
}

impl SendFundsTxGenerator {
    pub fn new(chain_identifier: ChainIdentifier, epoch: u64, transfer_amount: u64) -> Self {
        Self {
            chain_identifier,
            epoch,
            transfer_amount,
            nonce_counter: AtomicU32::new(0),
        }
    }
}

impl TxGenerator for SendFundsTxGenerator {
    fn generate_tx(&self, account: Account) -> Transaction {
        let nonce = self.nonce_counter.fetch_add(1, Ordering::Relaxed);
        let (recipient, _) = rtd_types::crypto::get_key_pair::<rtd_types::crypto::AccountKeyPair>();

        let mut tx_builder = TestTransactionBuilder::new_with_address_balance_gas(
            account.sender,
            DEFAULT_VALIDATOR_GAS_PRICE,
            self.chain_identifier,
            self.epoch,
            nonce,
        );

        {
            let builder = tx_builder.ptb_builder_mut();

            let withdrawal =
                FundsWithdrawalArg::balance_from_sender(self.transfer_amount, GAS::type_tag());
            let withdrawal_result = builder.funds_withdrawal(withdrawal).unwrap();

            let balance = builder.programmable_move_call(
                RTD_FRAMEWORK_PACKAGE_ID,
                Identifier::new("balance").unwrap(),
                Identifier::new("redeem_funds").unwrap(),
                vec![GAS::type_tag()],
                vec![withdrawal_result],
            );

            let recipient_arg = builder.pure(recipient).unwrap();
            builder.programmable_move_call(
                RTD_FRAMEWORK_PACKAGE_ID,
                Identifier::new("balance").unwrap(),
                Identifier::new("send_funds").unwrap(),
                vec![GAS::type_tag()],
                vec![balance, recipient_arg],
            );
        }

        tx_builder.build_and_sign(account.keypair.as_ref())
    }

    fn name(&self) -> &'static str {
        "SendFunds Transaction Generator"
    }
}
