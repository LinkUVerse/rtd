// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Explicit Alt login supply. Ordinary verification never installs SQL, grants
//! permissions, renews a password or resurrects a previously supplied role.
use std::collections::BTreeMap;
use std::fs::DirBuilder;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::net::IpAddr;
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use anyhow::ensure;
use diesel::QueryableByName;
use diesel::sql_types::Text;
use diesel_async::RunQueryDsl;
use rand::RngCore;
use rtd_indexer_alt_framework::postgres::Connection;
use rtd_indexer_alt_framework::postgres::Db;
use rtd_indexer_alt_framework::postgres::DbArgs;
use rtd_indexer_alt_schema::MIGRATIONS;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;
use url::Url;
use uuid::Uuid;

use crate::writer_fence;

#[cfg(test)]
#[path = "postgres_identity/tests.rs"]
mod tests;

const LIMIT: u64 = 1_048_576;
const ADMISSION_LOCK: &str = "SELECT pg_try_advisory_xact_lock(1381254145,1)";

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    Writer,
    QueryReader,
    CoverageReader,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pin {
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Config {
    version: u8,
    environment_id: String,
    chain_id: String,
    data_generation: Uuid,
    request_id: Uuid,
    administrator_url: Pin,
    tls_ca: Option<Pin>,
    runtime_max_connections: u32,
    operation_timeout_ms: u64,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Identity {
    version: u8,
    configuration_sha256: String,
    profile: Profile,
    id: Uuid,
    password: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Supplied {
    version: u8,
    configuration_sha256: String,
    identity_sha256: String,
    role_oid: i64,
    database_oid: i64,
    system_identifier: String,
}

#[derive(QueryableByName)]
struct JsonRow {
    #[diesel(sql_type = Text)]
    value: String,
}

struct Source {
    config: Config,
    sha: String,
    url: Url,
    inputs: BTreeMap<PathBuf, (String, bool)>,
}

pub struct Native {
    source: Source,
    state: PathBuf,
    _lock: File,
    identity: Identity,
    identity_sha: String,
    supplied: Option<Vec<u8>>,
}

impl Profile {
    pub fn parse(value: &str) -> Result<Self> {
        Ok(match value {
            "writer" => Self::Writer,
            "query-reader" => Self::QueryReader,
            "coverage-reader" => Self::CoverageReader,
            _ => anyhow::bail!("fixed Alt PostgreSQL profile required"),
        })
    }

    fn name(self) -> &'static str {
        match self {
            Self::Writer => "writer",
            Self::QueryReader => "query_reader",
            Self::CoverageReader => "coverage_reader",
        }
    }

    fn permits(self, namespace: &str, table: &str, action: &str) -> bool {
        if namespace != "public" {
            return false;
        }
        match self {
            Self::CoverageReader => {
                action == "SELECT" && matches!(table, "watermarks" | "kv_checkpoints")
            }
            Self::QueryReader => action == "SELECT" && native_table(table),
            Self::Writer => {
                action == "SELECT" && native_table(table)
                    || writer_fence::FENCED_TABLES.contains(&table)
                        && matches!(action, "INSERT" | "UPDATE" | "DELETE")
                    || table == "rtd_alt_writer_fence" && matches!(action, "INSERT" | "UPDATE")
            }
        }
    }
}

impl Identity {
    fn role(&self) -> String {
        format!("rtd_alt_{}_{}", self.profile.name(), self.id.simple())
    }
}

impl Source {
    fn load(path: &Path, expected: &str, development: bool) -> Result<Self> {
        let raw = pinned(
            &Pin {
                path: path.into(),
                sha256: expected.into(),
            },
            true,
        )?;
        let config: Config = serde_json::from_slice(&raw)?;
        let chain = bs58_chain(&config.chain_id)?;
        ensure!(
            config.version == 1
                && !chain.is_empty()
                && !config.environment_id.is_empty()
                && config.environment_id.len() <= 64
                && config
                    .environment_id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                && !config.data_generation.is_nil()
                && !config.request_id.is_nil()
                && (1..=32).contains(&config.runtime_max_connections)
                && (5000..=120000).contains(&config.operation_timeout_ms),
            "invalid Alt identity contract"
        );
        let url_raw = pinned(&config.administrator_url, true)?;
        let text = std::str::from_utf8(&url_raw)?
            .strip_suffix('\n')
            .unwrap_or(std::str::from_utf8(&url_raw)?);
        ensure!(
            !text.is_empty() && !text.bytes().any(|b| b.is_ascii_whitespace() || b == 0),
            "one private URL required"
        );
        let url = Url::parse(text)?;
        let queries: Vec<_> = url.query_pairs().collect();
        ensure!(
            matches!(url.scheme(), "postgres" | "postgresql")
                && url.host_str().is_some()
                && url.port().is_some_and(|port| port != 0)
                && !url.username().is_empty()
                && url.password().is_some_and(|p| !p.is_empty())
                && url.path().len() > 1
                && !url.path()[1..].contains('/')
                && url.fragment().is_none()
                && queries.len() == 1
                && queries[0].0 == "sslmode",
            "explicit database, credentials and transport required"
        );
        ensure!(
            url.path()[1..].len() <= 63
                && url.path()[1..]
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_'),
            "explicit simple database name required"
        );
        let mut inputs = BTreeMap::from([
            (path.into(), (expected.into(), true)),
            (
                config.administrator_url.path.clone(),
                (config.administrator_url.sha256.clone(), true),
            ),
        ]);
        if let Some(ca) = &config.tls_ca {
            ensure!(queries[0].1 == "require", "verified TLS must be required");
            pinned(ca, false)?;
            inputs.insert(ca.path.clone(), (ca.sha256.clone(), false));
        } else {
            ensure!(
                development
                    && queries[0].1 == "disable"
                    && url
                        .host_str()
                        .unwrap()
                        .trim_matches(['[', ']'])
                        .parse::<IpAddr>()
                        .is_ok_and(|ip| ip.is_loopback()),
                "verified TLS required"
            );
        }
        let source = Self {
            config,
            sha: expected.into(),
            url,
            inputs,
        };
        source.unchanged()?;
        Ok(source)
    }

    fn unchanged(&self) -> Result<()> {
        for (path, (expected, private)) in &self.inputs {
            pinned(
                &Pin {
                    path: path.clone(),
                    sha256: expected.clone(),
                },
                *private,
            )?;
        }
        Ok(())
    }

    async fn database(&self, identity: Option<&Identity>) -> Result<Db> {
        self.unchanged()?;
        let mut url = self.url.clone();
        if let Some(i) = identity {
            url.set_username(&i.role())
                .map_err(|_| anyhow::anyhow!("runtime URL rejected"))?;
            url.set_password(Some(&i.password))
                .map_err(|_| anyhow::anyhow!("runtime URL rejected"))?;
        }
        Db::for_write(
            url,
            DbArgs {
                db_connection_pool_size: 1,
                db_connection_timeout_ms: 3000,
                db_statement_timeout_ms: Some(5000),
                tls_verify_cert: true,
                tls_ca_cert_path: self.config.tls_ca.as_ref().map(|p| p.path.clone()),
                writer_epoch: None,
            },
        )
        .await
    }
}

impl Native {
    /// Export only local input digests for the archive lifecycle planner. The
    /// later verify command still authenticates and rechecks live authority.
    pub fn verification_binding(&self) -> Result<Value> {
        self.unchanged()?;
        let supplied = self
            .supplied
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("supplied Alt identity required"))?;
        let marker: Supplied = serde_json::from_slice(supplied)?;
        ensure!(
            marker.version == 1
                && marker.configuration_sha256 == self.source.sha
                && marker.identity_sha256 == self.identity_sha
                && marker.role_oid > 0
                && marker.database_oid > 0
                && !marker.system_identifier.is_empty()
                && marker.system_identifier.bytes().all(|b| b.is_ascii_digit()),
            "local Alt supply binding differs"
        );
        let mut inputs: BTreeMap<PathBuf, String> = self
            .source
            .inputs
            .iter()
            .map(|(path, (digest, _))| (path.clone(), digest.clone()))
            .collect();
        for name in ["identity.json", "database.url", "postgres.json"] {
            let path = self.state.join(name);
            inputs.insert(path.clone(), sha(&private_read(&path, true)?));
        }
        inputs.insert(self.state.join("supplied.json"), sha(supplied));
        self.unchanged()?;
        Ok(json!({
            "version": 1,
            "scope": "alt-postgres-verification-binding",
            "environmentId": self.source.config.environment_id,
            "chainId": self.source.config.chain_id,
            "dataGeneration": self.source.config.data_generation,
            "profile": self.identity.profile,
            "operationTimeoutMs": self.source.config.operation_timeout_ms,
            "productionTls": self.source.config.tls_ca.is_some(),
            "inputs": inputs
        }))
    }

    pub fn load(
        path: &Path,
        expected: &str,
        state: &Path,
        identity_sha: &str,
        profile: Profile,
        development: bool,
    ) -> Result<Self> {
        let source = Source::load(path, expected, development)?;
        let lock = directory(state, false)?;
        ensure!(hex(identity_sha), "identity digest required");
        let raw = private_read(&state.join("identity.json"), true)?;
        ensure!(sha(&raw) == identity_sha, "Alt identity differs");
        let identity: Identity = serde_json::from_slice(&raw)?;
        ensure!(
            identity.version == 1
                && identity.configuration_sha256 == source.sha
                && identity.profile == profile
                && !identity.id.is_nil()
                && hex(&identity.password),
            "Alt identity binding differs"
        );
        for (name, bytes) in exports(&source, &identity)? {
            ensure!(
                private_read(&state.join(name), true)? == bytes,
                "Alt runtime projection differs"
            );
        }
        let supplied = optional(&state.join("supplied.json"))?;
        ensure!(
            supplied.is_some() || !has_journal(state, "receipt-")?,
            "supplied marker missing; no implicit recovery"
        );
        let native = Self {
            source,
            state: state.into(),
            _lock: lock,
            identity,
            identity_sha: identity_sha.into(),
            supplied,
        };
        native.unchanged()?;
        Ok(native)
    }

    fn unchanged(&self) -> Result<()> {
        self.source.unchanged()?;
        ensure!(
            sha(&private_read(&self.state.join("identity.json"), true)?) == self.identity_sha
                && optional(&self.state.join("supplied.json"))? == self.supplied,
            "Alt identity or supply marker changed"
        );
        for (name, bytes) in exports(&self.source, &self.identity)? {
            ensure!(
                private_read(&self.state.join(name), true)? == bytes,
                "Alt runtime projection changed"
            );
        }
        Ok(())
    }

    async fn authenticate(&self) -> Result<()> {
        let db = self.source.database(Some(&self.identity)).await?;
        let mut conn = db.connect().await?;
        let user = self.identity.role();
        let row = query(&mut conn, &format!("SELECT to_jsonb(current_user=session_user AND session_user={} AND EXISTS(SELECT 1 FROM pg_stat_ssl WHERE pid=pg_backend_pid() AND ssl={}))::text AS value", literal(&user), self.source.config.tls_ca.is_some())).await?;
        ensure!(row == json!(true), "Alt runtime authentication rejected");
        Ok(())
    }

    async fn permissions(&self, conn: &mut Connection<'_>, complete: bool) -> Result<()> {
        let role = literal(&self.identity.role());
        let dangerous = query(conn, &format!("SELECT to_jsonb(has_database_privilege({role},current_database(),'CREATE,TEMP') OR has_database_privilege({role},current_database(),'CONNECT WITH GRANT OPTION') OR EXISTS(SELECT 1 FROM pg_namespace n WHERE left(n.nspname,3)<>'pg_' AND n.nspname<>'information_schema' AND (n.nspowner=(SELECT oid FROM pg_roles WHERE rolname={role}) OR has_schema_privilege({role},n.oid,'CREATE') OR has_schema_privilege({role},n.oid,'USAGE WITH GRANT OPTION') OR n.nspname<>'public' AND has_schema_privilege({role},n.oid,'USAGE'))) OR EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE p.prosecdef AND left(n.nspname,3)<>'pg_' AND n.nspname<>'information_schema' AND has_function_privilege({role},p.oid,'EXECUTE')) OR EXISTS(SELECT 1 FROM pg_default_acl d CROSS JOIN LATERAL aclexplode(d.defaclacl) a WHERE a.grantee IN (0,(SELECT oid FROM pg_roles WHERE rolname={role}))))::text AS value")).await?;
        ensure!(
            dangerous == json!(false),
            "Alt authority exceeds database/schema/function/default boundary"
        );
        let relations = query(conn, "SELECT coalesce(jsonb_agg(jsonb_build_object('oid',c.oid::bigint,'namespace',n.nspname,'name',c.relname,'kind',c.relkind,'owner',c.relowner::bigint,'rls',c.relrowsecurity OR c.relforcerowsecurity) ORDER BY n.nspname,c.relname),'[]')::text AS value FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE left(n.nspname,3)<>'pg_' AND n.nspname<>'information_schema' AND c.relkind IN ('r','p','v','m','f','S')").await?;
        for r in relations
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("relation catalog rejected"))?
        {
            let ns = r["namespace"].as_str().unwrap();
            let table = r["name"].as_str().unwrap();
            let oid = r["oid"].as_i64().unwrap();
            let sequence = r["kind"] == "S";
            ensure!(
                !(ns == "public" && native_table(table)) || r["kind"] == "r" && r["rls"] == false,
                "Alt relation state differs"
            );
            let actions: &[&str] = if sequence {
                &["SELECT", "USAGE", "UPDATE"]
            } else {
                &[
                    "SELECT",
                    "INSERT",
                    "UPDATE",
                    "DELETE",
                    "TRUNCATE",
                    "REFERENCES",
                    "TRIGGER",
                    "MAINTAIN",
                ]
            };
            for action in actions {
                let predicate = if sequence {
                    "has_sequence_privilege"
                } else {
                    "has_table_privilege"
                };
                let actual = query(conn, &format!("SELECT jsonb_build_array({predicate}({role},{oid}::oid,{}),{predicate}({role},{oid}::oid,{}))::text AS value", literal(action), literal(&format!("{action} WITH GRANT OPTION")))).await?;
                let expected = !sequence && self.identity.profile.permits(ns, table, action);
                ensure!(
                    actual[1] == false
                        && (actual[0] == false || expected)
                        && (!complete || !expected || actual[0] == true),
                    "Alt table/sequence privileges differ"
                );
            }
            if !sequence {
                let columns = query(conn, &format!("SELECT coalesce(jsonb_agg(jsonb_build_array(p.action,has_column_privilege({role},{oid}::oid,a.attnum,p.action),has_column_privilege({role},{oid}::oid,a.attnum,p.action||' WITH GRANT OPTION')) ORDER BY a.attnum,p.action),'[]')::text AS value FROM pg_attribute a CROSS JOIN (VALUES('SELECT'),('INSERT'),('UPDATE'),('REFERENCES')) p(action) WHERE a.attrelid={oid}::oid AND a.attnum>0 AND NOT a.attisdropped")).await?;
                for c in columns.as_array().unwrap() {
                    let expected = self
                        .identity
                        .profile
                        .permits(ns, table, c[0].as_str().unwrap());
                    ensure!(
                        c[2] == false
                            && (c[1] == false || expected)
                            && (!complete || !expected || c[1] == true),
                        "Alt column privilege differs"
                    );
                }
            }
        }
        if complete {
            let valid = query(conn, &format!("SELECT to_jsonb(has_database_privilege({role},current_database(),'CONNECT') AND has_schema_privilege({role},'public','USAGE'))::text AS value")).await?;
            ensure!(valid == true, "Alt connection or schema usage absent");
        }
        Ok(())
    }

    async fn snapshot(&self, conn: &mut Connection<'_>) -> Result<Value> {
        self.unchanged()?;
        let public_safe = query(conn, "SELECT to_jsonb(NOT EXISTS(SELECT 1 FROM pg_database d CROSS JOIN LATERAL aclexplode(coalesce(d.datacl,acldefault('d',d.datdba))) a WHERE d.datname=current_database() AND a.grantee=0 AND (a.privilege_type IN ('TEMPORARY','CREATE') OR a.is_grantable)) AND NOT EXISTS(SELECT 1 FROM pg_namespace n CROSS JOIN LATERAL aclexplode(coalesce(n.nspacl,acldefault('n',n.nspowner))) a WHERE left(n.nspname,3)<>'pg_' AND n.nspname<>'information_schema' AND a.grantee=0 AND (n.nspname<>'public' OR a.privilege_type='CREATE' OR a.is_grantable)))::text AS value").await?;
        ensure!(
            public_safe == true,
            "database PUBLIC temporary/create or schema privileges must be explicitly restricted before supply"
        );
        Db::require_current_migrations_on(conn, Some(&MIGRATIONS)).await?;
        writer_fence::verify_on(conn).await?;
        let role = literal(&self.identity.role());
        let base = query(conn, &format!("SELECT jsonb_build_object('systemIdentifier',(pg_control_system()).system_identifier::text,'databaseOid',d.oid::bigint,'databaseOwner',d.datdba::bigint,'databaseAcl',d.datacl,'serverVersion',current_setting('server_version_num')::bigint,'administrator',current_user,'administratorSuperuser',(SELECT rolsuper FROM pg_roles WHERE rolname=current_user),'role',(SELECT jsonb_build_object('oid',r.oid::bigint,'login',r.rolcanlogin,'superuser',r.rolsuper,'createRole',r.rolcreaterole,'createDb',r.rolcreatedb,'replication',r.rolreplication,'bypassRls',r.rolbypassrls,'inherit',r.rolinherit,'connections',r.rolconnlimit,'validUntil',r.rolvaliduntil::text,'password',auth.rolpassword,'settings',r.rolconfig,'databaseOverrides',(SELECT count(*) FROM pg_db_role_setting s WHERE s.setrole=r.oid AND s.setdatabase<>0),'memberships',(SELECT count(*) FROM pg_auth_members m WHERE m.member=r.oid OR m.roleid=r.oid),'objects',(SELECT count(*) FROM pg_shdepend s WHERE s.refclassid='pg_authid'::regclass AND s.refobjid=r.oid AND s.deptype='o')) FROM pg_roles r LEFT JOIN pg_authid auth ON auth.oid=r.oid WHERE r.rolname={role}))::text AS value FROM pg_database d WHERE d.datname=current_database()")).await?;
        ensure!(
            base["administratorSuperuser"] == true
                && (170000..180000).contains(&base["serverVersion"].as_i64().unwrap_or(0)),
            "explicit PG17 administrator required"
        );
        let native_names = table_names()
            .iter()
            .map(|s| literal(s))
            .collect::<Vec<_>>()
            .join(",");
        let catalog = query(conn, &format!("SELECT coalesce(jsonb_agg(jsonb_build_object('oid',c.oid::bigint,'name',c.relname,'kind',c.relkind,'owner',c.relowner::bigint,'acl',c.relacl,'rls',c.relrowsecurity OR c.relforcerowsecurity,'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,format_type(a.atttypid,a.atttypmod),a.attnotnull,a.attacl) ORDER BY a.attnum) FROM pg_attribute a WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped),'indexes',(SELECT jsonb_agg(jsonb_build_array(pg_get_indexdef(i.indexrelid),i.indisvalid,i.indisready) ORDER BY i.indexrelid) FROM pg_index i WHERE i.indrelid=c.oid),'constraints',(SELECT jsonb_agg(pg_get_constraintdef(k.oid) ORDER BY k.oid) FROM pg_constraint k WHERE k.conrelid=c.oid)) ORDER BY c.relname),'[]')::text AS value FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relname IN ({native_names})")).await?;
        ensure!(
            catalog
                .as_array()
                .is_some_and(|a| a.len() == table_names().len()
                    && a.iter().all(|r| r["kind"] == "r" && r["rls"] == false)),
            "Alt installed relations differ"
        );
        let chains = query(conn, "SELECT coalesce(jsonb_agg(DISTINCT encode(chain_id,'hex')),'[]')::text AS value FROM public.watermarks WHERE chain_id IS NOT NULL").await?;
        let chain = bs58_chain(&self.source.config.chain_id)?;
        let chain_hex = chain.iter().map(|b| format!("{b:02x}")).collect::<String>();
        ensure!(
            chains
                .as_array()
                .unwrap()
                .iter()
                .all(|c| c.as_str() == Some(&chain_hex)),
            "Alt watermark chain differs"
        );
        let existing = !base["role"].is_null();
        if let Some(raw) = &self.supplied {
            let supplied: Supplied = serde_json::from_slice(raw)?;
            ensure!(
                supplied.version == 1
                    && supplied.configuration_sha256 == self.source.sha
                    && supplied.identity_sha256 == self.identity_sha
                    && base["role"]["oid"].as_i64() == Some(supplied.role_oid)
                    && base["databaseOid"].as_i64() == Some(supplied.database_oid)
                    && base["systemIdentifier"].as_str() == Some(&supplied.system_identifier),
                "supplied Alt role/database replaced; no implicit recovery"
            );
        } else {
            ensure!(
                existing || !has_journal(&self.state, "intent-")?,
                "unresolved creation; absent Alt role cannot be resurrected"
            );
        }
        if existing {
            let r = &base["role"];
            let expected_settings = [
                "search_path=public".to_owned(),
                format!(
                    "default_transaction_read_only={}",
                    if self.identity.profile == Profile::Writer {
                        "off"
                    } else {
                        "on"
                    }
                ),
            ];
            let mut settings: Vec<String> = serde_json::from_value(r["settings"].clone())?;
            settings.sort();
            let mut expected_settings = expected_settings.to_vec();
            expected_settings.sort();
            ensure!(
                r["login"] == true
                    && r["superuser"] == false
                    && r["createRole"] == false
                    && r["createDb"] == false
                    && r["replication"] == false
                    && r["bypassRls"] == false
                    && r["inherit"] == false
                    && r["validUntil"].is_null()
                    && r["connections"] == self.source.config.runtime_max_connections
                    && r["memberships"] == 0
                    && r["databaseOverrides"] == 0
                    && r["objects"] == 0
                    && settings == expected_settings,
                "Alt role attributes differ"
            );
            self.permissions(conn, self.supplied.is_some()).await?;
        }
        // Hash catalog/ACL and role authentication material without exposing it.
        let directory = query(conn, "SELECT jsonb_build_object('schemas',(SELECT jsonb_agg(jsonb_build_array(oid,nspname,nspowner,nspacl) ORDER BY oid) FROM pg_namespace WHERE left(nspname,3)<>'pg_' AND nspname<>'information_schema'),'relations',(SELECT jsonb_agg(jsonb_build_array(c.oid,c.relname,c.relnamespace,c.relowner,c.relkind,c.relacl,c.relrowsecurity,c.relforcerowsecurity) ORDER BY c.oid) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE left(n.nspname,3)<>'pg_' AND n.nspname<>'information_schema'),'defaults',(SELECT jsonb_agg(to_jsonb(d) ORDER BY oid) FROM pg_default_acl d),'functions',(SELECT jsonb_agg(jsonb_build_array(p.oid,p.proowner,p.proacl,p.prosecdef,pg_get_functiondef(p.oid)) ORDER BY p.oid) FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE left(n.nspname,3)<>'pg_' AND n.nspname<>'information_schema' AND p.prokind='f'))::text AS value").await?;
        Ok(
            json!({"exists":existing,"roleOid":base["role"]["oid"],"databaseOid":base["databaseOid"],"systemIdentifier":base["systemIdentifier"],"snapshotSha256":sha(&serde_json::to_vec(&(base,catalog,directory,chains))?)}),
        )
    }

    fn report(&self, snapshot: Value, verified: bool) -> Result<Value> {
        let seal = sha(&serde_json::to_vec(
            &json!({"configuration":self.source.sha,"identity":self.identity_sha,"snapshot":snapshot}),
        )?);
        Ok(
            json!({"version":1,"component":"alt-postgres","profile":self.identity.profile,"environmentId":self.source.config.environment_id,"chainId":self.source.config.chain_id,"dataGeneration":self.source.config.data_generation,"requestId":self.source.config.request_id,"role":self.identity.role(),"identitySha256":self.identity_sha,"roleOid":snapshot["roleOid"],"databaseOid":snapshot["databaseOid"],"systemIdentifier":snapshot["systemIdentifier"],"roleExists":snapshot["exists"],"expectedPlanSha256":seal,"remoteVerified":verified,"applied":false,"ordinaryStartMayInitialize":false}),
        )
    }

    pub async fn inspect(&self, verify: bool) -> Result<Value> {
        tokio::time::timeout(
            Duration::from_millis(self.source.config.operation_timeout_ms),
            async {
                let db = self.source.database(None).await?;
                let mut conn = db.connect().await?;
                execute_sql(&mut conn, "BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY").await?;
                let result = async {
                    let snapshot = self.snapshot(&mut conn).await?;
                    if snapshot["exists"] == true {
                        self.authenticate().await?;
                    }
                    if verify {
                        ensure!(snapshot["exists"] == true, "Alt role absent");
                        self.permissions(&mut conn, true).await?;
                    }
                    self.unchanged()?;
                    self.report(snapshot, verify)
                }
                .await;
                let rollback = execute_sql(&mut conn, "ROLLBACK").await;
                result.and_then(|v| rollback.map(|_| v))
            },
        )
        .await?
    }

    pub async fn apply(&self, expected: &str) -> Result<Value> {
        ensure!(hex(expected), "current Alt identity plan required");
        tokio::time::timeout(Duration::from_millis(self.source.config.operation_timeout_ms), async {
            self.unchanged()?;
            let receipt = format!("receipt-{expected}.json");
            if let Some(raw) = optional(&self.state.join(&receipt))? {
                let old: Value = serde_json::from_slice(&raw)?;
                ensure!(old["identitySha256"] == self.identity_sha && old["appliedPlanSha256"] == expected && old["remoteVerified"] == true, "Alt receipt differs");
                let mut result = self.inspect(true).await?; result["applied"] = json!(true); result["appliedPlanSha256"] = json!(expected); return Ok(result);
            }
            let db = self.source.database(None).await?; let mut conn = db.connect().await?;
            execute_sql(&mut conn, "BEGIN").await?;
            let result: Result<()> = async {
                let lock = query(&mut conn, &format!("SELECT to_jsonb(({ADMISSION_LOCK}))::text AS value")).await?;
                ensure!(lock == true, "Alt writer or schema operation active");
                let snapshot = self.snapshot(&mut conn).await?;
                let exists = snapshot["exists"] == true;
                let plan = self.report(snapshot, false)?;
                ensure!(plan["expectedPlanSha256"] == expected, "Alt identity plan changed");
                if exists { self.authenticate().await?; }
                once(&self.state, &format!("intent-{expected}.json"), &serde_json::to_vec(&plan)?)?;
                let role = identifier(&self.identity.role());
                if !exists {
                    let verifier = postgres_protocol::password::scram_sha_256(self.identity.password.as_bytes());
                    execute_sql(&mut conn, &format!("CREATE ROLE {role} LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS CONNECTION LIMIT {} PASSWORD {}", self.source.config.runtime_max_connections, literal(&verifier))).await?;
                    execute_sql(&mut conn, &format!("ALTER ROLE {role} SET search_path=public")).await?;
                    execute_sql(&mut conn, &format!("ALTER ROLE {role} SET default_transaction_read_only={}", if self.identity.profile == Profile::Writer { "off" } else { "on" })).await?;
                }
                let db_name = query(&mut conn, "SELECT to_jsonb(current_database())::text AS value").await?;
                execute_sql(&mut conn, &format!("GRANT CONNECT ON DATABASE {} TO {role}; GRANT USAGE ON SCHEMA public TO {role}", identifier(db_name.as_str().unwrap()))).await?;
                for table in table_names() { for action in ["SELECT","INSERT","UPDATE","DELETE"] {
                    if self.identity.profile.permits("public", table, action) { execute_sql(&mut conn, &format!("GRANT {action} ON public.{} TO {role}", identifier(table))).await?; }
                }}
                self.permissions(&mut conn, true).await?;
                Db::require_current_migrations_on(&mut conn, Some(&MIGRATIONS)).await?;
                writer_fence::verify_on(&mut conn).await?;
                self.unchanged()?;
                Ok(())
            }.await;
            if let Err(error) = result { let _ = execute_sql(&mut conn, "ROLLBACK").await; return Err(error); }
            execute_sql(&mut conn, "COMMIT").await?;
            drop(conn); drop(db);
            let mut result = self.inspect(true).await?;
            once(&self.state, "supplied.json", &serde_json::to_vec(&Supplied {
                version:1, configuration_sha256:self.source.sha.clone(), identity_sha256:self.identity_sha.clone(),
                role_oid:result["roleOid"].as_i64().ok_or_else(|| anyhow::anyhow!("Alt role OID absent"))?,
                database_oid:result["databaseOid"].as_i64().unwrap(), system_identifier:result["systemIdentifier"].as_str().unwrap().into(),
            })?)?;
            result["applied"] = json!(true); result["appliedPlanSha256"] = json!(expected);
            once(&self.state, &receipt, &serde_json::to_vec(&result)?)?;
            Ok(result)
        }).await?
    }
}

