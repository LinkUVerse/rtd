// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! This file contains the definition of the RtdBridgeEvent enum, of
//! which each variant is an emitted Event struct defind in the Move
//! Bridge module. We rely on structures in this file to decode
//! the bcs content of the emitted events.

#![allow(non_upper_case_globals)]

use crate::crypto::BridgeAuthorityPublicKey;
use crate::error::BridgeError;
use crate::error::BridgeResult;
use crate::types::BridgeAction;
use crate::types::RtdToEthTokenTransfer;
use ethers::types::Address as EthAddress;
use fastcrypto::encoding::Encoding;
use fastcrypto::encoding::Hex;
use move_core_types::language_storage::StructTag;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use rtd_json_rpc_types::RtdEvent;
use rtd_types::BRIDGE_PACKAGE_ID;
use rtd_types::TypeTag;
use rtd_types::base_types::RtdAddress;
use rtd_types::bridge::BridgeChainId;
use rtd_types::bridge::MoveTypeBridgeMessageKey;
use rtd_types::bridge::MoveTypeCommitteeMember;
use rtd_types::bridge::MoveTypeCommitteeMemberRegistration;
use rtd_types::collection_types::VecMap;
use rtd_types::crypto::ToFromBytes;
use rtd_types::parse_rtd_type_tag;

// `TokendDepositedEvent` emitted in bridge.move
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct MoveTokenDepositedEvent {
    pub seq_num: u64,
    pub source_chain: u8,
    pub sender_address: Vec<u8>,
    pub target_chain: u8,
    pub target_address: Vec<u8>,
    pub token_type: u8,
    pub amount_rtd_adjusted: u64,
}

macro_rules! new_move_event {
    ($struct_name:ident, $move_struct_name:ident) => {

        // `$move_struct_name` emitted in bridge.move
        #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
        pub struct $move_struct_name {
            pub message_key: MoveTypeBridgeMessageKey,
        }

        // Sanitized version of the given `move_struct_name`
        #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
        pub struct $struct_name {
            pub nonce: u64,
            pub source_chain: BridgeChainId,
        }

        impl TryFrom<$move_struct_name> for $struct_name {
            type Error = BridgeError;

            fn try_from(event: $move_struct_name) -> BridgeResult<Self> {
                let source_chain = BridgeChainId::try_from(event.message_key.source_chain).map_err(|_e| {
                    BridgeError::Generic(format!(
                        "Failed to convert {} to {}. Failed to convert source chain {} to BridgeChainId",
                        stringify!($move_struct_name),
                        stringify!($struct_name),
                        event.message_key.source_chain,
                    ))
                })?;
                Ok(Self {
                    nonce: event.message_key.bridge_seq_num,
                    source_chain,
                })
            }
        }
    };
}

new_move_event!(TokenTransferClaimed, MoveTokenTransferClaimed);
new_move_event!(TokenTransferApproved, MoveTokenTransferApproved);
new_move_event!(
    TokenTransferAlreadyApproved,
    MoveTokenTransferAlreadyApproved
);
new_move_event!(TokenTransferAlreadyClaimed, MoveTokenTransferAlreadyClaimed);
new_move_event!(TokenTransferLimitExceed, MoveTokenTransferLimitExceed);

// `EmergencyOpEvent` emitted in bridge.move
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EmergencyOpEvent {
    pub frozen: bool,
}

// `CommitteeUpdateEvent` emitted in committee.move
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveCommitteeUpdateEvent {
    pub members: VecMap<Vec<u8>, MoveTypeCommitteeMember>,
    pub stake_participation_percentage: u64,
}

// `CommitteeMemberUrlUpdateEvent` emitted in committee.move
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveCommitteeMemberUrlUpdateEvent {
    pub member: Vec<u8>,
    pub new_url: Vec<u8>,
}

// `BlocklistValidatorEvent` emitted in committee.move
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveBlocklistValidatorEvent {
    pub blocklisted: bool,
    pub public_keys: Vec<Vec<u8>>,
}

