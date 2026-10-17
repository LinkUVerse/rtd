// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use rtd_kvstore::BigTableClient;
use rtd_kvstore::KeyValueStoreReader;
use rtd_rpc::proto::google::rpc::bad_request::FieldViolation;
use rtd_rpc::proto::rtd::rpc::v2::GetDatatypeRequest;
use rtd_rpc::proto::rtd::rpc::v2::GetDatatypeResponse;
use rtd_rpc::proto::rtd::rpc::v2::GetFunctionRequest;
use rtd_rpc::proto::rtd::rpc::v2::GetFunctionResponse;
use rtd_rpc::proto::rtd::rpc::v2::GetPackageRequest;
use rtd_rpc::proto::rtd::rpc::v2::GetPackageResponse;
use rtd_rpc::proto::rtd::rpc::v2::ListPackageVersionsRequest;
use rtd_rpc::proto::rtd::rpc::v2::ListPackageVersionsResponse;
use rtd_rpc::proto::rtd::rpc::v2::PackageVersion;
use rtd_rpc::proto::rtd::rpc::v2::get_package_request::Selector;
use rtd_rpc::proto::rtd::rpc::v2::move_package_service_server::MovePackageService;
use rtd_rpc_api::ErrorReason;
use rtd_rpc_api::RpcError;
use rtd_rpc_api::grpc::v2::move_package_service::PageToken;
use rtd_rpc_api::grpc::v2::move_package_service::get_datatype_response;
use rtd_rpc_api::grpc::v2::move_package_service::get_function_response;
use rtd_rpc_api::grpc::v2::move_package_service::get_package_response;
use rtd_types::base_types::ObjectID;
use rtd_types::move_package::MovePackage;

use crate::KvRpcServer;

#[tonic::async_trait]
impl MovePackageService for KvRpcServer {
    async fn get_package(
        &self,
        request: tonic::Request<GetPackageRequest>,
    ) -> Result<tonic::Response<GetPackageResponse>, tonic::Status> {
        let mut admission = if let Some(height) =
            crate::operation::admission_ceiling(request.metadata())?
        {
            let info = super::get_admitted_service_info(
                self.client.clone(),
                self.chain_id,
                self.server_version.clone(),
                &self.service_info_watermark_pipelines,
                height,
            )
            .await?;
            Some(
                info.checkpoint_height
                    .ok_or_else(|| tonic::Status::unavailable("Missing admission checkpoint"))?,
            )
        } else {
            None
        };
        if request.get_ref().selector.is_some() && admission.is_none() {
            let info = super::get_service_info(
                self.client.clone(),
                self.chain_id,
                self.server_version.clone(),
                &self.service_info_watermark_pipelines,
            )
            .await
            .map_err(tonic::Status::from)?;
            admission = Some(
                info.checkpoint_height
                    .ok_or_else(|| tonic::Status::unavailable("Missing indexed checkpoint"))?,
            );
        }
        get_package(self.client.clone(), request.into_inner(), admission)
            .await
            .map(tonic::Response::new)
            .map_err(Into::into)
    }

    async fn get_datatype(
        &self,
        request: tonic::Request<GetDatatypeRequest>,
    ) -> Result<tonic::Response<GetDatatypeResponse>, tonic::Status> {
        let admission = if let Some(height) =
            crate::operation::admission_ceiling(request.metadata())?
        {
            let info = super::get_admitted_service_info(
                self.client.clone(),
                self.chain_id,
                self.server_version.clone(),
                &self.service_info_watermark_pipelines,
                height,
            )
            .await?;
            Some(
                info.checkpoint_height
                    .ok_or_else(|| tonic::Status::unavailable("Missing admission checkpoint"))?,
            )
        } else {
            None
        };
        get_datatype(self.client.clone(), request.into_inner(), admission)
            .await
            .map(tonic::Response::new)
            .map_err(Into::into)
    }

