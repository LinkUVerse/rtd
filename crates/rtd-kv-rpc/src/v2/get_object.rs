// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use futures::StreamExt;
use rtd_kvstore::{BigTableClient, KeyValueStoreReader};
use rtd_rpc::proto::rtd::rpc::v2::BatchGetObjectsRequest;
use rtd_rpc::proto::rtd::rpc::v2::BatchGetObjectsResponse;
use rtd_rpc::proto::rtd::rpc::v2::{GetObjectRequest, GetObjectResponse, GetObjectResult};
use rtd_rpc_api::{
    ObjectNotFoundError, RpcError, grpc::v2::ledger_service::validate_get_object_requests,
};
use rtd_types::base_types::{ObjectID, SequenceNumber};
use rtd_types::storage::ObjectKey;
use std::collections::HashMap;

use crate::PackageResolver;
use crate::render::object_to_response;

pub const MAX_BATCH_REQUESTS: usize = 1000;
pub const MAX_CONCURRENT_UNVERSIONED_SCANS: usize = 50;
pub(crate) async fn get_object(
    mut client: BigTableClient,
    GetObjectRequest {
        object_id,
        version,
        read_mask,
        ..
    }: GetObjectRequest,
    resolver: &PackageResolver,
    admission: Option<u64>,
) -> Result<GetObjectResponse, RpcError> {
    let (requests, read_mask) =
        validate_get_object_requests(vec![(object_id, version)], read_mask)?;
    let (object_id, version) = requests[0];
    let object = if let Some(height) = admission {
        client
            .get_object_at_checkpoint(object_id.into(), version, height)
            .await?
            .ok_or_else(|| ObjectNotFoundError::new(object_id))?
    } else {
        match version {
            Some(version) => client
                .get_objects(&[ObjectKey(object_id.into(), version.into())])
                .await?
                .pop()
                .ok_or_else(|| ObjectNotFoundError::new_with_version(object_id, version))?,
            None => client
                .get_latest_object(&object_id.into())
                .await?
                .ok_or_else(|| ObjectNotFoundError::new(object_id))?,
        }
    };
    let message = object_to_response(&object, &read_mask, resolver).await?;
    Ok(GetObjectResponse::new(message))
}