// `UpdateRouteLimitEvent` emitted in limiter.move
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UpdateRouteLimitEvent {
    pub sending_chain: u8,
    pub receiving_chain: u8,
    pub new_limit: u64,
}

// `TokenRegistrationEvent` emitted in treasury.move
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveTokenRegistrationEvent {
    pub type_name: String,
    pub decimal: u8,
    pub native_token: bool,
}

// Sanitized version of MoveTokenRegistrationEvent
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TokenRegistrationEvent {
    pub type_name: TypeTag,
    pub decimal: u8,
    pub native_token: bool,
}

impl TryFrom<MoveTokenRegistrationEvent> for TokenRegistrationEvent {
    type Error = BridgeError;

    fn try_from(event: MoveTokenRegistrationEvent) -> BridgeResult<Self> {
        let type_name = parse_rtd_type_tag(&format!("0x{}", event.type_name)).map_err(|e| {
            BridgeError::InternalError(format!(
                "Failed to parse TypeTag: {e}, type name: {}",
                event.type_name
            ))
        })?;

        Ok(Self {
            type_name,
            decimal: event.decimal,
            native_token: event.native_token,
        })
    }
}

// `NewTokenEvent` emitted in treasury.move
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveNewTokenEvent {
    pub token_id: u8,
    pub type_name: String,
    pub native_token: bool,
    pub decimal_multiplier: u64,
    pub notional_value: u64,
}

// Sanitized version of MoveNewTokenEvent
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct NewTokenEvent {
    pub token_id: u8,
    pub type_name: TypeTag,
    pub native_token: bool,
    pub decimal_multiplier: u64,
    pub notional_value: u64,
}

impl TryFrom<MoveNewTokenEvent> for NewTokenEvent {
    type Error = BridgeError;

    fn try_from(event: MoveNewTokenEvent) -> BridgeResult<Self> {
        let type_name = parse_rtd_type_tag(&format!("0x{}", event.type_name)).map_err(|e| {
            BridgeError::InternalError(format!(
                "Failed to parse TypeTag: {e}, type name: {}",
                event.type_name
            ))
        })?;

        Ok(Self {
            token_id: event.token_id,
            type_name,
            native_token: event.native_token,
            decimal_multiplier: event.decimal_multiplier,
            notional_value: event.notional_value,
        })
    }
}

// `UpdateTokenPriceEvent` emitted in treasury.move
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UpdateTokenPriceEvent {
    pub token_id: u8,
    pub new_price: u64,
}

// Sanitized version of MoveTokenDepositedEvent
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct EmittedRtdToEthTokenBridgeV1 {
    pub nonce: u64,
    pub rtd_chain_id: BridgeChainId,
    pub eth_chain_id: BridgeChainId,
    pub rtd_address: RtdAddress,
    pub eth_address: EthAddress,
    pub token_id: u8,
    // The amount of tokens deposited with decimal points on Rtd side
    pub amount_rtd_adjusted: u64,
}

// Sanitized version of MoveCommitteeUpdateEvent
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct CommitteeUpdate {
    pub members: Vec<MoveTypeCommitteeMember>,
    pub stake_participation_percentage: u64,
}

impl TryFrom<MoveCommitteeUpdateEvent> for CommitteeUpdate {
    type Error = BridgeError;

    fn try_from(event: MoveCommitteeUpdateEvent) -> BridgeResult<Self> {
        let members = event
            .members
            .contents
            .into_iter()
            .map(|v| v.value)
            .collect();
        Ok(Self {
            members,
            stake_participation_percentage: event.stake_participation_percentage,
        })
    }
}

// Sanitized version of MoveBlocklistValidatorEvent
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct BlocklistValidatorEvent {
    pub blocklisted: bool,
    pub public_keys: Vec<BridgeAuthorityPublicKey>,
}

impl TryFrom<MoveBlocklistValidatorEvent> for BlocklistValidatorEvent {
    type Error = BridgeError;