    async fn get_function(
        &self,
        request: tonic::Request<GetFunctionRequest>,
    ) -> Result<tonic::Response<GetFunctionResponse>, tonic::Status> {
        let admission = if let Some(height) =
            crate::operation::admission_ceiling(request.metadata())?
        {
            let info = super::get_admitted_service_info(
                self.client.clone(),
                self.chain_id,
                self.server_version.clone(),
                &self.service_info_watermark_pipelines,
                height,
            )
            .await?;
            Some(
                info.checkpoint_height
                    .ok_or_else(|| tonic::Status::unavailable("Missing admission checkpoint"))?,
            )
        } else {
            None
        };
        get_function(self.client.clone(), request.into_inner(), admission)
            .await
            .map(tonic::Response::new)
            .map_err(Into::into)
    }

    async fn list_package_versions(
        &self,
        request: tonic::Request<ListPackageVersionsRequest>,
    ) -> Result<tonic::Response<ListPackageVersionsResponse>, tonic::Status> {
        let admission = if let Some(height) =
            crate::operation::admission_ceiling(request.metadata())?
        {
            let info = super::get_admitted_service_info(
                self.client.clone(),
                self.chain_id,
                self.server_version.clone(),
                &self.service_info_watermark_pipelines,
                height,
            )
            .await?;
            Some(
                info.checkpoint_height
                    .ok_or_else(|| tonic::Status::unavailable("Missing admission checkpoint"))?,
            )
        } else {
            None
        };
        list_package_versions(self.client.clone(), request.into_inner(), admission)
            .await
            .map(tonic::Response::new)
            .map_err(Into::into)
    }
}

async fn get_package(
    client: BigTableClient,
    request: GetPackageRequest,
    admission: Option<u64>,
) -> Result<GetPackageResponse, RpcError> {
    let package_id_str = request.package_id.as_ref().ok_or_else(|| {
        FieldViolation::new("package_id")
            .with_description("missing package_id")
            .with_reason(ErrorReason::FieldMissing)
    })?;

    let id = parse_package_id(package_id_str)?;
    let package = match request.selector {
        None => load_package(client, id, admission).await?,
        Some(selector) => {
            load_selected_package(
                client,
                id,
                selector,
                admission.ok_or_else(|| {
                    RpcError::new(tonic::Code::Unavailable, "Missing selection boundary")
                })?,
            )
            .await?
        }
    };
    get_package_response(&package)
}

async fn load_selected_package(
    mut client: BigTableClient,
    anchor: ObjectID,
    selector: Selector,
    ceiling: u64,
) -> Result<MovePackage, RpcError> {
    let original = resolve_original_package_id(client.clone(), anchor, None).await?;
    let metadata = match selector {
        Selector::Version(version) => client
            .get_packages_by_version(&[(original, version)])
            .await?
            .pop(),
        Selector::AtCheckpoint(height) => {
            if height > ceiling {
                return Err(RpcError::not_found());
            }
            client.get_package_latest(original, height).await?
        }
        _ => {
            return Err(RpcError::new(
                tonic::Code::InvalidArgument,
                "Unsupported package selector",
            ));
        }
    }
    .ok_or_else(RpcError::not_found)?;
    if metadata.cp_sequence_number > ceiling {
        return Err(RpcError::not_found());
    }
    let selection_matches = match selector {
        Selector::Version(version) => metadata.package_version == version,
        Selector::AtCheckpoint(height) => metadata.cp_sequence_number <= height,
        _ => false,
    };
    if metadata.original_id.as_slice() != original.as_ref() || !selection_matches {
        return Err(RpcError::new(
            tonic::Code::Unavailable,
            "Selected package metadata mismatch",
        ));
    }
    let storage = ObjectID::from_bytes(&metadata.package_id)
        .map_err(|_| RpcError::new(tonic::Code::Unavailable, "Selected package ID invalid"))?;
    // System packages share a storage ID across versions. Read the exact object key.
    let key = rtd_types::storage::ObjectKey(storage, metadata.package_version.into());
    let object = client.get_objects(&[key]).await?.pop().ok_or_else(|| {
        RpcError::new(
            tonic::Code::Unavailable,
            "Selected package object unavailable",
        )
    })?;
    let package = object.into_inner().data.try_into_package().ok_or_else(|| {
        RpcError::new(tonic::Code::Unavailable, "Selected object is not a package")
    })?;
    if package.id() != storage
        || package.version().value() != metadata.package_version
        || package.original_package_id() != original
    {
        return Err(RpcError::new(
            tonic::Code::Unavailable,
            "Selected package content mismatch",
        ));
    }
    Ok(package)
}

