// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use move_core_types::account_address::AccountAddress;
use rtd_kvstore::{BigTableClient, KeyValueStoreReader};
use rtd_package_resolver::Package;
use rtd_package_resolver::PackageStore;
use rtd_package_resolver::error::Error;
use rtd_types::base_types::ObjectID;

const STORE: &str = "BigTable";

pub struct BigTablePackageStore {
    client: BigTableClient,
    admission: Option<u64>,
}

impl BigTablePackageStore {
    pub fn new(client: BigTableClient) -> Self {
        Self {
            client,
            admission: None,
        }
    }

    pub(crate) fn admitted(client: BigTableClient, height: u64) -> Self {
        Self {
            client,
            admission: Some(height),
        }
    }
}

impl crate::KvRpcServer {
    pub(crate) fn request_package_resolver(
        &self,
        admission: Option<u64>,
    ) -> crate::PackageResolver {
        let Some(height) = admission else {
            return self.package_resolver.clone();
        };
        // Each admitted request owns both its package store and its caches.
        let store: Arc<dyn PackageStore> =
            Arc::new(rtd_package_resolver::PackageStoreWithLruCache::new(
                BigTablePackageStore::admitted(self.client.clone(), height),
            ));
        Arc::new(rtd_package_resolver::Resolver::new(store))
    }
}

#[async_trait::async_trait]
impl PackageStore for BigTablePackageStore {
    async fn fetch(&self, id: AccountAddress) -> rtd_package_resolver::Result<Arc<Package>> {
        if let Some(height) = self.admission {
            let package = crate::v2::move_package_service::load_package(
                self.client.clone(),
                id.into(),
                Some(height),
            )
            .await
            .map_err(|error| Error::Store {
                store: STORE,
                error: error.to_string(),
            })?;
            return Ok(Arc::new(Package::read_from_package(&package)?));
        }
        let mut client = self.client.clone();
        let object = client
            .get_latest_object(&ObjectID::from(id))
            .await
            .map_err(|e| Error::Store {
                store: STORE,
                error: e.to_string(),
            })?
            .ok_or(Error::PackageNotFound(id))?;
        let package = object
            .data
            .try_as_package()
            .ok_or(Error::NotAPackage(object.id().into()))?;
        Ok(Arc::new(Package::read_from_package(package)?))
    }
}