    fn try_from(event: MoveBlocklistValidatorEvent) -> BridgeResult<Self> {
        let public_keys = event.public_keys.into_iter().map(|bytes|
            BridgeAuthorityPublicKey::from_bytes(&bytes).map_err(|e|
                BridgeError::Generic(format!("Failed to convert MoveBlocklistValidatorEvent to BlocklistValidatorEvent. Failed to convert public key to BridgeAuthorityPublicKey: {:?}", e))
            )
        ).collect::<BridgeResult<Vec<_>>>()?;
        Ok(Self {
            blocklisted: event.blocklisted,
            public_keys,
        })
    }
}

// Sanitized version of MoveCommitteeMemberUrlUpdateEvent
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct CommitteeMemberUrlUpdateEvent {
    pub member: BridgeAuthorityPublicKey,
    pub new_url: String,
}

impl TryFrom<MoveCommitteeMemberUrlUpdateEvent> for CommitteeMemberUrlUpdateEvent {
    type Error = BridgeError;

    fn try_from(event: MoveCommitteeMemberUrlUpdateEvent) -> BridgeResult<Self> {
        let member = BridgeAuthorityPublicKey::from_bytes(&event.member).map_err(|e|
            BridgeError::Generic(format!("Failed to convert MoveBlocklistValidatorEvent to BlocklistValidatorEvent. Failed to convert public key to BridgeAuthorityPublicKey: {:?}", e))
        )?;
        let new_url = String::from_utf8(event.new_url).map_err(|e|
            BridgeError::Generic(format!("Failed to convert MoveBlocklistValidatorEvent to BlocklistValidatorEvent. Failed to convert new_url to String: {:?}", e))
        )?;
        Ok(Self { member, new_url })
    }
}

impl TryFrom<MoveTokenDepositedEvent> for EmittedRtdToEthTokenBridgeV1 {
    type Error = BridgeError;

    fn try_from(event: MoveTokenDepositedEvent) -> BridgeResult<Self> {
        if event.amount_rtd_adjusted == 0 {
            return Err(BridgeError::ZeroValueBridgeTransfer(format!(
                "Failed to convert MoveTokenDepositedEvent to EmittedRtdToEthTokenBridgeV1. Manual intervention is required. 0 value transfer should not be allowed in Move: {:?}",
                event,
            )));
        }

        let token_id = event.token_type;
        let rtd_chain_id = BridgeChainId::try_from(event.source_chain).map_err(|_e| {
            BridgeError::Generic(format!(
                "Failed to convert MoveTokenDepositedEvent to EmittedRtdToEthTokenBridgeV1. Failed to convert source chain {} to BridgeChainId",
                event.token_type,
            ))
        })?;
        let eth_chain_id = BridgeChainId::try_from(event.target_chain).map_err(|_e| {
            BridgeError::Generic(format!(
                "Failed to convert MoveTokenDepositedEvent to EmittedRtdToEthTokenBridgeV1. Failed to convert target chain {} to BridgeChainId",
                event.token_type,
            ))
        })?;
        if !rtd_chain_id.is_rtd_chain() {
            return Err(BridgeError::Generic(format!(
                "Failed to convert MoveTokenDepositedEvent to EmittedRtdToEthTokenBridgeV1. Invalid source chain {}",
                event.source_chain
            )));
        }
        if eth_chain_id.is_rtd_chain() {
            return Err(BridgeError::Generic(format!(
                "Failed to convert MoveTokenDepositedEvent to EmittedRtdToEthTokenBridgeV1. Invalid target chain {}",
                event.target_chain
            )));
        }

        let rtd_address = RtdAddress::from_bytes(event.sender_address)
            .map_err(|e| BridgeError::Generic(format!("Failed to convert MoveTokenDepositedEvent to EmittedRtdToEthTokenBridgeV1. Failed to convert sender_address to RtdAddress: {:?}", e)))?;
        let eth_address = EthAddress::from_str(&Hex::encode(&event.target_address))?;

        Ok(Self {
            nonce: event.seq_num,
            rtd_chain_id,
            eth_chain_id,
            rtd_address,
            eth_address,
            token_id,
            amount_rtd_adjusted: event.amount_rtd_adjusted,
        })
    }
}

