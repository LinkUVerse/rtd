// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use move_core_types::{account_address::AccountAddress, ident_str};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use sui_types::crypto::{Signature, Signer};
use sui_types::transaction::*;

use super::*;
use crate::authority_client::AuthorityAPI;
use crate::test_authority_clients::{LocalAuthorityClient, LocalAuthorityClientFaultConfig};
use sui_types::utils::to_sender_signed_transaction;

use sui_types::effects::TransactionEffects;
use sui_types::messages_grpc::{TransactionInfoRequest, VerifiedObjectInfoResponse};

pub fn set_local_client_config(
    authorities: &mut AuthorityAggregator<LocalAuthorityClient>,
    index: usize,
    config: LocalAuthorityClientFaultConfig,
) {
    let mut clients = authorities.clone_inner_clients_test_only();
    let mut clients_values_mut = clients.values_mut();
    let mut i = 0;
    while i < index {
        clients_values_mut.next();
        i += 1;
    }
    clients_values_mut
        .next()
        .unwrap()
        .authority_client_mut()
        .fault_config = config;
    let clients = clients.into_iter().map(|(k, v)| (k, Arc::new(v))).collect();
    authorities.authority_clients = Arc::new(clients);
}

pub fn create_object_move_transaction(
    src: SuiAddress,
    secret: &dyn Signer<Signature>,
    dest: SuiAddress,
    value: u64,
    package_id: ObjectID,
    gas_object_ref: ObjectRef,
    gas_price: u64,
) -> Transaction {
    // When creating an object_basics object, we provide the value (u64) and address which will own the object
    let arguments = vec![
        CallArg::Pure(value.to_le_bytes().to_vec()),
        CallArg::Pure(bcs::to_bytes(&AccountAddress::from(dest)).unwrap()),
    ];

    to_sender_signed_transaction(
        TransactionData::new_move_call(
            src,
            package_id,
            ident_str!("object_basics").to_owned(),
            ident_str!("create").to_owned(),
            Vec::new(),
            gas_object_ref,
            arguments,
            TEST_ONLY_GAS_UNIT_FOR_HEAVY_COMPUTATION_STORAGE * gas_price,
            gas_price,
        )
        .unwrap(),
        secret,
    )
}

pub fn delete_object_move_transaction(
    src: SuiAddress,
    secret: &dyn Signer<Signature>,
    object_ref: ObjectRef,
    framework_obj_id: ObjectID,
    gas_object_ref: ObjectRef,
    gas_price: u64,
) -> Transaction {
    to_sender_signed_transaction(
        TransactionData::new_move_call(
            src,
            framework_obj_id,
            ident_str!("object_basics").to_owned(),
            ident_str!("delete").to_owned(),
            Vec::new(),
            gas_object_ref,
            vec![CallArg::Object(ObjectArg::ImmOrOwnedObject(object_ref))],
            TEST_ONLY_GAS_UNIT_FOR_OBJECT_BASICS * gas_price,
            gas_price,
        )
        .unwrap(),
        secret,
    )
}

pub fn set_object_move_transaction(
    src: SuiAddress,
    secret: &dyn Signer<Signature>,
    object_ref: ObjectRef,
    value: u64,
    framework_obj_id: ObjectID,
    gas_object_ref: ObjectRef,
    gas_price: u64,
) -> Transaction {
    let args = vec![
        CallArg::Object(ObjectArg::ImmOrOwnedObject(object_ref)),
        CallArg::Pure(bcs::to_bytes(&value).unwrap()),
    ];

    to_sender_signed_transaction(
        TransactionData::new_move_call(
            src,
            framework_obj_id,
            ident_str!("object_basics").to_owned(),
            ident_str!("set_value").to_owned(),
            Vec::new(),
            gas_object_ref,
            args,
            TEST_ONLY_GAS_UNIT_FOR_OBJECT_BASICS * gas_price,
            gas_price,
        )
        .unwrap(),
        secret,
    )
}

pub async fn do_transaction<A>(authority: &Arc<SafeClient<A>>, transaction: &Transaction)
where
    A: AuthorityAPI + Send + Sync + Clone + 'static,
{
    authority
        .handle_transaction(transaction.clone(), Some(make_socket_addr()))
        .await
        .unwrap();
}

fn make_socket_addr() -> std::net::SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0)
}

pub async fn extract_cert<A>(
    authorities: &[Arc<SafeClient<A>>],
    committee: &Committee,
    transaction_digest: &TransactionDigest,
) -> CertifiedTransaction
where
    A: AuthorityAPI + Send + Sync + Clone + 'static,
{
    let mut votes = vec![];
    let mut tx_data: Option<SenderSignedData> = None;
    for authority in authorities {
        let response = authority
            .handle_transaction_info_request(TransactionInfoRequest {
                transaction_digest: *transaction_digest,
            })
            .await;
        match response {
            Ok(PlainTransactionInfoResponse::Signed(signed)) => {
                let (data, sig) = signed.into_data_and_sig();
                votes.push(sig);
                if let Some(inner_transaction) = tx_data {
                    assert_eq!(
                        inner_transaction.intent_message().value,
                        data.intent_message().value
                    );
                }
                tx_data = Some(data);
            }
            Ok(PlainTransactionInfoResponse::ExecutedWithCert(cert, _, _)) => {
                return cert;
            }
            _ => {}
        }
    }

    CertifiedTransaction::new(tx_data.unwrap(), votes, committee).unwrap()
}

pub async fn do_cert<A>(
    authority: &SafeClient<A>,
    cert: &CertifiedTransaction,
) -> TransactionEffects
where
    A: AuthorityAPI + Send + Sync + Clone + 'static,
{
    authority
        .handle_certificate_v2(cert.clone(), Some(make_socket_addr()))
        .await
        .unwrap()
        .signed_effects
        .into_data()
}

pub async fn do_cert_configurable<A>(authority: &A, cert: &CertifiedTransaction)
where
    A: AuthorityAPI + Send + Sync + Clone + 'static,
{
    let result = authority
        .handle_certificate_v2(cert.clone(), Some(make_socket_addr()))
        .await;
    if result.is_err() {
        println!("Error in do cert {:?}", result.err());
    }
}

pub async fn get_latest_ref<A>(authority: Arc<SafeClient<A>>, object_id: ObjectID) -> ObjectRef
where
    A: AuthorityAPI + Send + Sync + Clone + 'static,
{
    if let Ok(VerifiedObjectInfoResponse { object }) = authority
        .handle_object_info_request(ObjectInfoRequest::latest_object_info_request(
            object_id,
            LayoutGenerationOption::None,
        ))
        .await
    {
        return object.compute_object_reference();
    }
    panic!("Object not found!");
}