async fn get_datatype(
    client: BigTableClient,
    request: GetDatatypeRequest,
    admission: Option<u64>,
) -> Result<GetDatatypeResponse, RpcError> {
    let package_id_str = request.package_id.as_ref().ok_or_else(|| {
        FieldViolation::new("package_id")
            .with_description("missing package_id")
            .with_reason(ErrorReason::FieldMissing)
    })?;

    let module_name = request.module_name.as_ref().ok_or_else(|| {
        FieldViolation::new("module_name")
            .with_description("missing module_name")
            .with_reason(ErrorReason::FieldMissing)
    })?;

    let datatype_name = request.name.as_ref().ok_or_else(|| {
        FieldViolation::new("name")
            .with_description("missing name")
            .with_reason(ErrorReason::FieldMissing)
    })?;

    let package = load_package(client, parse_package_id(package_id_str)?, admission).await?;
    get_datatype_response(&package, module_name, datatype_name)
}

async fn get_function(
    client: BigTableClient,
    request: GetFunctionRequest,
    admission: Option<u64>,
) -> Result<GetFunctionResponse, RpcError> {
    let package_id_str = request.package_id.as_ref().ok_or_else(|| {
        FieldViolation::new("package_id")
            .with_description("missing package_id")
            .with_reason(ErrorReason::FieldMissing)
    })?;

    let module_name = request.module_name.as_ref().ok_or_else(|| {
        FieldViolation::new("module_name")
            .with_description("missing module_name")
            .with_reason(ErrorReason::FieldMissing)
    })?;

    let function_name = request.name.as_ref().ok_or_else(|| {
        FieldViolation::new("name")
            .with_description("missing name")
            .with_reason(ErrorReason::FieldMissing)
    })?;

    let package = load_package(client, parse_package_id(package_id_str)?, admission).await?;
    get_function_response(&package, module_name, function_name)
}

async fn list_package_versions(
    mut client: BigTableClient,
    request: ListPackageVersionsRequest,
    admission: Option<u64>,
) -> Result<ListPackageVersionsResponse, RpcError> {
    let package_id_str = request.package_id.as_ref().ok_or_else(|| {
        FieldViolation::new("package_id")
            .with_description("missing package_id")
            .with_reason(ErrorReason::FieldMissing)
    })?;
    let original_package_id =
        resolve_original_package_id(client.clone(), parse_package_id(package_id_str)?, admission)
            .await?;

    let page_size = request
        .page_size
        .map(|s| (s as usize).clamp(1, 10000))
        .unwrap_or(1000);

    let page_token = request
        .page_token
        .map(|token| PageToken::decode(&token))
        .transpose()?;

    if let Some(token) = &page_token
        && token.original_package_id != original_package_id
    {
        return Err(FieldViolation::new("page_token")
            .with_description("page token package ID does not match request package ID")
            .with_reason(ErrorReason::FieldInvalid)
            .into());
    }

    // The token's version is the first version of the next page (inclusive); the reader's
    // `after_version` bound is exclusive.
    let after_version = page_token.map(|token| token.version.saturating_sub(1));

    let mut versions: Vec<_> = client
        .get_package_versions(
            original_package_id,
            admission.unwrap_or(u64::MAX),
            after_version,
            None,
            page_size + 1,
            false,
        )
        .await
        .map_err(RpcError::from)?
        .into_iter()
        .map(|pkg| {
            let storage_id = ObjectID::from_bytes(&pkg.package_id)
                .map_err(|e| RpcError::new(tonic::Code::Internal, e.to_string()))?;
            Ok(PackageVersion::new(&storage_id.into(), pkg.package_version))
        })
        .collect::<Result<_, RpcError>>()?;

    let next_page_token = if versions.len() > page_size {
        versions.pop().and_then(|next| {
            next.version.map(|version| {
                PageToken {
                    original_package_id,
                    version,
                }
                .encode()
            })
        })
    } else {
        None
    };

    Ok(ListPackageVersionsResponse::new(versions, next_page_token))
}

