// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Shared proto-rendering layer: turns resolved BigTable data
//! (`CheckpointData`/`TransactionData` + object maps) into the `rtd.rpc.v2`
//! proto messages, honoring a `FieldMaskTree`. Used by both the v2 point-get
//! handlers and the list handlers so rendering is identical across
//! them.

use std::collections::HashMap;

use linku_common::ZipDebugEqIteratorExt;
use move_core_types::language_storage::StructTag;
use rtd_kvstore::{CheckpointData, TransactionData};
use rtd_rpc::field::FieldMaskTree;
use rtd_rpc::merge::Merge;
use rtd_rpc::proto::rtd::rpc::v2::{
    Checkpoint, Event, ExecutedTransaction, Object as ProtoObject, ObjectSet as ProtoObjectSet,
    Transaction, TransactionEffects, TransactionEvents, UserSignature,
};
use rtd_rpc_api::RpcError;
use rtd_rpc_api::proto::timestamp_ms_to_proto;
use rtd_types::TypeTag;
use rtd_types::base_types::ObjectID;
use rtd_types::crypto::{Ed25519RtdSignature, RtdSignatureInner, Signature, ToFromBytes};
use rtd_types::object::Object;
use rtd_types::object::rpc_visitor::proto::ProtoVisitor;
use rtd_types::signature::GenericSignature;
use rtd_types::storage::ObjectKey;
use rtd_types::transaction::TransactionData as RtdTransactionData;
use tracing::warn;

use crate::PackageResolver;
use crate::object_cache::ObjectMap;
use crate::resolve::compute_object_keys;

/// Maximum size in bytes for JSON-rendered Move values (1 MiB).
const MAX_JSON_MOVE_VALUE_SIZE: usize = 1024 * 1024;

/// Full Node's transaction store keeps the all-zero sender signature created
/// by `VerifiedTransaction::new_system_transaction`. Raw checkpoint contents
/// intentionally carry no user signatures for system transactions. Preserve
/// the raw HBase row and reconstruct that Full Node response-only placeholder.
fn fullnode_signatures(
    transaction: &RtdTransactionData,
    mut signatures: Vec<GenericSignature>,
) -> Vec<GenericSignature> {
    if signatures.is_empty() && transaction.as_v1().kind.is_system_tx() {
        signatures.push(GenericSignature::Signature(Signature::Ed25519RtdSignature(
            Ed25519RtdSignature::from_bytes(&[0; Ed25519RtdSignature::LENGTH])
                .expect("all-zero system signature has a fixed valid length"),
        )));
    }
    signatures
}

/// Render a Move value as JSON using the package resolver for type layout.
pub(crate) async fn render_json(
    resolver: &PackageResolver,
    struct_tag: &StructTag,
    contents: &[u8],
) -> Result<prost_types::Value, RpcError> {
    let type_tag = TypeTag::Struct(Box::new(struct_tag.clone()));
    let layout = resolver.type_layout(type_tag).await.map_err(|error| {
        RpcError::new(
            tonic::Code::Unavailable,
            format!("JSON type layout unavailable: {error}"),
        )
    })?;
    ProtoVisitor::new(MAX_JSON_MOVE_VALUE_SIZE)
        .deserialize_value(contents, &layout)
        .map_err(|error| {
            let code = match &error {
                rtd_types::object::rpc_visitor::Error::Meter(_) => tonic::Code::ResourceExhausted,
                _ => tonic::Code::DataLoss,
            };
            RpcError::new(code, format!("Unable to render requested JSON: {error}"))
        })
}

pub(crate) async fn object_to_response(
    source: &rtd_types::object::Object,
    mask: &FieldMaskTree,
    resolver: &PackageResolver,
) -> Result<ProtoObject, RpcError> {
    let mut message = ProtoObject::default();
    if mask.contains(ProtoObject::JSON_FIELD)
        && let Some(move_object) = source.data.try_as_move()
    {
        message.json = Some(Box::new(
            render_json(
                resolver,
                &move_object.type_().clone().into(),
                move_object.contents(),
            )
            .await?,
        ));
    }
    message.merge(source, mask);
    Ok(message)
}