crate::declare_events!(
    RtdToEthTokenBridgeV1(EmittedRtdToEthTokenBridgeV1) => ("bridge::TokenDepositedEvent", MoveTokenDepositedEvent),
    TokenTransferApproved(TokenTransferApproved) => ("bridge::TokenTransferApproved", MoveTokenTransferApproved),
    TokenTransferClaimed(TokenTransferClaimed) => ("bridge::TokenTransferClaimed", MoveTokenTransferClaimed),
    TokenTransferAlreadyApproved(TokenTransferAlreadyApproved) => ("bridge::TokenTransferAlreadyApproved", MoveTokenTransferAlreadyApproved),
    TokenTransferAlreadyClaimed(TokenTransferAlreadyClaimed) => ("bridge::TokenTransferAlreadyClaimed", MoveTokenTransferAlreadyClaimed),
    TokenTransferLimitExceed(TokenTransferLimitExceed) => ("bridge::TokenTransferLimitExceed", MoveTokenTransferLimitExceed),
    EmergencyOpEvent(EmergencyOpEvent) => ("bridge::EmergencyOpEvent", EmergencyOpEvent),
    // No need to define a sanitized event struct for MoveTypeCommitteeMemberRegistration
    // because the info provided by validators could be invalid
    CommitteeMemberRegistration(MoveTypeCommitteeMemberRegistration) => ("committee::CommitteeMemberRegistration", MoveTypeCommitteeMemberRegistration),
    CommitteeUpdateEvent(CommitteeUpdate) => ("committee::CommitteeUpdateEvent", MoveCommitteeUpdateEvent),
    CommitteeMemberUrlUpdateEvent(CommitteeMemberUrlUpdateEvent) => ("committee::CommitteeMemberUrlUpdateEvent", MoveCommitteeMemberUrlUpdateEvent),
    BlocklistValidatorEvent(BlocklistValidatorEvent) => ("committee::BlocklistValidatorEvent", MoveBlocklistValidatorEvent),
    TokenRegistrationEvent(TokenRegistrationEvent) => ("treasury::TokenRegistrationEvent", MoveTokenRegistrationEvent),
    NewTokenEvent(NewTokenEvent) => ("treasury::NewTokenEvent", MoveNewTokenEvent),
    UpdateTokenPriceEvent(UpdateTokenPriceEvent) => ("treasury::UpdateTokenPriceEvent", UpdateTokenPriceEvent),
    UpdateRouteLimitEvent(UpdateRouteLimitEvent) => ("limiter::UpdateRouteLimitEvent", UpdateRouteLimitEvent),

    // Add new event types here. Format:
    // EnumVariantName(Struct) => ("{module}::{event_struct}", CorrespondingMoveStruct)
);

#[macro_export]
macro_rules! declare_events {
    ($($variant:ident($type:path) => ($event_tag:expr, $event_struct:path)),* $(,)?) => {

        #[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
        pub enum RtdBridgeEvent {
            $($variant($type),)*
        }

        $(pub static $variant: OnceCell<StructTag> = OnceCell::new();)*

        pub(crate) fn init_all_struct_tags() {
            $($variant.get_or_init(|| {
                StructTag::from_str(&format!("0x{}::{}", BRIDGE_PACKAGE_ID.to_hex(), $event_tag)).unwrap()
            });)*
        }

        // Try to convert a RtdEvent into RtdBridgeEvent
        impl RtdBridgeEvent {
            pub fn try_from_rtd_event(event: &RtdEvent) -> BridgeResult<Option<RtdBridgeEvent>> {
                init_all_struct_tags(); // Ensure all tags are initialized

                // Unwrap safe: we inited above
                $(
                    if &event.type_ == $variant.get().unwrap() {
                        let event_struct: $event_struct = bcs::from_bytes(event.bcs.bytes()).map_err(|e| BridgeError::InternalError(format!("Failed to deserialize event to {}: {:?}", stringify!($event_struct), e)))?;
                        return Ok(Some(RtdBridgeEvent::$variant(event_struct.try_into()?)));
                    }
                )*
                Ok(None)
            }
        }
    };
}