/// Load a package by its storage ID: a storage ID identifies exactly one immutable package
/// version, so the latest object at that ID is the package itself.
pub(crate) async fn load_package(
    mut client: BigTableClient,
    package_id: ObjectID,
    admission: Option<u64>,
) -> Result<MovePackage, RpcError> {
    let object = client
        .get_latest_object(&package_id)
        .await
        .map_err(RpcError::from)?
        .ok_or_else(RpcError::not_found)?;

    let package =
        object.into_inner().data.try_into_package().ok_or_else(|| {
            RpcError::new(tonic::Code::InvalidArgument, "object is not a package")
        })?;
    let Some(height) = admission else {
        return Ok(package);
    };
    let original = package.original_package_id();
    let metadata = if rtd_types::is_system_package(package_id) {
        client.get_package_latest(original, height).await?
    } else {
        client
            .get_packages_by_version(&[(original, package.version().value())])
            .await?
            .pop()
    }
    .ok_or_else(|| {
        RpcError::new(
            tonic::Code::Unavailable,
            "Package admission metadata unavailable",
        )
    })?;
    if metadata.cp_sequence_number > height {
        return Err(RpcError::new(
            tonic::Code::FailedPrecondition,
            "Package exceeds publication admission boundary",
        ));
    }
    if metadata.package_id.as_slice() != package_id.as_ref()
        || metadata.original_id.as_slice() != original.as_ref()
    {
        return Err(RpcError::new(
            tonic::Code::Unavailable,
            "Package admission identity mismatch",
        ));
    }
    let key = rtd_types::storage::ObjectKey(package_id, metadata.package_version.into());
    let object = client.get_objects(&[key]).await?.pop().ok_or_else(|| {
        RpcError::new(
            tonic::Code::Unavailable,
            "Admitted package object unavailable",
        )
    })?;
    let package = object.into_inner().data.try_into_package().ok_or_else(|| {
        RpcError::new(tonic::Code::Unavailable, "Admitted object is not a package")
    })?;
    if package.id() != package_id
        || package.version().value() != metadata.package_version
        || package.original_package_id() != original
    {
        return Err(RpcError::new(
            tonic::Code::Unavailable,
            "Admitted package content mismatch",
        ));
    }
    Ok(package)
}

/// Resolve the original (first-version) package ID for a storage ID. `packages_by_id` answers
/// with a 32-byte point lookup; on a miss the object itself is read so the error matches what a
/// full node returns (`NotFound` vs "object is not a package"), and so a package whose
/// `packages_by_id` row is not yet written is still served.
async fn resolve_original_package_id(
    mut client: BigTableClient,
    package_id: ObjectID,
    admission: Option<u64>,
) -> Result<ObjectID, RpcError> {
    if admission.is_some() {
        return load_package(client, package_id, admission)
            .await
            .map(|package| package.original_package_id());
    }
    let pairs = client
        .get_package_original_ids(&[package_id])
        .await
        .map_err(RpcError::from)?;
    if let Some((_, original_id)) = pairs
        .iter()
        .find(|(storage_id, _)| *storage_id == package_id)
    {
        return Ok(*original_id);
    }

    load_package(client, package_id, None)
        .await
        .map(|package| package.original_package_id())
}