/// Render a summary-only `CheckpointData` into the proto `Checkpoint` (the fast
/// path: read mask requests neither transactions nor objects).
pub(crate) fn checkpoint_to_response(
    checkpoint: CheckpointData,
    read_mask: &FieldMaskTree,
) -> Result<Checkpoint, RpcError> {
    let summary = checkpoint
        .summary
        .ok_or_else(|| anyhow::anyhow!("checkpoint summary missing"))?;
    let mut message = Checkpoint::default();
    message.merge(&summary, read_mask);

    if read_mask.contains(Checkpoint::SIGNATURE_FIELD) {
        let signatures = checkpoint
            .signatures
            .ok_or_else(|| anyhow::anyhow!("checkpoint signatures missing"))?;
        message.merge(signatures, read_mask);
    }

    if read_mask.contains(Checkpoint::CONTENTS_FIELD.name) {
        let contents = checkpoint
            .contents
            .ok_or_else(|| anyhow::anyhow!("checkpoint contents missing"))?;
        message.merge(contents, read_mask);
    }

    Ok(message)
}

/// Render nested checkpoint fields through the same resolver and masks as point queries.
/// Every object map belongs to one checkpoint; sorting keeps its public ObjectSet canonical.
pub(crate) async fn render_full_checkpoint(
    checkpoint: CheckpointData,
    txs: Vec<TransactionData>,
    objects: ObjectMap,
    read_mask: &FieldMaskTree,
    resolver: &PackageResolver,
) -> Result<Checkpoint, RpcError> {
    let summary = checkpoint
        .summary
        .as_ref()
        .ok_or_else(|| RpcError::new(tonic::Code::Unavailable, "checkpoint summary missing"))?;
    let height = summary.sequence_number;
    let timestamp = summary.timestamp_ms;
    let mut message = checkpoint_to_response(checkpoint, read_mask)?;
    if let Some(mask) = read_mask.subtree(Checkpoint::TRANSACTIONS_FIELD.name) {
        for tx in txs {
            if tx.checkpoint_number != height || tx.timestamp != timestamp {
                return Err(RpcError::new(
                    tonic::Code::DataLoss,
                    "Checkpoint transaction metadata mismatch",
                ));
            }
            message
                .transactions
                .push(transaction_to_response(tx, &mask, &objects, resolver).await?);
        }
    }
    if let Some(mask) = read_mask
        .subtree(Checkpoint::OBJECTS_FIELD.name)
        .and_then(|mask| mask.subtree(ProtoObjectSet::OBJECTS_FIELD.name))
    {
        let mut ordered: Vec<_> = objects.iter().collect();
        ordered.sort_by_key(|(key, _)| **key);
        let mut rendered = Vec::with_capacity(ordered.len());
        for (_, object) in ordered {
            rendered.push(object_to_response(object, &mask, resolver).await?);
        }
        message.objects = Some(ProtoObjectSet::default().with_objects(rendered));
    }
    Ok(message)
}

