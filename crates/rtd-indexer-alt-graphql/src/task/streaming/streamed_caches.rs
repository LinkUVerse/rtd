// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use async_graphql::dataloader::DataLoader;
use rtd_indexer_alt_reader::pg_reader::PgReader;

use super::EvictableCache;
use super::StreamedObjectStore;
use super::StreamedTransactionStore;
use super::StreamingPackageStore;

/// The in-memory caches a streamed checkpoint reads ahead of the durable index: packages for type
/// resolution, transaction contents for live `previousTransaction` lookups, and object contents for
/// live cross-checkpoint object lookups. Threaded as one handle from the stream task down to the
/// subscription resolvers, so a scope carries every streamed source it can read from in one place.
pub(crate) struct StreamedCaches {
    pub(crate) package_loader: Option<Arc<DataLoader<PgReader>>>,
    pub(crate) package_store: Arc<StreamingPackageStore>,
    pub(crate) transaction_store: Arc<StreamedTransactionStore>,
    pub(crate) object_store: Arc<StreamedObjectStore>,
}

impl StreamedCaches {
    pub(crate) fn new(
        package_store: Arc<StreamingPackageStore>,
        transaction_store: Arc<StreamedTransactionStore>,
        object_store: Arc<StreamedObjectStore>,
    ) -> Self {
        Self {
            package_loader: None,
            package_store,
            transaction_store,
            object_store,
        }
    }

    pub(crate) fn with_package_loader(mut self, loader: Arc<DataLoader<PgReader>>) -> Self {
        self.package_loader = Some(loader);
        self
    }

    /// Each cache as an [`EvictableCache`], for registration with the eviction task. Eviction stays
    /// per-store: each drains against its own watermark pipeline.
    pub(crate) fn to_evictable(&self) -> Vec<Arc<dyn EvictableCache>> {
        vec![
            self.package_store.clone(),
            self.transaction_store.clone(),
            self.object_store.clone(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use move_core_types::account_address::AccountAddress;
    use prometheus::Registry;
    use rtd_indexer_alt_reader::package_resolver::DbPackageStore;
    use rtd_indexer_alt_reader::package_resolver::PackageCache;
    use rtd_indexer_alt_reader::pg_reader::PgReader;
    use rtd_package_resolver::Package;
    use rtd_package_resolver::PackageStore;
    use rtd_pg_db::DbArgs;

    use crate::scope::Scope;
    use crate::task::streaming::process_checkpoint;
    use crate::task::streaming::test_utils::make_test_proto_checkpoint;

    use super::*;

    #[tokio::test]
    async fn scopes_bind_streamed_packages_and_refuse_unavailable_historical_fallback() {
        let reader = PgReader::new(None, None, DbArgs::default(), &Registry::new())
            .await
            .unwrap();
        let loader = Arc::new(reader.as_data_loader());
        let packages = Arc::new(StreamingPackageStore::new(Arc::new(PackageCache::new(
            DbPackageStore::new(loader.clone()),
        ))));
        let id = AccountAddress::TWO;
        let old = Arc::new(Package::for_test(id, 1.into()));
        let new = Arc::new(Package::for_test(id, 2.into()));
        packages.index_packages(5, std::slice::from_ref(&old));
        packages.index_packages(10, std::slice::from_ref(&new));
        let caches = Arc::new(
            StreamedCaches::new(
                packages.clone(),
                Arc::new(StreamedTransactionStore::new()),
                Arc::new(StreamedObjectStore::new()),
            )
            .with_package_loader(loader),
        );
        for (height, expected) in [(7, &old), (10, &new)] {
            let checkpoint =
                Arc::new(process_checkpoint(make_test_proto_checkpoint(height)).unwrap());
            let scope = Scope::for_streamed_checkpoint(
                caches.clone(),
                crate::config::Limits::default().package_resolver(),
                checkpoint,
            );
            assert!(Arc::ptr_eq(&scope.fetch(id).await.unwrap(), expected));
        }
        // Backfill must use its historical database view even while a newer streamed package exists.
        let backfill = Scope::for_backfill(
            caches.clone(),
            crate::config::Limits::default().package_resolver(),
            7,
        );
        assert!(backfill.fetch(id).await.is_err());
        packages.evict_up_to(5);
        let checkpoint = Arc::new(process_checkpoint(make_test_proto_checkpoint(7)).unwrap());
        let historical = Scope::for_streamed_checkpoint(
            caches,
            crate::config::Limits::default().package_resolver(),
            checkpoint,
        );
        assert!(historical.fetch(id).await.is_err());
        assert!(Arc::ptr_eq(&packages.fetch(id).await.unwrap(), &new));
    }
}