fn parse_package_id(package_id_str: &str) -> Result<ObjectID, RpcError> {
    package_id_str.parse::<ObjectID>().map_err(|e| {
        FieldViolation::new("package_id")
            .with_description(format!("invalid package_id: {}", e))
            .with_reason(ErrorReason::FieldInvalid)
            .into()
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use rtd_kvstore::tables;
    use rtd_kvstore::testing::MockBigtableServer;
    use rtd_types::base_types::{ObjectID, SequenceNumber};
    use rtd_types::digests::TransactionDigest;
    use rtd_types::move_package::MovePackage;
    use rtd_types::object::{Data, Object};
    use rtd_types::storage::ObjectKey;

    use super::*;

    async fn insert_package(
        mock: &MockBigtableServer,
        id: ObjectID,
        version: u64,
        checkpoint: u64,
        system: bool,
    ) {
        insert_package_lineage(mock, id, id, version, checkpoint, system).await;
    }

    async fn insert_package_lineage(
        mock: &MockBigtableServer,
        id: ObjectID,
        original: ObjectID,
        version: u64,
        checkpoint: u64,
        system: bool,
    ) {
        let mut modules = BTreeMap::new();
        {
            let seed_bytes = include_bytes!(
                "../../../rtd-single-node-benchmark/tests/data/package_publish_from_bytecode/package_a/build/a/bytecode_modules/a.mv"
            );
            let seed = MovePackage::new(
                id,
                1.into(),
                BTreeMap::from([("a".to_string(), seed_bytes.to_vec())]),
                100_000,
                vec![],
                BTreeMap::new(),
            )
            .unwrap();
            let mut module = seed
                .deserialize_module_by_str(
                    "a",
                    &rtd_protocol_config::ProtocolConfig::get_for_max_version_UNSAFE()
                        .binary_config(Some(false)),
                )
                .unwrap();
            module.address_identifiers[0] = original.into();
            let mut bytes = Vec::new();
            module
                .serialize_with_version(module.version, &mut bytes)
                .unwrap();
            modules.insert("a".to_string(), bytes);
        }
        let package = MovePackage::new(
            id,
            version.into(),
            modules,
            100_000,
            vec![],
            BTreeMap::new(),
        )
        .unwrap();
        let object = Object::new_package_from_data(Data::Package(package), TransactionDigest::ZERO);
        mock.insert_row(
            tables::objects::NAME,
            tables::objects::encode_key(&ObjectKey(id, version.into())),
            tables::objects::encode(&object).unwrap(),
        )
        .await;
        mock.insert_row(
            tables::packages::NAME,
            tables::packages::encode_key(original.as_ref(), version),
            tables::packages::encode(checkpoint, id.as_ref(), system),
        )
        .await;
    }

    #[tokio::test]
    async fn package_selectors_resolve_upgrade_lineage_and_exact_bounds() {
        let mock = MockBigtableServer::new();
        let (addr, _handle) = mock.start().await.unwrap();
        let client = BigTableClient::new_local(addr.to_string(), "test".to_string())
            .await
            .unwrap();
        let original = ObjectID::random();
        let upgraded = ObjectID::random();
        insert_package_lineage(&mock, original, original, 1, 10, false).await;
        insert_package_lineage(&mock, upgraded, original, 2, 20, false).await;
        let first = load_selected_package(client.clone(), upgraded, Selector::Version(1), 20)
            .await
            .unwrap();
        assert_eq!(first.id(), original);
        let second =
            load_selected_package(client.clone(), original, Selector::AtCheckpoint(20), 20)
                .await
                .unwrap();
        assert_eq!(second.id(), upgraded);
        assert_eq!(second.version().value(), 2);
        let historical =
            load_selected_package(client.clone(), upgraded, Selector::AtCheckpoint(19), 20)
                .await
                .unwrap();
        assert_eq!(historical.version().value(), 1);
        for (selector, bound) in [
            (Selector::Version(999999), 20),
            (Selector::AtCheckpoint(0), 20),
            (Selector::AtCheckpoint(21), 20),
            (Selector::Version(2), 19),
        ] {
            let error: tonic::Status =
                load_selected_package(client.clone(), original, selector, bound)
                    .await
                    .unwrap_err()
                    .into();
            assert_eq!(error.code(), tonic::Code::NotFound);
        }
    }

    #[tokio::test]
    async fn package_selectors_read_exact_system_version() {
        let mock = MockBigtableServer::new();
        let (addr, _handle) = mock.start().await.unwrap();
        let client = BigTableClient::new_local(addr.to_string(), "test".to_string())
            .await
            .unwrap();
        let id = ObjectID::from_hex_literal("0x2").unwrap();
        for (version, height) in [(1, 0), (2, 10), (3, 20)] {
            insert_package(&mock, id, version, height, true).await;
        }
        let first = load_selected_package(client.clone(), id, Selector::Version(1), 20)
            .await
            .unwrap();
        assert_eq!(first.version().value(), 1);
        let historical = load_selected_package(client, id, Selector::AtCheckpoint(10), 20)
            .await
            .unwrap();
        assert_eq!(historical.version().value(), 2);
    }

    #[tokio::test]
    async fn package_selectors_reject_missing_or_conflicting_objects() {
        let mock = MockBigtableServer::new();
        let (addr, _handle) = mock.start().await.unwrap();
        let client = BigTableClient::new_local(addr.to_string(), "test".to_string())
            .await
            .unwrap();
        let original = ObjectID::random();
        let foreign = ObjectID::random();
        insert_package(&mock, original, 1, 10, false).await;
        insert_package(&mock, foreign, 2, 20, false).await;
        // The index claims the foreign object belongs to this lineage.
        mock.insert_row(
            tables::packages::NAME,
            tables::packages::encode_key(original.as_ref(), 2),
            tables::packages::encode(20, foreign.as_ref(), false),
        )
        .await;
        let wrong: tonic::Status =
            load_selected_package(client.clone(), original, Selector::Version(2), 20)
                .await
                .unwrap_err()
                .into();
        assert_eq!(wrong.code(), tonic::Code::Unavailable);
        let missing = ObjectID::random();
        mock.insert_row(
            tables::packages::NAME,
            tables::packages::encode_key(original.as_ref(), 3),
            tables::packages::encode(20, missing.as_ref(), false),
        )
        .await;
        let absent: tonic::Status =
            load_selected_package(client, original, Selector::Version(3), 20)
                .await
                .unwrap_err()
                .into();
        assert_eq!(absent.code(), tonic::Code::Unavailable);
    }

    #[tokio::test]
    async fn admission_resolver_caches_are_private_to_each_request() {
        use std::sync::Arc;

        let mock = MockBigtableServer::new();
        let (addr, handle) = mock.start().await.unwrap();
        let server = crate::KvRpcServer::new_local(addr.to_string(), "test".to_string(), None)
            .await
            .unwrap();
        let id = ObjectID::random();
        insert_package(&mock, id, 1, 10, false).await;

        // Warm both the legacy shared cache and a higher admitted request first.
        let legacy = server.request_package_resolver(None);
        let high = server.request_package_resolver(Some(10));
        let shared_package = legacy.package_store().fetch(id.into()).await.unwrap();
        let high_package = high.package_store().fetch(id.into()).await.unwrap();
        assert!(!Arc::ptr_eq(&shared_package, &high_package));
        assert!(Arc::ptr_eq(
            &high_package,
            &high.package_store().fetch(id.into()).await.unwrap()
        ));
        assert!(Arc::ptr_eq(&legacy, &server.request_package_resolver(None)));

        // A lower request must still read and enforce publication metadata.
        let low = server.request_package_resolver(Some(9));
        assert!(low.package_store().fetch(id.into()).await.is_err());
        let independent = server.request_package_resolver(Some(10));
        let independent_package = independent.package_store().fetch(id.into()).await.unwrap();
        assert!(!Arc::ptr_eq(&high_package, &independent_package));
        assert_eq!(high_package.storage_id(), id.into());
        assert!(high_package.modules().contains_key("a"));
        handle.abort();
    }

    #[tokio::test]
    async fn admission_pins_system_package_and_paginated_versions() {
        let mock = MockBigtableServer::new();
        let (addr, handle) = mock.start().await.unwrap();
        let client = BigTableClient::new_local(addr.to_string(), "test".to_string())
            .await
            .unwrap();
        let id = rtd_types::RTD_FRAMEWORK_PACKAGE_ID;
        for (version, height) in [(1, 0), (2, 10), (3, 20)] {
            insert_package(&mock, id, version, height, true).await;
        }
        assert_eq!(
            load_package(client.clone(), id, None)
                .await
                .unwrap()
                .version()
                .value(),
            3
        );
        assert_eq!(
            load_package(client.clone(), id, Some(9))
                .await
                .unwrap()
                .version()
                .value(),
            1
        );
        assert_eq!(
            load_package(client.clone(), id, Some(10))
                .await
                .unwrap()
                .version()
                .value(),
            2
        );
        let mut request = ListPackageVersionsRequest::default();
        request.package_id = Some(id.to_string());
        request.page_size = Some(1);
        let first = list_package_versions(client.clone(), request.clone(), Some(10))
            .await
            .unwrap();
        assert_eq!(first.versions.len(), 1);
        assert_eq!(first.versions[0].version, Some(1));
        request.page_token = first.next_page_token;
        assert!(request.page_token.is_some());
        let second = list_package_versions(client.clone(), request.clone(), Some(10))
            .await
            .unwrap();
        assert_eq!(second.versions.len(), 1);
        assert_eq!(second.versions[0].version, Some(2));
        assert!(second.next_page_token.is_none());
        let lower = list_package_versions(client, request, Some(9))
            .await
            .unwrap();
        assert!(lower.versions.is_empty());
        assert!(lower.next_page_token.is_none());
        handle.abort();
    }

    #[tokio::test]
    async fn admission_rejects_future_package_and_incomplete_metadata() {
        let mock = MockBigtableServer::new();
        let (addr, handle) = mock.start().await.unwrap();
        let client = BigTableClient::new_local(addr.to_string(), "test".to_string())
            .await
            .unwrap();
        let id = ObjectID::random();
        insert_package(&mock, id, 1, 10, false).await;
        let error = load_package(client.clone(), id, Some(9)).await.unwrap_err();
        assert_eq!(
            tonic::Status::from(error).code(),
            tonic::Code::FailedPrecondition
        );
        assert_eq!(load_package(client, id, Some(10)).await.unwrap().id(), id);
        let key = tables::packages::encode_key(id.as_ref(), 1);
        let valid: Vec<_> = tables::packages::encode(0, id.as_ref(), false)
            .into_iter()
            .map(|(name, value)| (bytes::Bytes::from_static(name.as_bytes()), value))
            .collect();
        assert!(tables::packages::decode(&key, &valid).is_ok());
        for absent in [b"cp", b"pi", b"sp"] {
            let filtered: Vec<_> = valid
                .iter()
                .filter(|(name, _)| name.as_ref() != absent)
                .cloned()
                .collect();
            assert!(tables::packages::decode(&key, &filtered).is_err());
        }
        let mut duplicate = valid.clone();
        duplicate.push(valid[0].clone());
        assert!(tables::packages::decode(&key, &duplicate).is_err());
        handle.abort();
    }

    #[tokio::test]
    async fn list_package_versions_falls_back_to_object_when_packages_by_id_missing() {
        let mock = MockBigtableServer::new();
        let (addr, _handle) = mock.start().await.unwrap();
        let client = BigTableClient::new_local(addr.to_string(), "test".to_string())
            .await
            .unwrap();

        let original_id = ObjectID::random();
        let pkg = MovePackage::new(
            original_id,
            SequenceNumber::from(1),
            BTreeMap::new(),
            100_000,
            vec![],
            BTreeMap::new(),
        )
        .unwrap();
        let obj = Object::new_package_from_data(Data::Package(pkg), TransactionDigest::ZERO);

        // Insert package object into `objects` table
        let obj_key = tables::objects::encode_key(&ObjectKey(original_id, SequenceNumber::from(1)));
        let obj_cells = tables::objects::encode(&obj).unwrap();
        mock.insert_row(tables::objects::NAME, obj_key, obj_cells)
            .await;

        // Insert 3 versions into `packages` table: v1 at cp 10, v2 at cp 20, v3 at cp 30
        for (v, cp) in [(1, 10), (2, 20), (3, 30)] {
            let row_key = tables::packages::encode_key(original_id.as_ref(), v);
            let cells = tables::packages::encode(cp, original_id.as_ref(), false);
            mock.insert_row(tables::packages::NAME, row_key, cells)
                .await;
        }

        let mut req = ListPackageVersionsRequest::default();
        req.package_id = Some(original_id.to_string());
        req.page_size = Some(10);
        let resp = list_package_versions(client.clone(), req, None)
            .await
            .unwrap();
        assert_eq!(resp.versions.len(), 3);
        assert_eq!(resp.versions[0].version, Some(1));
        assert_eq!(resp.versions[1].version, Some(2));
        assert_eq!(resp.versions[2].version, Some(3));
        assert!(resp.next_page_token.is_none());
    }

    #[tokio::test]
    async fn list_package_versions_resolves_original_id_without_reading_objects() {
        let mock = MockBigtableServer::new();
        let (addr, _handle) = mock.start().await.unwrap();
        let client = BigTableClient::new_local(addr.to_string(), "test".to_string())
            .await
            .unwrap();

        let storage_id = ObjectID::random();
        let original_id = ObjectID::random();

        // Insert ONLY the `packages_by_id` mapping and the `packages` version rows; no
        // `objects` row exists, so resolution must come from the mapping alone.
        mock.insert_row(
            tables::packages_by_id::NAME,
            tables::packages_by_id::encode_key(storage_id.as_ref()),
            tables::packages_by_id::encode(original_id.as_ref()),
        )
        .await;
        for (v, cp) in [(1, 10), (2, 20), (3, 30)] {
            let row_key = tables::packages::encode_key(original_id.as_ref(), v);
            let cells = tables::packages::encode(cp, original_id.as_ref(), false);
            mock.insert_row(tables::packages::NAME, row_key, cells)
                .await;
        }

        let mut req = ListPackageVersionsRequest::default();
        req.package_id = Some(storage_id.to_string());
        req.page_size = Some(10);
        let resp = list_package_versions(client.clone(), req, None)
            .await
            .unwrap();
        assert_eq!(resp.versions.len(), 3);
        assert_eq!(resp.versions[0].version, Some(1));
        assert_eq!(resp.versions[1].version, Some(2));
        assert_eq!(resp.versions[2].version, Some(3));
        assert!(resp.next_page_token.is_none());

        assert!(
            mock.read_rows_calls()
                .await
                .into_iter()
                .all(|call| call.table != tables::objects::NAME),
            "no objects ReadRows should be issued when packages_by_id has the row"
        );
    }

    #[tokio::test]
    async fn list_package_versions_unknown_package_is_not_found() {
        let mock = MockBigtableServer::new();
        let (addr, _handle) = mock.start().await.unwrap();
        let client = BigTableClient::new_local(addr.to_string(), "test".to_string())
            .await
            .unwrap();

        let mut req = ListPackageVersionsRequest::default();
        req.package_id = Some(ObjectID::random().to_string());
        req.page_size = Some(10);
        let err = list_package_versions(client.clone(), req, None)
            .await
            .unwrap_err();
        let status: tonic::Status = err.into();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }
}