/// Render a `TransactionData` into the proto `ExecutedTransaction`. `objects`
/// supplies the transaction's canonical object set and the object types for
/// changed/unchanged-consensus effects. Missing entries fail an explicitly
/// requested complete object set but remain best-effort for effects annotation.
pub(crate) async fn transaction_to_response(
    source: TransactionData,
    mask: &FieldMaskTree,
    objects: &HashMap<ObjectKey, Object>,
    resolver: &PackageResolver,
) -> Result<ExecutedTransaction, RpcError> {
    let digest = source.digest;
    let object_mask = mask
        .subtree(ExecutedTransaction::OBJECTS_FIELD.name)
        .and_then(|object_set_mask| object_set_mask.subtree(ProtoObjectSet::OBJECTS_FIELD.name));
    let object_keys = if object_mask.is_some() {
        source.transaction_data.as_ref().ok_or_else(|| {
            RpcError::new(
                tonic::Code::Internal,
                format!("transaction {digest} data column missing"),
            )
        })?;
        source.effects.as_ref().ok_or_else(|| {
            RpcError::new(
                tonic::Code::Internal,
                format!("transaction {digest} effects column missing"),
            )
        })?;
        Some(compute_object_keys(&source))
    } else {
        None
    };
    let mut message = ExecutedTransaction::default();

    if mask.contains(ExecutedTransaction::DIGEST_FIELD.name) {
        message.digest = Some(digest.to_string());
    }

    if let Some(submask) = mask.subtree(ExecutedTransaction::TRANSACTION_FIELD.name)
        && let Some(tx_data) = source.transaction_data.as_ref()
    {
        message.transaction = Some(Transaction::merge_from(tx_data, &submask));
    }

    if let Some(submask) = mask.subtree(ExecutedTransaction::SIGNATURES_FIELD.name) {
        let tx_data = source.transaction_data.as_ref().ok_or_else(|| {
            RpcError::new(
                tonic::Code::Internal,
                format!("transaction {digest} data column missing for signatures"),
            )
        })?;
        let sigs = source.signatures.ok_or_else(|| {
            RpcError::new(
                tonic::Code::Internal,
                format!("transaction {digest} signatures column missing"),
            )
        })?;
        message.signatures = fullnode_signatures(tx_data, sigs)
            .into_iter()
            .map(|signature| UserSignature::merge_from(&signature, &submask))
            .collect();
    }

    if let Some(submask) = mask.subtree(ExecutedTransaction::EFFECTS_FIELD.name)
        && let Some(effects) = source.effects
    {
        let mut effects = TransactionEffects::merge_from(&effects, &submask);
        if submask.contains(TransactionEffects::UNCHANGED_LOADED_RUNTIME_OBJECTS_FIELD.name) {
            effects.unchanged_loaded_runtime_objects = source
                .unchanged_loaded_runtime_objects
                .iter()
                .map(Into::into)
                .collect();
        }
        for changed_object in effects.changed_objects.iter_mut() {
            let Ok(object_id) = changed_object.object_id().parse::<ObjectID>() else {
                warn!(
                    object_id = changed_object.object_id(),
                    "failed to parse object_id in changed_objects"
                );
                continue;
            };
            let version = changed_object
                .input_version_opt()
                .unwrap_or_else(|| changed_object.output_version());
            if let Some(object) = objects.get(&ObjectKey(object_id, version.into())) {
                changed_object.set_object_type(object_type_to_string(object.into()));
            }
        }

        for unchanged in effects.unchanged_consensus_objects.iter_mut() {
            let Ok(object_id) = unchanged.object_id().parse::<ObjectID>() else {
                warn!(
                    object_id = unchanged.object_id(),
                    "failed to parse object_id in unchanged_consensus_objects"
                );
                continue;
            };
            if let Some(object) = objects.get(&ObjectKey(object_id, unchanged.version().into())) {
                unchanged.set_object_type(object_type_to_string(object.into()));
            }
        }

        message.effects = Some(effects);
    }

    if let Some(submask) = mask.subtree(ExecutedTransaction::EVENTS_FIELD.name)
        && let Some(events) = &source.events
    {
        message.events = Some(TransactionEvents::merge_from(events, &submask));

        if let Some(event_mask) = submask.subtree(TransactionEvents::EVENTS_FIELD.name)
            && event_mask.contains(Event::JSON_FIELD.name)
            && let Some(proto_events) = message.events.as_mut()
        {
            for (proto_event, rtd_event) in
                proto_events.events.iter_mut().zip_debug_eq(&events.data)
            {
                proto_event.json = Some(Box::new(
                    render_json(resolver, &rtd_event.type_, &rtd_event.contents).await?,
                ));
            }
        }
    }
    if mask.contains(ExecutedTransaction::CHECKPOINT_FIELD.name) {
        message.checkpoint = Some(source.checkpoint_number);
    }
    if mask.contains(ExecutedTransaction::TIMESTAMP_FIELD.name) {
        message.timestamp = Some(timestamp_ms_to_proto(source.timestamp));
    }

    if mask.contains(ExecutedTransaction::BALANCE_CHANGES_FIELD.name) {
        message.balance_changes = source.balance_changes.into_iter().map(Into::into).collect();
    }

    if let Some(object_mask) = object_mask {
        let object_keys = object_keys.expect("object keys exist when an object mask is present");
        let mut rendered_objects = Vec::with_capacity(object_keys.len());
        for object_key in object_keys {
            let object = objects.get(&object_key).ok_or_else(|| {
                RpcError::new(
                    tonic::Code::Internal,
                    format!("unable to fetch object {object_key:?} for transaction {digest}"),
                )
            })?;
            rendered_objects.push(object_to_response(object, &object_mask, resolver).await?);
        }
        message.objects = Some(ProtoObjectSet::default().with_objects(rendered_objects));
    }

    Ok(message)
}

