// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dataloader::DataLoader;
use async_graphql::dataloader::Loader;
use diesel::prelude::QueryableByName;
use diesel::sql_types::Array;
use diesel::sql_types::BigInt;
use diesel::sql_types::Bytea;
use move_core_types::account_address::AccountAddress;
use rtd_indexer_alt_schema::schema::kv_packages;
use rtd_package_resolver::Package;
use rtd_package_resolver::PackageStore;
use rtd_package_resolver::PackageStoreWithLruCache;
use rtd_package_resolver::Result;
use rtd_package_resolver::error::Error;
use rtd_types::object::Object;

use crate::pg_reader::PgReader;

const STORE: &str = "PostgreSQL";

pub type PackageCache = PackageStoreWithLruCache<DbPackageStore>;
pub struct DbPackageStore {
    loader: Arc<DataLoader<PgReader>>,
    checkpoint: Option<u64>,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
struct PackageKey {
    id: AccountAddress,
    checkpoint: Option<u64>,
}

impl DbPackageStore {
    pub fn new(loader: Arc<DataLoader<PgReader>>) -> Self {
        Self {
            loader,
            checkpoint: None,
        }
    }

    pub fn at_checkpoint(loader: Arc<DataLoader<PgReader>>, checkpoint: u64) -> Self {
        Self {
            loader,
            checkpoint: Some(checkpoint),
        }
    }
}

#[async_trait::async_trait]
impl PackageStore for DbPackageStore {
    async fn fetch(&self, id: AccountAddress) -> Result<Arc<Package>> {
        let key = PackageKey {
            id,
            checkpoint: self.checkpoint,
        };
        let Some(package) = self.loader.load_one(key).await? else {
            return Err(Error::PackageNotFound(id));
        };

        Ok(package)
    }
}

#[async_trait::async_trait]
impl Loader<PackageKey> for PgReader {
    type Value = Arc<Package>;
    type Error = Error;