impl RtdBridgeEvent {
    pub fn try_into_bridge_action(self) -> Option<BridgeAction> {
        match self {
            RtdBridgeEvent::RtdToEthTokenBridgeV1(event) => {
                let EmittedRtdToEthTokenBridgeV1 {
                    nonce,
                    rtd_chain_id,
                    eth_chain_id,
                    rtd_address,
                    eth_address,
                    token_id,
                    amount_rtd_adjusted,
                } = event;

                Some(BridgeAction::RtdToEthTokenTransfer(RtdToEthTokenTransfer {
                    nonce,
                    rtd_chain_id,
                    eth_chain_id,
                    rtd_address,
                    eth_address,
                    token_id,
                    amount_adjusted: amount_rtd_adjusted,
                }))
            }
            RtdBridgeEvent::TokenTransferApproved(_event) => None,
            RtdBridgeEvent::TokenTransferClaimed(_event) => None,
            RtdBridgeEvent::TokenTransferAlreadyApproved(_event) => None,
            RtdBridgeEvent::TokenTransferAlreadyClaimed(_event) => None,
            RtdBridgeEvent::TokenTransferLimitExceed(_event) => None,
            RtdBridgeEvent::EmergencyOpEvent(_event) => None,
            RtdBridgeEvent::CommitteeMemberRegistration(_event) => None,
            RtdBridgeEvent::CommitteeUpdateEvent(_event) => None,
            RtdBridgeEvent::CommitteeMemberUrlUpdateEvent(_event) => None,
            RtdBridgeEvent::BlocklistValidatorEvent(_event) => None,
            RtdBridgeEvent::TokenRegistrationEvent(_event) => None,
            RtdBridgeEvent::NewTokenEvent(_event) => None,
            RtdBridgeEvent::UpdateTokenPriceEvent(_event) => None,
            RtdBridgeEvent::UpdateRouteLimitEvent(_event) => None,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::crypto::BridgeAuthorityKeyPair;
    use crate::e2e_tests::test_utils::BridgeTestClusterBuilder;
    use crate::types::BridgeAction;
    use crate::types::RtdToEthBridgeAction;
    use ethers::types::Address as EthAddress;
    use rtd_json_rpc_types::BcsEvent;
    use rtd_json_rpc_types::RtdEvent;
    use rtd_types::Identifier;
    use rtd_types::base_types::ObjectID;
    use rtd_types::base_types::RtdAddress;
    use rtd_types::bridge::BridgeChainId;
    use rtd_types::bridge::TOKEN_ID_RTD;
    use rtd_types::crypto::get_key_pair;
    use rtd_types::digests::TransactionDigest;
    use rtd_types::event::EventID;

    /// Returns a test RtdEvent and corresponding BridgeAction
    pub fn get_test_rtd_event_and_action(identifier: Identifier) -> (RtdEvent, BridgeAction) {
        init_all_struct_tags(); // Ensure all tags are initialized
        let sanitized_event = EmittedRtdToEthTokenBridgeV1 {
            nonce: 1,
            rtd_chain_id: BridgeChainId::RtdTestnet,
            rtd_address: RtdAddress::random_for_testing_only(),
            eth_chain_id: BridgeChainId::EthSepolia,
            eth_address: EthAddress::random(),
            token_id: TOKEN_ID_RTD,
            amount_rtd_adjusted: 100,
        };
        let emitted_event = MoveTokenDepositedEvent {
            seq_num: sanitized_event.nonce,
            source_chain: sanitized_event.rtd_chain_id as u8,
            sender_address: sanitized_event.rtd_address.to_vec(),
            target_chain: sanitized_event.eth_chain_id as u8,
            target_address: sanitized_event.eth_address.as_bytes().to_vec(),
            token_type: sanitized_event.token_id,
            amount_rtd_adjusted: sanitized_event.amount_rtd_adjusted,
        };

        let tx_digest = TransactionDigest::random();
        let event_idx = 10u16;
        let bridge_action = BridgeAction::RtdToEthBridgeAction(RtdToEthBridgeAction {
            rtd_tx_digest: tx_digest,
            rtd_tx_event_index: event_idx,
            rtd_bridge_event: sanitized_event.clone(),
        });
        let event = RtdEvent {
            type_: RtdToEthTokenBridgeV1.get().unwrap().clone(),
            bcs: BcsEvent::new(bcs::to_bytes(&emitted_event).unwrap()),
            id: EventID {
                tx_digest,
                event_seq: event_idx as u64,
            },

            // The following fields do not matter as of writing,
            // but if tests start to fail, it's worth checking these fields.
            package_id: ObjectID::ZERO,
            transaction_module: identifier.clone(),
            sender: RtdAddress::random_for_testing_only(),
            parsed_json: serde_json::json!({"test": "test"}),
            timestamp_ms: None,
        };
        (event, bridge_action)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn test_bridge_events_when_init() {
        telemetry_subscribers::init_for_testing();
        init_all_struct_tags();
        let mut bridge_test_cluster = BridgeTestClusterBuilder::new()
            .with_eth_env(false)
            .with_bridge_cluster(false)
            .with_num_validators(2)
            .build()
            .await;

        let events = bridge_test_cluster
            .new_bridge_events(
                HashSet::from_iter([
                    CommitteeMemberRegistration.get().unwrap().clone(),
                    CommitteeUpdateEvent.get().unwrap().clone(),
                    TokenRegistrationEvent.get().unwrap().clone(),
                    NewTokenEvent.get().unwrap().clone(),
                ]),
                false,
            )
            .await;
        let mut mask = 0u8;
        for event in events.iter() {
            match RtdBridgeEvent::try_from_rtd_event(event).unwrap().unwrap() {
                RtdBridgeEvent::CommitteeMemberRegistration(_event) => mask |= 0x1,
                RtdBridgeEvent::CommitteeUpdateEvent(_event) => mask |= 0x2,
                RtdBridgeEvent::TokenRegistrationEvent(_event) => mask |= 0x4,
                RtdBridgeEvent::NewTokenEvent(_event) => mask |= 0x8,
                _ => panic!("Got unexpected event: {:?}", event),
            }
        }
        // assert all the above events are emitted
        assert_eq!(mask, 0xF);

        // TODO: trigger other events and make sure they are converted correctly
    }

    #[test]
    fn test_conversion_for_committee_member_url_update_event() {
        let (_, kp): (_, BridgeAuthorityKeyPair) = get_key_pair();
        let new_url = "https://example.com:443";
        let event: CommitteeMemberUrlUpdateEvent = MoveCommitteeMemberUrlUpdateEvent {
            member: kp.public.as_bytes().to_vec(),
            new_url: new_url.as_bytes().to_vec(),
        }
        .try_into()
        .unwrap();
        assert_eq!(event.member, kp.public);
        assert_eq!(event.new_url, new_url);

        CommitteeMemberUrlUpdateEvent::try_from(MoveCommitteeMemberUrlUpdateEvent {
            member: vec![1, 2, 3],
            new_url: new_url.as_bytes().to_vec(),
        })
        .unwrap_err();

        CommitteeMemberUrlUpdateEvent::try_from(MoveCommitteeMemberUrlUpdateEvent {
            member: kp.public.as_bytes().to_vec(),
            new_url: [240, 130, 130, 172].into(),
        })
        .unwrap_err();
    }

    // TODO: add conversion tests for other events

    #[test]
    fn test_0_rtd_amount_conversion_for_rtd_event() {
        let emitted_event = MoveTokenDepositedEvent {
            seq_num: 1,
            source_chain: BridgeChainId::RtdTestnet as u8,
            sender_address: RtdAddress::random_for_testing_only().to_vec(),
            target_chain: BridgeChainId::EthSepolia as u8,
            target_address: EthAddress::random().as_bytes().to_vec(),
            token_type: TOKEN_ID_RTD,
            amount_rtd_adjusted: 0,
        };
        match EmittedRtdToEthTokenBridgeV1::try_from(emitted_event).unwrap_err() {
            BridgeError::ZeroValueBridgeTransfer(_) => (),
            other => panic!("Expected Generic error, got: {:?}", other),
        }
    }
}