pub(crate) async fn batch_get_objects(
    client: BigTableClient,
    BatchGetObjectsRequest {
        requests,
        read_mask,
        ..
    }: BatchGetObjectsRequest,
    resolver: &PackageResolver,
    admission: Option<u64>,
) -> Result<BatchGetObjectsResponse, RpcError> {
    if requests.len() > MAX_BATCH_REQUESTS {
        return Err(RpcError::new(
            tonic::Code::InvalidArgument,
            format!("number of batch requests exceed limit of {MAX_BATCH_REQUESTS}"),
        ));
    }
    let requests = requests
        .into_iter()
        .map(|req| (req.object_id, req.version))
        .collect();
    let (requests, read_mask) = validate_get_object_requests(requests, read_mask)?;

    if let Some(height) = admission {
        let futures = requests.into_iter().map(|(address, version)| {
            let mut reader = client.clone();
            let mask = &read_mask;
            async move {
                match reader
                    .get_object_at_checkpoint(address.into(), version, height)
                    .await
                {
                    Ok(Some(object)) => match object_to_response(&object, mask, resolver).await {
                        Ok(message) => GetObjectResult::new_object(message),
                        Err(error) => GetObjectResult::new_error(error.into_status_proto()),
                    },
                    Ok(None) => {
                        let error: RpcError = ObjectNotFoundError::new(address).into();
                        GetObjectResult::new_error(error.into_status_proto())
                    }
                    Err(error) => {
                        GetObjectResult::new_error(RpcError::from(error).into_status_proto())
                    }
                }
            }
        });
        let objects = futures::stream::iter(futures)
            .buffered(MAX_CONCURRENT_UNVERSIONED_SCANS)
            .collect()
            .await;
        return Ok(BatchGetObjectsResponse::new(objects));
    }

    let mut exact_object_keys = Vec::new();
    let mut unversioned_object_ids = Vec::new();
    for (address, version) in &requests {
        let object_id: ObjectID = (*address).into();
        match version {
            Some(version) => {
                let sequence_number: SequenceNumber = (*version).into();
                exact_object_keys.push(ObjectKey(object_id, sequence_number));
            }
            None => {
                unversioned_object_ids.push(object_id);
            }
        }
    }

    exact_object_keys.sort();
    exact_object_keys.dedup();
    unversioned_object_ids.sort();
    unversioned_object_ids.dedup();

    let exact_future = async {
        if exact_object_keys.is_empty() {
            Ok(HashMap::new())
        } else {
            let mut client = client.clone();
            let objects = client.get_objects(&exact_object_keys).await?;
            Ok(objects
                .into_iter()
                .map(|obj| ((obj.id(), obj.version()), obj))
                .collect())
        }
    };

    let unversioned_future = async {
        let unversioned_futures = unversioned_object_ids.into_iter().map(|object_id| {
            let mut client = client.clone();
            async move {
                let object = client.get_latest_object(&object_id).await?;
                Ok::<_, RpcError>((object_id, object))
            }
        });
        let unversioned_results = futures::stream::iter(unversioned_futures)
            .buffer_unordered(MAX_CONCURRENT_UNVERSIONED_SCANS)
            .collect::<Vec<_>>()
            .await;

        let mut unversioned_objects = HashMap::new();
        for result in unversioned_results {
            let (object_id, object) = result?;
            if let Some(object) = object {
                unversioned_objects.insert(object_id, object);
            }
        }
        Ok::<_, RpcError>(unversioned_objects)
    };

    let (exact_objects, unversioned_objects) = tokio::try_join!(exact_future, unversioned_future)?;

    let mut objects = Vec::with_capacity(requests.len());
    for (address, version) in requests {
        let object_id: ObjectID = address.into();
        match version {
            Some(version) => {
                let sequence_number: SequenceNumber = version.into();
                if let Some(object) = exact_objects.get(&(object_id, sequence_number)) {
                    objects.push(
                        match object_to_response(object, &read_mask, resolver).await {
                            Ok(message) => GetObjectResult::new_object(message),
                            Err(error) => GetObjectResult::new_error(error.into_status_proto()),
                        },
                    );
                } else {
                    let err: RpcError =
                        ObjectNotFoundError::new_with_version(address, version).into();
                    objects.push(GetObjectResult::new_error(err.into_status_proto()));
                }
            }
            None => {
                if let Some(object) = unversioned_objects.get(&object_id) {
                    objects.push(
                        match object_to_response(object, &read_mask, resolver).await {
                            Ok(message) => GetObjectResult::new_object(message),
                            Err(error) => GetObjectResult::new_error(error.into_status_proto()),
                        },
                    );
                } else {
                    let err: RpcError = ObjectNotFoundError::new(address).into();
                    objects.push(GetObjectResult::new_error(err.into_status_proto()));
                }
            }
        }
    }
    Ok(BatchGetObjectsResponse::new(objects))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rtd_kvstore::BigTableClient as InnerBigTableClient;
    use rtd_kvstore::testing::MockBigtableServer;
    use rtd_package_resolver::PackageStore;
    use rtd_package_resolver::Resolver;
    use rtd_types::base_types::ObjectID;

    use super::*;
    use crate::package_store::BigTablePackageStore;

    #[tokio::test]
    async fn admission_scans_past_many_future_versions_and_checks_effects() {
        use bytes::Bytes;
        use rtd_types::digests::TransactionDigest;
        use rtd_types::effects::TransactionEffects;
        use rtd_types::object::{Object, Owner};
        let mock = MockBigtableServer::new();
        let (address, handle) = mock.start().await.unwrap();
        let mut client = InnerBigTableClient::new_local(address.to_string(), "test".to_string())
            .await
            .unwrap();
        let id = ObjectID::random();
        let mut versions = Vec::new();
        for index in 0..66u64 {
            let digest = TransactionDigest::new([index as u8 + 1; 32]);
            let mut inner =
                Object::with_id_owner_version_for_testing(id, (index + 1).into(), Owner::Immutable)
                    .into_inner();
            inner.previous_transaction = digest;
            let object = Object::from(inner);
            let reference = object.compute_object_reference();
            let effects = TransactionEffects::new_from_execution_v1(
                rtd_types::execution_status::ExecutionStatus::Success,
                0,
                Default::default(),
                vec![],
                vec![],
                digest,
                vec![(reference, Owner::Immutable)],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                (reference, Owner::Immutable),
                None,
                vec![],
            );
            mock.insert_row(
                rtd_kvstore::tables::objects::NAME,
                rtd_kvstore::tables::objects::encode_key(&ObjectKey(id, object.version())),
                rtd_kvstore::tables::objects::encode(&object).unwrap(),
            )
            .await;
            mock.insert_row(
                rtd_kvstore::tables::transactions::NAME,
                rtd_kvstore::tables::transactions::encode_key(&digest),
                [
                    ("cn", Bytes::from(bcs::to_bytes(&(index + 5)).unwrap())),
                    ("ef", Bytes::from(bcs::to_bytes(&effects).unwrap())),
                ],
            )
            .await;
            versions.push(object);
        }
        let original = versions.first().unwrap().clone();
        let latest = versions.last().unwrap().clone();
        assert!(latest.version() > original.version());
        let historical = client
            .get_object_at_checkpoint(id, None, 5)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            historical.compute_object_reference(),
            original.compute_object_reference()
        );
        assert!(
            client
                .get_object_at_checkpoint(id, Some(latest.version().value()), 5)
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(
            client
                .get_object_at_checkpoint(id, Some(original.version().value()), 5)
                .await
                .unwrap()
                .unwrap()
                .compute_object_reference(),
            original.compute_object_reference()
        );
        assert!(
            client
                .get_object_at_checkpoint(id, None, 4)
                .await
                .unwrap()
                .is_none()
        );
        // A row key alone cannot authenticate an altered object body.
        let mut changed = original.clone().into_inner();
        changed.previous_transaction = latest.previous_transaction;
        let changed = rtd_types::object::Object::from(changed);
        mock.insert_row(
            rtd_kvstore::tables::objects::NAME,
            rtd_kvstore::tables::objects::encode_key(&ObjectKey(id, changed.version())),
            rtd_kvstore::tables::objects::encode(&changed).unwrap(),
        )
        .await;
        assert!(
            client
                .get_object_at_checkpoint(id, Some(changed.version().value()), 100)
                .await
                .is_err()
        );
        handle.abort();
    }

    #[tokio::test]
    async fn requested_object_json_returns_per_item_errors_and_recovers() {
        use crate::v2::test_utils::{
            framework_objects_for_json_tests, two_transaction_object_checkpoint,
        };
        use rtd_rpc::field::{FieldMask, FieldMaskUtil};
        let mock = MockBigtableServer::new();
        let (addr, server) = mock.start().await.unwrap();
        let client = InnerBigTableClient::new_local(addr.to_string(), "test".to_owned())
            .await
            .unwrap();
        let checkpoint = two_transaction_object_checkpoint();
        let object = checkpoint
            .object_set
            .iter()
            .find(|object| object.data.try_as_move().is_some())
            .unwrap();
        let packages = framework_objects_for_json_tests();
        let package = &packages[0];
        for object in [object, package] {
            mock.insert_row(
                rtd_kvstore::tables::objects::NAME,
                rtd_kvstore::tables::objects::encode_key(&ObjectKey(object.id(), object.version())),
                rtd_kvstore::tables::objects::encode(object).unwrap(),
            )
            .await;
        }
        let store: Arc<dyn PackageStore> = Arc::new(BigTablePackageStore::new(client.clone()));
        let resolver = Arc::new(Resolver::new(store));
        let mut request = BatchGetObjectsRequest::default();
        request.read_mask = Some(FieldMask::from_paths(["object_id", "json"]));
        request.requests = [object, package]
            .into_iter()
            .map(|object| {
                let mut item = GetObjectRequest::default();
                item.object_id = Some(object.id().to_string());
                item.version = Some(object.version().value());
                item
            })
            .collect();
        let response = batch_get_objects(client.clone(), request.clone(), &resolver, None)
            .await
            .unwrap();
        assert_eq!(
            response.objects[0].clone().to_result().unwrap_err().code,
            tonic::Code::Unavailable as i32
        );
        assert!(response.objects[1].clone().to_result().is_ok());
        let mut single = request.requests[0].clone();
        single.read_mask = request.read_mask.clone();
        let error = get_object(client.clone(), single.clone(), &resolver, None)
            .await
            .unwrap_err();
        assert_eq!(tonic::Status::from(error).code(), tonic::Code::Unavailable);
        single.read_mask = Some(FieldMask::from_paths(["object_id"]));
        assert!(
            get_object(client.clone(), single, &resolver, None)
                .await
                .is_ok()
        );
        for package in packages {
            mock.insert_row(
                rtd_kvstore::tables::objects::NAME,
                rtd_kvstore::tables::objects::encode_key(&ObjectKey(
                    package.id(),
                    package.version(),
                )),
                rtd_kvstore::tables::objects::encode(&package).unwrap(),
            )
            .await;
        }
        let recovered = batch_get_objects(client, request, &resolver, None)
            .await
            .unwrap();
        assert!(
            recovered.objects[0]
                .clone()
                .to_result()
                .unwrap()
                .json
                .is_some()
        );
        assert!(recovered.objects[1].clone().to_result().is_ok());
        server.abort();
    }

    #[tokio::test]
    async fn test_batch_get_objects_limit_exceeded() {
        let mock = MockBigtableServer::new();
        let (addr, server) = mock.start().await.expect("start mock BigTable");
        let client = InnerBigTableClient::new_local(addr.to_string(), "test".to_string())
            .await
            .expect("connect to mock BigTable");
        let package_store: Arc<dyn PackageStore> =
            Arc::new(BigTablePackageStore::new(client.clone()));
        let resolver = Arc::new(Resolver::new(package_store));

        let requests = (0..MAX_BATCH_REQUESTS + 1)
            .map(|_| {
                let mut req = GetObjectRequest::default();
                req.object_id = Some(ObjectID::random().to_canonical_string(true));
                req.version = Some(1);
                req
            })
            .collect();

        let mut req = BatchGetObjectsRequest::default();
        req.requests = requests;

        let err = batch_get_objects(client, req, &resolver, None)
            .await
            .unwrap_err();
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);

        server.abort();
    }
}