    async fn load(&self, keys: &[PackageKey]) -> Result<HashMap<PackageKey, Arc<Package>>> {
        let mut id_to_package = HashMap::new();
        if keys.is_empty() {
            return Ok(id_to_package);
        }

        let mut conn = self.connect().await.map_err(|e| Error::Store {
            store: STORE,
            error: e.to_string(),
        })?;

        #[derive(QueryableByName)]
        #[diesel(table_name = kv_packages)]
        struct SerializedPackage {
            #[diesel(sql_type = BigInt)]
            request_index: i64,
            package_version: i64,
            cp_sequence_number: i64,
            serialized_object: Vec<u8>,
        }

        let ids: Vec<_> = keys.iter().map(|key| key.id.into_bytes()).collect();
        let bounds: Vec<i64> = keys
            .iter()
            .map(|key| match key.checkpoint {
                None => Ok(i64::MAX),
                Some(height) => i64::try_from(height).map_err(|_| Error::Store {
                    store: STORE,
                    error: "Package checkpoint exceeds database range".to_owned(),
                }),
            })
            .collect::<Result<_>>()?;
        let query = diesel::sql_query(
            r#"
                SELECT
                    k.ordinality::BIGINT AS request_index,
                    v.package_version, v.cp_sequence_number, v.serialized_object
                FROM UNNEST($1::BYTEA[], $2::BIGINT[]) WITH ORDINALITY
                    AS k(package_id, checkpoint, ordinality)
                CROSS JOIN LATERAL (
                    SELECT
                        package_version, cp_sequence_number, serialized_object
                    FROM
                        kv_packages
                    WHERE
                        kv_packages.package_id = k.package_id
                        AND cp_sequence_number <= k.checkpoint
                    ORDER BY
                        package_version DESC
                    LIMIT 1
                ) v
            "#,
        )
        .bind::<Array<Bytea>, _>(ids)
        .bind::<Array<BigInt>, _>(bounds);

        let stored_packages: Vec<SerializedPackage> =
            conn.results(query).await.map_err(|e| Error::Store {
                store: STORE,
                error: e.to_string(),
            })?;

        for stored in stored_packages {
            let object: Object = bcs::from_bytes(&stored.serialized_object)?;
            let Some(move_package) = object.data.try_as_package() else {
                return Err(Error::NotAPackage(object.id().into()));
            };

            let key = stored
                .request_index
                .checked_sub(1)
                .and_then(|index| usize::try_from(index).ok())
                .and_then(|index| keys.get(index))
                .ok_or_else(|| Error::Store {
                    store: STORE,
                    error: "Invalid package request index".to_owned(),
                })?;
            if AccountAddress::from(move_package.id()) != key.id
                || i64::try_from(move_package.version().value()).ok()
                    != Some(stored.package_version)
                || stored.cp_sequence_number < 0
            {
                return Err(Error::Store {
                    store: STORE,
                    error: "Package metadata and content mismatch".to_owned(),
                });
            }
            let package = Package::read_from_package(move_package)?;
            id_to_package.insert(*key, Arc::new(package));
        }

        Ok(id_to_package)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use async_graphql::dataloader::Loader;
    use diesel::ExpressionMethods;
    use diesel::QueryDsl;
    use diesel_async::RunQueryDsl;
    use move_core_types::account_address::AccountAddress;
    use prometheus::Registry;
    use rtd_indexer_alt_schema::MIGRATIONS;
    use rtd_indexer_alt_schema::packages::StoredPackage;
    use rtd_indexer_alt_schema::schema::kv_packages;
    use rtd_package_resolver::Package;
    use rtd_package_resolver::PackageStore;
    use rtd_pg_db::Db;
    use rtd_pg_db::DbArgs;
    use rtd_pg_db::temp::TempDb;
    use rtd_types::base_types::ObjectID;
    use rtd_types::digests::TransactionDigest;
    use rtd_types::move_package::MovePackage;
    use rtd_types::object::Data;
    use rtd_types::object::Object;

    use super::DbPackageStore;
    use super::PackageKey;
    use super::PgReader;

    fn package(id: ObjectID, version: u64, checkpoint: i64) -> (StoredPackage, Package) {
        let bytes = include_bytes!(
            "../../rtd-single-node-benchmark/tests/data/package_publish_from_bytecode/package_a/build/a/bytecode_modules/a.mv"
        );
        let seed = MovePackage::new(
            id,
            1.into(),
            BTreeMap::from([("a".into(), bytes.to_vec())]),
            u64::MAX,
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
        module.address_identifiers[0] = id.into();
        let mut bytes = Vec::new();
        module
            .serialize_with_version(module.version, &mut bytes)
            .unwrap();
        let package = MovePackage::new(
            id,
            version.into(),
            BTreeMap::from([("a".into(), bytes)]),
            u64::MAX,
            vec![],
            BTreeMap::new(),
        )
        .unwrap();
        let object = Object::new_package_from_data(
            Data::Package(package),
            TransactionDigest::genesis_marker(),
        );
        let expected = Package::read_from_object(&object).unwrap();
        (
            StoredPackage {
                package_id: id.to_vec(),
                package_version: version as i64,
                original_id: id.to_vec(),
                is_system_package: true,
                cp_sequence_number: checkpoint,
                serialized_object: bcs::to_bytes(&object).unwrap(),
            },
            expected,
        )
    }

    #[tokio::test]
    async fn historical_packages_share_a_loader_without_sharing_future_layouts() {
        let temp = TempDb::new().unwrap();
        let url = temp.database().url().clone();
        let writer = Db::for_write(url.clone(), DbArgs::default()).await.unwrap();
        writer.run_migrations(Some(&MIGRATIONS)).await.unwrap();
        let reader = PgReader::new(None, Some(url), DbArgs::default(), &Registry::new())
            .await
            .unwrap();
        let id = ObjectID::from(AccountAddress::TWO);
        let (old_row, old) = package(id, 1, 5);
        let (new_row, new) = package(id, 2, 10);
        let mut conn = writer.connect().await.unwrap();
        diesel::insert_into(kv_packages::table)
            .values(vec![old_row.clone(), new_row.clone()])
            .execute(&mut conn)
            .await
            .unwrap();

        let key = |checkpoint| PackageKey {
            id: id.into(),
            checkpoint,
        };
        let found = Loader::load(
            &reader,
            &[
                key(Some(4)),
                key(Some(5)),
                key(Some(9)),
                key(Some(10)),
                key(None),
            ],
        )
        .await
        .unwrap();
        assert!(!found.contains_key(&key(Some(4))));
        for checkpoint in [Some(5), Some(9)] {
            assert_eq!(format!("{:?}", found[&key(checkpoint)]), format!("{old:?}"));
        }
        for checkpoint in [Some(10), None] {
            assert_eq!(format!("{:?}", found[&key(checkpoint)]), format!("{new:?}"));
        }
        assert!(Loader::load(&reader, &[key(Some(u64::MAX))]).await.is_err());

        let loader = Arc::new(reader.as_data_loader());
        let historical = DbPackageStore::at_checkpoint(loader.clone(), 5);
        let latest = DbPackageStore::at_checkpoint(loader, 10);
        let (older, newer) = tokio::join!(historical.fetch(id.into()), latest.fetch(id.into()));
        assert_eq!(format!("{:?}", older.unwrap()), format!("{old:?}"));
        assert_eq!(format!("{:?}", newer.unwrap()), format!("{new:?}"));

        diesel::update(kv_packages::table.filter(kv_packages::package_version.eq(2)))
            .set(kv_packages::serialized_object.eq(old_row.serialized_object.clone()))
            .execute(&mut conn)
            .await
            .unwrap();
        assert!(latest.fetch(id.into()).await.is_err());
        assert!(historical.fetch(id.into()).await.is_ok());
        diesel::update(kv_packages::table.filter(kv_packages::package_version.eq(2)))
            .set((
                kv_packages::serialized_object.eq(new_row.serialized_object),
                kv_packages::cp_sequence_number.eq(-1),
            ))
            .execute(&mut conn)
            .await
            .unwrap();
        assert!(latest.fetch(id.into()).await.is_err());
    }
}