fn native_table(table: &str) -> bool {
    writer_fence::FENCED_TABLES.contains(&table)
        || matches!(table, "__diesel_schema_migrations" | "rtd_alt_writer_fence")
}

fn table_names() -> Vec<&'static str> {
    let mut tables = writer_fence::FENCED_TABLES.to_vec();
    tables.extend(["__diesel_schema_migrations", "rtd_alt_writer_fence"]);
    tables.sort_unstable();
    tables
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn literal(value: &str) -> String {
    format!("E'{}'", value.replace('\\', "\\\\").replace('\'', "''"))
}
fn identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn bs58_chain(value: &str) -> Result<Vec<u8>> {
    use fastcrypto::encoding::Base58;
    use fastcrypto::encoding::Encoding;
    let bytes = Base58::decode(value)?;
    ensure!(
        bytes.len() == 32 && Base58::encode(&bytes) == value,
        "canonical full chain ID required"
    );
    Ok(bytes)
}

fn private_read(path: &Path, private: bool) -> Result<Vec<u8>> {
    ensure!(
        path.is_absolute() && path.canonicalize()? == path,
        "canonical source required"
    );
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)?;
    let before = file.metadata()?;
    // SAFETY: geteuid takes no pointers and has no memory safety preconditions.
    let uid = unsafe { libc::geteuid() };
    ensure!(
        before.is_file()
            && before.len() <= LIMIT
            && before.mode() & 0o022 == 0
            && (before.uid() == uid || !private && before.uid() == 0)
            && (!private
                || before.uid() == uid && before.mode() & 0o077 == 0 && before.nlink() == 1),
        "owned private regular input required"
    );
    let mut bytes = Vec::new();
    (&mut file).take(LIMIT + 1).read_to_end(&mut bytes)?;
    let after = file.metadata()?;
    ensure!(
        bytes.len() as u64 == before.len()
            && before.dev() == after.dev()
            && before.ino() == after.ino()
            && before.len() == after.len()
            && before.mtime() == after.mtime()
            && before.mtime_nsec() == after.mtime_nsec()
            && before.ctime() == after.ctime()
            && before.ctime_nsec() == after.ctime_nsec(),
        "input changed during read"
    );
    Ok(bytes)
}