fn object_type_to_string(object_type: rtd_types::base_types::ObjectType) -> String {
    match object_type {
        rtd_types::base_types::ObjectType::Package => "package".to_owned(),
        rtd_types::base_types::ObjectType::Struct(move_object_type) => {
            move_object_type.to_canonical_string(true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use move_core_types::account_address::AccountAddress;
    use rtd_kvstore::TransactionData as KvTransactionData;
    use rtd_package_resolver::{Package, PackageStore, Resolver};
    use rtd_rpc::field::{FieldMask, FieldMaskUtil};
    use rtd_rpc::proto::rtd::rpc::v2::BalanceChange as ProtoBalanceChange;
    use rtd_rpc::proto::rtd::rpc::v2::ObjectReference;
    use rtd_types::TypeTag;
    use rtd_types::balance_change::BalanceChange;
    use rtd_types::base_types::{ObjectID, RtdAddress};
    use rtd_types::effects::TestEffectsBuilder;
    use rtd_types::object::Object;
    use rtd_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
    use rtd_types::storage::ObjectKey;
    use rtd_types::transaction::{
        Command, ProgrammableMoveCall, SenderSignedData, Transaction,
        TransactionData as RtdTransactionData, TransactionKind, VerifiedTransaction,
    };
    use rtd_types::type_input::{StructInput, TypeInput};
    use std::sync::Arc;

    use crate::v2::test_utils::{
        assert_identity_only_object_mask, canonical_transaction_object_keys, kv_transaction_data,
        response_object_keys, two_transaction_object_checkpoint,
    };
    use rtd_types::digests::TransactionDigest;

    fn test_tx_data() -> (TransactionDigest, RtdTransactionData) {
        let sender = RtdAddress::random_for_testing_only();
        let gas = Object::immutable_with_id_for_testing(ObjectID::random());
        let pt = {
            let mut builder = ProgrammableTransactionBuilder::new();
            builder.transfer_rtd(RtdAddress::random_for_testing_only(), None);
            builder.finish()
        };
        let data = RtdTransactionData::new_programmable(
            sender,
            vec![gas.compute_object_reference()],
            pt,
            1_000_000,
            1,
        );
        let tx = Transaction::new(SenderSignedData::new(data.clone(), vec![]));
        (*tx.digest(), data)
    }

    #[test]
    fn system_placeholder_matches_fullnode_without_changing_user_signatures() {
        let genesis = VerifiedTransaction::new_genesis_transaction(vec![]).into_inner();
        let genesis_data = genesis.transaction_data();
        let expected = genesis.tx_signatures().to_vec();
        assert_eq!(expected.len(), 1);
        assert_eq!(fullnode_signatures(genesis_data, vec![]), expected);
        assert_eq!(
            fullnode_signatures(genesis_data, expected.clone()),
            expected
        );

        let (_, user_data) = test_tx_data();
        assert!(fullnode_signatures(&user_data, vec![]).is_empty());
    }

    struct FrameworkPackageStore;

    #[async_trait::async_trait]
    impl PackageStore for FrameworkPackageStore {
        async fn fetch(&self, id: AccountAddress) -> rtd_package_resolver::Result<Arc<Package>> {
            let object = crate::v2::test_utils::framework_objects_for_json_tests()
                .into_iter()
                .find(|object| AccountAddress::from(object.id()) == id)
                .ok_or(rtd_package_resolver::error::Error::PackageNotFound(id))?;
            Ok(Arc::new(Package::read_from_object(&object)?))
        }
    }

    fn test_resolver() -> PackageResolver {
        let store: Arc<dyn PackageStore> = Arc::new(FrameworkPackageStore);
        Arc::new(Resolver::new(store))
    }

    #[tokio::test]
    async fn requested_json_rejects_missing_layout_and_corrupt_contents() {
        let resolver = test_resolver();
        let missing: StructTag = "0x42::missing::Value".parse().unwrap();
        let error = render_json(&resolver, &missing, &[]).await.unwrap_err();
        assert_eq!(tonic::Status::from(error).code(), tonic::Code::Unavailable);
        let tag: StructTag = "0x1::string::String".parse().unwrap();
        let error = render_json(&resolver, &tag, &[]).await.unwrap_err();
        assert_eq!(tonic::Status::from(error).code(), tonic::Code::DataLoss);
        let valid = bcs::to_bytes(&"recovered".as_bytes()).unwrap();
        assert!(render_json(&resolver, &tag, &valid).await.is_ok());
    }

    #[tokio::test]
    async fn checkpoint_rejects_mismatched_transaction_metadata() {
        let checkpoint = two_transaction_object_checkpoint();
        let data = CheckpointData {
            summary: Some(checkpoint.summary.data().clone()),
            signatures: Some(checkpoint.summary.auth_sig().clone()),
            contents: Some(checkpoint.contents.clone()),
        };
        let mut tx = kv_transaction_data(&checkpoint, 0);
        tx.checkpoint_number += 1;
        let mask = FieldMaskTree::from(FieldMask::from_paths(["transactions.digest"]));
        let error = render_full_checkpoint(
            data.clone(),
            vec![tx],
            Arc::new(HashMap::new()),
            &mask,
            &test_resolver(),
        )
        .await
        .unwrap_err();
        assert_eq!(tonic::Status::from(error).code(), tonic::Code::DataLoss);
        let mut tx = kv_transaction_data(&checkpoint, 0);
        tx.timestamp += 1;
        let error = render_full_checkpoint(
            data,
            vec![tx],
            Arc::new(HashMap::new()),
            &mask,
            &test_resolver(),
        )
        .await
        .unwrap_err();
        assert_eq!(tonic::Status::from(error).code(), tonic::Code::DataLoss);
    }

    #[tokio::test]
    async fn requested_json_reports_size_limit() {
        let tag: StructTag = "0x1::string::String".parse().unwrap();
        let bytes = bcs::to_bytes(&vec![b'x'; MAX_JSON_MOVE_VALUE_SIZE + 1]).unwrap();
        let error = render_json(&test_resolver(), &tag, &bytes)
            .await
            .unwrap_err();
        assert_eq!(
            tonic::Status::from(error).code(),
            tonic::Code::ResourceExhausted
        );
    }

    #[tokio::test]
    async fn transaction_to_response_returns_balance_changes_when_requested() {
        let (digest, tx_data) = test_tx_data();
        let tx = Transaction::new(SenderSignedData::new(tx_data.clone(), vec![]));
        let effects = TestEffectsBuilder::new(tx.data()).build();
        let balance_change = BalanceChange {
            address: RtdAddress::random_for_testing_only(),
            coin_type: TypeTag::U64,
            amount: 42,
        };
        let source = KvTransactionData {
            digest,
            transaction_data: Some(tx_data),
            signatures: Some(vec![]),
            effects: Some(effects),
            events: None,
            checkpoint_number: 7,
            timestamp: 42,
            balance_changes: vec![balance_change.clone()],
            unchanged_loaded_runtime_objects: vec![],
        };
        let mask = FieldMaskTree::from(FieldMask::from_str("balance_changes"));
        let resolver = test_resolver();

        let response = transaction_to_response(source, &mask, &HashMap::new(), &resolver)
            .await
            .expect("render should succeed");

        assert_eq!(
            response.balance_changes,
            vec![ProtoBalanceChange::from(balance_change)]
        );
    }

    #[tokio::test]
    async fn transaction_to_response_preserves_malformed_historical_type_inputs() {
        let (digest, mut tx_data) = test_tx_data();
        let RtdTransactionData::V1(data) = &mut tx_data;
        let TransactionKind::ProgrammableTransaction(programmable) = &mut data.kind else {
            panic!("test transaction should be programmable");
        };
        programmable
            .commands
            .push(Command::MoveCall(Box::new(ProgrammableMoveCall {
                package: ObjectID::random(),
                module: "type_name".to_owned(),
                function: "get".to_owned(),
                type_arguments: vec![TypeInput::Struct(Box::new(StructInput {
                    address: AccountAddress::ONE,
                    module: "example".to_owned(),
                    name: "Hapiness>".to_owned(),
                    type_params: vec![],
                }))],
                arguments: vec![],
            })));
        let expected_bcs = bcs::to_bytes(&tx_data).expect("transaction should serialize");
        let source = KvTransactionData {
            digest,
            transaction_data: Some(tx_data),
            signatures: None,
            effects: None,
            events: None,
            checkpoint_number: 7,
            timestamp: 42,
            balance_changes: vec![],
            unchanged_loaded_runtime_objects: vec![],
        };
        let mask = FieldMaskTree::from(FieldMask::from_paths(["transaction.bcs"]));

        let response = transaction_to_response(source, &mask, &HashMap::new(), &test_resolver())
            .await
            .expect("historical malformed type input should render");

        assert_eq!(
            response
                .transaction
                .expect("transaction should be present")
                .bcs
                .expect("transaction BCS should be present")
                .value
                .expect("transaction BCS value should be present"),
            expected_bcs
        );
    }

    #[tokio::test]
    async fn transaction_to_response_returns_unchanged_loaded_runtime_objects_when_requested() {
        let (digest, tx_data) = test_tx_data();
        let tx = Transaction::new(SenderSignedData::new(tx_data.clone(), vec![]));
        let effects = TestEffectsBuilder::new(tx.data()).build();
        let obj_key = ObjectKey(ObjectID::random(), 3.into());
        let source = KvTransactionData {
            digest,
            transaction_data: Some(tx_data),
            signatures: Some(vec![]),
            effects: Some(effects),
            events: None,
            checkpoint_number: 7,
            timestamp: 42,
            balance_changes: vec![],
            unchanged_loaded_runtime_objects: vec![obj_key],
        };
        let mask = FieldMaskTree::from(FieldMask::from_str(
            "effects.unchanged_loaded_runtime_objects",
        ));
        let resolver = test_resolver();

        let response = transaction_to_response(source, &mask, &HashMap::new(), &resolver)
            .await
            .expect("render should succeed");

        let effects = response.effects.expect("effects should be present");
        assert_eq!(
            effects.unchanged_loaded_runtime_objects,
            vec![ObjectReference::from(&obj_key)]
        );
    }

    #[tokio::test]
    async fn transaction_objects_are_canonical_and_honor_nested_masks() {
        let checkpoint = two_transaction_object_checkpoint();
        let objects = checkpoint
            .object_set
            .iter()
            .cloned()
            .map(|object| (ObjectKey(object.id(), object.version()), object))
            .collect::<HashMap<_, _>>();
        let expected_keys = canonical_transaction_object_keys(&checkpoint, 0);
        let sibling_created_id =
            rtd_types::test_checkpoint_data_builder::TestCheckpointBuilder::derive_object_id(11);
        let resolver = test_resolver();

        for (mask, identity_only) in [
            (FieldMask::from_paths(["objects"]), false),
            (
                FieldMask::from_paths(["objects.objects.object_id", "objects.objects.version"]),
                true,
            ),
        ] {
            let response = transaction_to_response(
                kv_transaction_data(&checkpoint, 0),
                &FieldMaskTree::from(mask),
                &objects,
                &resolver,
            )
            .await
            .expect("render should succeed");

            assert_eq!(response_object_keys(&response), expected_keys);
            assert!(
                response_object_keys(&response)
                    .iter()
                    .all(|key| key.0 != sibling_created_id),
                "transaction object set must exclude the sibling transaction's created object"
            );
            if identity_only {
                assert_identity_only_object_mask(&response);
            }
        }
    }

    #[tokio::test]
    async fn transaction_objects_require_complete_source_data() {
        let checkpoint = two_transaction_object_checkpoint();
        let source = kv_transaction_data(&checkpoint, 0);
        let digest = source.digest;
        let objects = checkpoint
            .object_set
            .iter()
            .cloned()
            .map(|object| (ObjectKey(object.id(), object.version()), object))
            .collect::<HashMap<_, _>>();
        let mask = FieldMaskTree::from(FieldMask::from_paths(["objects"]));
        let resolver = test_resolver();

        let mut missing_data = source.clone();
        missing_data.transaction_data = None;
        let status = tonic::Status::from(
            transaction_to_response(missing_data, &mask, &objects, &resolver)
                .await
                .expect_err("missing transaction data should fail"),
        );
        assert_eq!(status.code(), tonic::Code::Internal);
        assert_eq!(
            status.message(),
            format!("transaction {digest} data column missing")
        );

        let mut missing_effects = source.clone();
        missing_effects.effects = None;
        let status = tonic::Status::from(
            transaction_to_response(missing_effects, &mask, &objects, &resolver)
                .await
                .expect_err("missing effects should fail"),
        );
        assert_eq!(status.code(), tonic::Code::Internal);
        assert_eq!(
            status.message(),
            format!("transaction {digest} effects column missing")
        );

        let mut missing_object_map = objects;
        let missing_key = canonical_transaction_object_keys(&checkpoint, 0)[0];
        missing_object_map.remove(&missing_key);
        let status = tonic::Status::from(
            transaction_to_response(source, &mask, &missing_object_map, &resolver)
                .await
                .expect_err("missing canonical object should fail"),
        );
        assert_eq!(status.code(), tonic::Code::Internal);
        assert_eq!(
            status.message(),
            format!("unable to fetch object {missing_key:?} for transaction {digest}")
        );
    }
}