fn pinned(pin: &Pin, private: bool) -> Result<Vec<u8>> {
    ensure!(hex(&pin.sha256), "input SHA required");
    let bytes = private_read(&pin.path, private)?;
    ensure!(sha(&bytes) == pin.sha256, "pinned input differs");
    Ok(bytes)
}

fn optional(path: &Path) -> Result<Option<Vec<u8>>> {
    match path.symlink_metadata() {
        Ok(_) => Ok(Some(private_read(path, true)?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn directory(path: &Path, create: bool) -> Result<File> {
    ensure!(
        path.is_absolute()
            && path
                .parent()
                .is_some_and(|p| p.canonicalize().is_ok_and(|c| c == p)),
        "canonical state parent required"
    );
    if create && !path.exists() {
        DirBuilder::new().mode(0o700).create(path)?;
    }
    let meta = path.symlink_metadata()?;
    // SAFETY: geteuid takes no pointers and has no memory safety preconditions.
    let uid = unsafe { libc::geteuid() };
    ensure!(
        meta.is_dir()
            && !meta.file_type().is_symlink()
            && meta.uid() == uid
            && meta.mode() & 0o077 == 0
            && path.canonicalize()? == path,
        "private state directory required"
    );
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(create)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path.join("identity.lock"))?;
    let metadata = file.metadata()?;
    ensure!(
        metadata.is_file()
            && metadata.uid() == uid
            && metadata.mode() & 0o077 == 0
            && metadata.nlink() == 1,
        "private identity lock required"
    );
    // SAFETY: file owns a valid descriptor, flock neither stores nor dereferences Rust pointers.
    ensure!(
        unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0,
        "Alt identity operation already running"
    );
    Ok(file)
}

fn once(directory: &Path, name: &str, bytes: &[u8]) -> Result<()> {
    ensure!(
        !name.is_empty() && !name.contains(['/', '\\']) && name != "." && name != "..",
        "fixed state filename required"
    );
    let destination = directory.join(name);
    if let Some(old) = optional(&destination)? {
        ensure!(old == bytes, "existing private state differs");
        return Ok(());
    }
    let temporary = directory.join(format!(".identity-{}.tmp", Uuid::new_v4()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    let result = std::fs::hard_link(&temporary, &destination);
    std::fs::remove_file(&temporary)?;
    result?;
    File::open(directory)?.sync_all()?;
    Ok(())
}

fn has_journal(directory: &Path, prefix: &str) -> Result<bool> {
    let mut found = false;
    for (count, entry) in std::fs::read_dir(directory)?.enumerate() {
        ensure!(count < 10000, "identity journal limit exceeded");
        found |= entry?.file_name().to_string_lossy().starts_with(prefix);
    }
    Ok(found)
}

fn exports(source: &Source, identity: &Identity) -> Result<Vec<(&'static str, Vec<u8>)>> {
    let mut url = source.url.clone();
    url.set_username(&identity.role())
        .map_err(|_| anyhow::anyhow!("runtime URL rejected"))?;
    url.set_password(Some(&identity.password))
        .map_err(|_| anyhow::anyhow!("runtime URL rejected"))?;
    Ok(vec![
        ("database.url", format!("{url}\n").into_bytes()),
        (
            "postgres.json",
            serde_json::to_vec(
                &json!({"host":url.host_str(),"port":url.port().unwrap_or(5432),"database":url.path().strip_prefix('/').unwrap(),"user":identity.role(),"password":identity.password,"ca_file":source.config.tls_ca.as_ref().map(|p| &p.path)}),
            )?,
        ),
    ])
}

pub fn prepare(
    path: &Path,
    expected: &str,
    state: &Path,
    profile: Profile,
    development: bool,
) -> Result<Value> {
    let source = Source::load(path, expected, development)?;
    let _lock = directory(state, true)?;
    ensure!(
        optional(&state.join("supplied.json"))?.is_some() || !has_journal(state, "receipt-")?,
        "Alt supplied marker missing; no implicit recovery"
    );
    let identity_path = state.join("identity.json");
    if optional(&identity_path)?.is_none() {
        ensure!(
            !has_journal(state, "intent-")?
                && !has_journal(state, "receipt-")?
                && optional(&state.join("supplied.json"))?.is_none(),
            "lost Alt identity cannot be replaced"
        );
        let mut random = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut random);
        let identity = Identity {
            version: 1,
            configuration_sha256: source.sha.clone(),
            profile,
            id: Uuid::new_v4(),
            password: random.iter().map(|b| format!("{b:02x}")).collect(),
        };
        once(state, "identity.json", &serde_json::to_vec(&identity)?)?;
    }
    let raw = private_read(&identity_path, true)?;
    let identity: Identity = serde_json::from_slice(&raw)?;
    ensure!(
        identity.version == 1
            && identity.configuration_sha256 == source.sha
            && identity.profile == profile
            && !identity.id.is_nil()
            && hex(&identity.password),
        "Alt identity scope differs"
    );
    for (name, bytes) in exports(&source, &identity)? {
        once(state, name, &bytes)?;
    }
    source.unchanged()?;
    Ok(
        json!({"version":1,"component":"alt-postgres","profile":profile,"role":identity.role(),"identitySha256":sha(&raw),"prepared":true,"remoteEffects":false,"activated":false}),
    )
}

async fn query(conn: &mut Connection<'_>, sql: &str) -> Result<Value> {
    let row: JsonRow = diesel::sql_query(sql).get_result(&mut **conn).await?;
    Ok(serde_json::from_str(&row.value)?)
}

async fn execute_sql(conn: &mut Connection<'_>, sql: &str) -> Result<()> {
    use diesel_async::SimpleAsyncConnection;
    conn.batch_execute(sql).await?;
    Ok(())
}

/// Called only by explicit initialize-schema --apply while its admission lock
/// is held. Identity plan/verify and ordinary indexer startup cannot call this.
pub async fn restrict_runtime_access(conn: &mut Connection<'_>) -> Result<()> {
    let database = query(conn, "SELECT to_jsonb(current_database())::text AS value").await?;
    execute_sql(conn, &format!("REVOKE CREATE,TEMPORARY ON DATABASE {} FROM PUBLIC; REVOKE CREATE ON SCHEMA public FROM PUBLIC", identifier(database.as_str().ok_or_else(|| anyhow::anyhow!("database identity absent"))?))).await
}

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    profile: &str,
    action: &str,
    config: &Path,
    expected: &str,
    state: &Path,
    identity: Option<&str>,
    plan: Option<&str>,
    development: bool,
) -> Result<Value> {
    let profile = Profile::parse(profile)?;
    if action == "prepare" {
        ensure!(
            identity.is_none() && plan.is_none(),
            "prepare takes no identity or plan"
        );
        return prepare(config, expected, state, profile, development);
    }
    ensure!(
        matches!(action, "plan" | "apply" | "verify" | "binding")
            && (action == "apply" || plan.is_none()),
        "fixed action and arguments required"
    );
    let native = Native::load(
        config,
        expected,
        state,
        identity.ok_or_else(|| anyhow::anyhow!("identity SHA required"))?,
        profile,
        development,
    )?;
    if action == "binding" {
        native.verification_binding()
    } else if action == "apply" {
        native
            .apply(plan.ok_or_else(|| anyhow::anyhow!("plan SHA required"))?)
            .await
    } else {
        native.inspect(action == "verify").await
    }
}
