// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::Stdio;

use serde_json::Value;
use serde_json::json;
use tokio::process::Command;

use super::Config;
use super::Identity;
use super::Native;
use super::Pin;
use super::Profile;
use super::Source;
use super::execute;
use super::execute_sql;
use super::identifier;
use super::literal;
use super::prepare;
use super::private_read;
use super::query;
use super::sha;

#[path = "preflight.rs"]
mod preflight;

fn private(path: &Path, raw: impl AsRef<[u8]>) -> String {
    fs::write(path, raw.as_ref()).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    sha(raw.as_ref())
}

fn configuration(root: &Path, url: &str, ca: Option<&Path>) -> (std::path::PathBuf, String) {
    let admin = root.join("administrator.url");
    let admin_sha = private(&admin, url);
    let config = Config {
        version: 1,
        environment_id: "alt-pg-native".into(),
        chain_id: "11111111111111111111111111111111".into(),
        data_generation: "71000000-0000-4000-8000-000000000001".parse().unwrap(),
        request_id: "72000000-0000-4000-8000-000000000001".parse().unwrap(),
        administrator_url: Pin {
            path: admin,
            sha256: admin_sha,
        },
        tls_ca: ca.map(|p| Pin {
            path: p.into(),
            sha256: sha(&fs::read(p).unwrap()),
        }),
        runtime_max_connections: 4,
        operation_timeout_ms: 5000,
    };
    let path = root.join("config.json");
    let sha = private(&path, serde_json::to_vec(&config).unwrap());
    (path, sha)
}

#[tokio::test]
async fn offline_profiles_are_private_stable_disjoint_and_never_create_remote_state() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let (config, sha) = configuration(
        &root,
        "postgres://postgres:unused@127.0.0.1:1/alt?sslmode=disable",
        None,
    );
    let mut roles = std::collections::BTreeSet::new();
    let mut passwords = std::collections::BTreeSet::new();
    for (name, profile) in [
        ("writer", Profile::Writer),
        ("query-reader", Profile::QueryReader),
        ("coverage-reader", Profile::CoverageReader),
    ] {
        let state = root.join(name);
        let first = prepare(&config, &sha, &state, profile, true).unwrap();
        assert_eq!(
            first,
            prepare(&config, &sha, &state, profile, true).unwrap()
        );
        let raw = private_read(&state.join("identity.json"), true).unwrap();
        let identity: Identity = serde_json::from_slice(&raw).unwrap();
        assert!(roles.insert(identity.role()));
        assert!(passwords.insert(identity.password.clone()));
        assert_eq!(
            fs::metadata(&state).unwrap().permissions().mode() & 0o777,
            0o700
        );
        for file in [
            "identity.json",
            "database.url",
            "postgres.json",
            "identity.lock",
        ] {
            assert_eq!(
                fs::metadata(state.join(file)).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        let mut other = Profile::Writer;
        if profile == other {
            other = Profile::QueryReader;
        }
        assert!(prepare(&config, &sha, &state, other, true).is_err());
        let native = Native::load(
            &config,
            &sha,
            &state,
            first["identitySha256"].as_str().unwrap(),
            profile,
            true,
        )
        .unwrap();
        assert!(
            Native::load(
                &config,
                &sha,
                &state,
                first["identitySha256"].as_str().unwrap(),
                profile,
                true
            )
            .is_err(),
            "concurrent identity use admitted"
        );
        drop(native);
        assert_eq!(
            private_read(&state.join("identity.json"), true).unwrap(),
            raw
        );
        assert!(!state.join("supplied.json").exists());
    }
    assert!(!Profile::Writer.permits("public", "__diesel_schema_migrations", "UPDATE"));
    assert!(!Profile::Writer.permits("public", "rtd_alt_writer_fence", "DELETE"));
    assert!(Profile::Writer.permits("public", "kv_transactions", "DELETE"));
    assert!(!Profile::CoverageReader.permits("public", "kv_transactions", "SELECT"));
    assert!(!Profile::QueryReader.permits("other", "kv_transactions", "SELECT"));
}

#[tokio::test]
async fn offline_binding_requires_supply_and_never_contains_credentials() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let (config, digest) = configuration(
        &root,
        "postgres://postgres:unused@127.0.0.1:1/alt?sslmode=disable",
        None,
    );
    let state = root.join("writer");
    let prepared = prepare(&config, &digest, &state, Profile::Writer, true).unwrap();
    let id = prepared["identitySha256"].as_str().unwrap();
    let native = Native::load(&config, &digest, &state, id, Profile::Writer, true).unwrap();
    assert!(native.verification_binding().is_err());
    drop(native);
    private(&state.join("supplied.json"), json!({"version":1,"configurationSha256":digest,"identitySha256":id,"roleOid":1,"databaseOid":2,"systemIdentifier":"123"}).to_string());
    let native = Native::load(&config, &digest, &state, id, Profile::Writer, true).unwrap();
    let binding = native.verification_binding().unwrap();
    assert_eq!(binding["productionTls"], false);
    assert_eq!(binding["inputs"].as_object().unwrap().len(), 6);
    assert_eq!(
        binding["inputs"][state.join("identity.json").to_str().unwrap()],
        id
    );
    let encoded = binding.to_string();
    let identity: Identity =
        serde_json::from_slice(&fs::read(state.join("identity.json")).unwrap()).unwrap();
    assert!(!encoded.contains(&identity.password));
    assert!(!encoded.contains("postgres://"));
    private(&state.join("database.url"), b"changed");
    assert!(native.verification_binding().is_err());
}

#[tokio::test]
async fn invalid_source_projection_mode_pin_and_lost_identity_are_rejected_offline() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let (config, sha) = configuration(
        &root,
        "postgres://postgres:unused@127.0.0.1:1/alt?sslmode=disable",
        None,
    );
    assert!(
        prepare(
            &config,
            &sha,
            &root.join("production"),
            Profile::Writer,
            false
        )
        .is_err()
    );
    assert!(!root.join("production").exists());
    assert!(
        prepare(
            &config,
            &"0".repeat(64),
            &root.join("wrong-pin"),
            Profile::Writer,
            true
        )
        .is_err()
    );
    let raw = fs::read(&config).unwrap();
    let original: Value = serde_json::from_slice(&raw).unwrap();
    for (field, value) in [
        ("version", json!(2)),
        ("chainId", json!("invalid")),
        (
            "dataGeneration",
            json!("00000000-0000-0000-0000-000000000000"),
        ),
        ("runtimeMaxConnections", json!(0)),
        ("operationTimeoutMs", json!(1)),
        ("unknown", json!(true)),
    ] {
        let mut v = original.clone();
        v[field] = value;
        let pin = private(&config, v.to_string());
        assert!(Source::load(&config, &pin, true).is_err(), "field {field}");
    }
    private(&config, &raw);
    let state = root.join("writer");
    let first = prepare(&config, &sha, &state, Profile::Writer, true).unwrap();
    let id = first["identitySha256"].as_str().unwrap();
    let projected = fs::read(state.join("postgres.json")).unwrap();
    private(&state.join("postgres.json"), b"{}");
    assert!(Native::load(&config, &sha, &state, id, Profile::Writer, true).is_err());
    private(&state.join("postgres.json"), &projected);
    fs::set_permissions(
        state.join("identity.json"),
        fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    assert!(Native::load(&config, &sha, &state, id, Profile::Writer, true).is_err());
    fs::set_permissions(
        state.join("identity.json"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    fs::rename(state.join("identity.json"), state.join("kept.json")).unwrap();
    symlink(state.join("kept.json"), state.join("identity.json")).unwrap();
    assert!(Native::load(&config, &sha, &state, id, Profile::Writer, true).is_err());
    fs::remove_file(state.join("identity.json")).unwrap();
    private(&state.join("intent-test.json"), b"{}");
    assert!(prepare(&config, &sha, &state, Profile::Writer, true).is_err());
    assert_eq!(fs::read(state.join("postgres.json")).unwrap(), projected);
}

async fn cli(
    fixture: &Value,
    profile: &str,
    action: &str,
    config: &Path,
    sha: &str,
    state: &Path,
    identity: Option<&str>,
    plan: Option<&str>,
) -> std::process::Output {
    let mut command = Command::new(fixture["indexer"].as_str().unwrap());
    command
        .env_clear()
        .env("RUST_LOG", "error")
        .args([
            "postgres-identity",
            "--profile",
            profile,
            "--action",
            action,
            "--config",
            config.to_str().unwrap(),
            "--expected-sha256",
            sha,
            "--state-directory",
            state.to_str().unwrap(),
        ])
        .stdin(Stdio::null())
        .kill_on_drop(true);
    if let Some(id) = identity {
        command.args(["--identity-sha256", id]);
    }
    if let Some(plan) = plan {
        command.args(["--expected-plan-sha256", plan]);
    }
    tokio::time::timeout(std::time::Duration::from_secs(45), command.output())
        .await
        .unwrap()
        .unwrap()
}

async fn rejected_all(
    fixture: &Value,
    profile: &str,
    config: &Path,
    sha: &str,
    state: &Path,
    id: &str,
    receipt_plan: &str,
) {
    for action in ["plan", "verify", "apply"] {
        let output = cli(
            fixture,
            profile,
            action,
            config,
            sha,
            state,
            Some(id),
            if action == "apply" {
                Some(receipt_plan)
            } else {
                None
            },
        )
        .await;
        assert_eq!(
            output.status.code(),
            Some(2),
            "{profile}/{action} was admitted"
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("ALT_POSTGRES_IDENTITY_REJECTED"));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires isolated PostgreSQL17 TLS fixture and freshly built native indexer"]
async fn actual_tls_three_profiles_real_sql_current_authority_and_no_resurrection() {
    let fixture: Value = serde_json::from_slice(
        &fs::read(std::env::var("RTD_NATIVE_ALT_PG_FIXTURE").unwrap()).unwrap(),
    )
    .unwrap();
    let root = Path::new(fixture["evidence"].as_str().unwrap());
    let (config, sha) = configuration(
        root,
        fixture["administratorUrl"].as_str().unwrap(),
        Some(Path::new(fixture["ca"].as_str().unwrap())),
    );
    let initialized = Command::new(fixture["indexer"].as_str().unwrap())
        .env_clear()
        .env("RUST_LOG", "error")
        .args([
            "initialize-schema",
            "--database-url-file",
            root.join("administrator.url").to_str().unwrap(),
            "--tls-verify-cert",
            "--tls-ca-cert-path",
            fixture["ca"].as_str().unwrap(),
            "--db-connection-pool-size",
            "2",
            "--db-statement-timeout-ms",
            "5000",
            "--apply",
        ])
        .kill_on_drop(true)
        .output()
        .await
        .unwrap();
    assert!(
        initialized.status.success(),
        "explicit Alt schema initializer failed"
    );
    let source = Source::load(&config, &sha, false).unwrap();
    let db = source.database(None).await.unwrap();
    let mut admin = db.connect().await.unwrap();
    let first = prepare(
        &config,
        &sha,
        &root.join("precondition"),
        Profile::Writer,
        false,
    )
    .unwrap();
    let native = Native::load(
        &config,
        &sha,
        &root.join("precondition"),
        first["identitySha256"].as_str().unwrap(),
        Profile::Writer,
        false,
    )
    .unwrap();
    assert!(native.inspect(false).await.is_err(), "PUBLIC TEMP accepted");
    drop(native);
    let database = query(
        &mut admin,
        "SELECT to_jsonb(current_database())::text AS value",
    )
    .await
    .unwrap();
    let restricted = Command::new(fixture["indexer"].as_str().unwrap())
        .env_clear()
        .env("RUST_LOG", "error")
        .args([
            "initialize-schema",
            "--database-url-file",
            root.join("administrator.url").to_str().unwrap(),
            "--tls-verify-cert",
            "--tls-ca-cert-path",
            fixture["ca"].as_str().unwrap(),
            "--db-connection-pool-size",
            "2",
            "--apply",
            "--restrict-runtime-access",
        ])
        .kill_on_drop(true)
        .output()
        .await
        .unwrap();
    assert!(
        restricted.status.success(),
        "explicit runtime ACL initialization rejected"
    );
    execute_sql(
        &mut admin,
        "CREATE SCHEMA outside; CREATE TABLE outside.protected(value bigint)",
    )
    .await
    .unwrap();
    let mut fault_cases = 0;
    for (name, profile) in [
        ("writer", Profile::Writer),
        ("query-reader", Profile::QueryReader),
        ("coverage-reader", Profile::CoverageReader),
    ] {
        let state = root.join(name);
        let prepared = prepare(&config, &sha, &state, profile, false).unwrap();
        let id = prepared["identitySha256"].as_str().unwrap();
        let raw = fs::read(state.join("identity.json")).unwrap();
        let identity: Identity = serde_json::from_slice(&raw).unwrap();
        let role = identifier(&identity.role());
        let output = cli(
            &fixture,
            name,
            "plan",
            &config,
            &sha,
            &state,
            Some(id),
            None,
        )
        .await;
        if !output.status.success() {
            let native = Native::load(&config, &sha, &state, id, profile, false).unwrap();
            let error = native
                .inspect(false)
                .await
                .expect_err("CLI and direct plan diverged");
            let administrator =
                url::Url::parse(fixture["administratorUrl"].as_str().unwrap()).unwrap();
            let diagnostic = format!("{error:#}")
                .replace(administrator.password().unwrap(), "<redacted>")
                .replace(&identity.password, "<redacted>");
            private(&root.join("plan-diagnostic.txt"), &diagnostic);
            panic!("Alt plan rejected: {diagnostic}");
        }
        let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
        let p = plan["expectedPlanSha256"].as_str().unwrap();
        let applied = cli(
            &fixture,
            name,
            "apply",
            &config,
            &sha,
            &state,
            Some(id),
            Some(p),
        )
        .await;
        assert!(applied.status.success(), "{name} supply rejected");
        preflight::check(&fixture, name, &config, &sha, &state, id, &mut admin, &role).await;
        let verify = cli(
            &fixture,
            name,
            "verify",
            &config,
            &sha,
            &state,
            Some(id),
            None,
        )
        .await;
        assert!(verify.status.success());
        let value: Value = serde_json::from_slice(&verify.stdout).unwrap();
        assert_eq!(value["remoteVerified"], true);
        let own = source.database(Some(&identity)).await.unwrap();
        let mut conn = own.connect().await.unwrap();
        assert_eq!(
            query(
                &mut conn,
                "SELECT to_jsonb(count(*))::text AS value FROM public.watermarks"
            )
            .await
            .unwrap(),
            0
        );
        if profile != Profile::Writer {
            execute_sql(&mut conn, "SET default_transaction_read_only=off")
                .await
                .unwrap();
            let denied = execute_sql(&mut conn, "DELETE FROM public.watermarks WHERE false")
                .await
                .unwrap_err();
            assert!(denied.to_string().contains("permission denied"));
        } else {
            let epoch = crate::writer_fence::advance_existing_epoch(&mut conn)
                .await
                .unwrap();
            assert!(epoch > 0);
            execute_sql(&mut conn,&format!("SET rtd.alt_writer_epoch={epoch}; INSERT INTO public.watermarks(pipeline,epoch_hi_inclusive,checkpoint_hi_inclusive,tx_hi,timestamp_ms_hi_inclusive,reader_lo,pruner_timestamp,pruner_hi,chain_id) VALUES('identity_sql',0,0,0,0,0,now(),0,decode('{}','hex')); DELETE FROM public.watermarks WHERE pipeline='identity_sql'","00".repeat(32))).await.unwrap();
            assert!(
                execute_sql(
                    &mut conn,
                    "DELETE FROM public.rtd_alt_writer_fence WHERE false"
                )
                .await
                .is_err()
            );
        }
        if profile == Profile::CoverageReader {
            assert!(
                query(
                    &mut conn,
                    "SELECT to_jsonb(count(*))::text AS value FROM public.kv_transactions"
                )
                .await
                .is_err()
            );
        }
        drop(conn);
        drop(own);
        let marker = fs::read(state.join("supplied.json")).unwrap();
        fs::remove_file(state.join("supplied.json")).unwrap();
        assert!(Native::load(&config, &sha, &state, id, profile, false).is_err());
        assert!(prepare(&config, &sha, &state, profile, false).is_err());
        private(&state.join("supplied.json"), &marker);
        let original_receipt = state.join(format!("receipt-{p}.json"));
        let receipt_bytes = fs::read(&original_receipt).unwrap();
        fs::remove_file(&original_receipt).unwrap();
        fs::remove_file(state.join("supplied.json")).unwrap();
        let reconcile = cli(
            &fixture,
            name,
            "plan",
            &config,
            &sha,
            &state,
            Some(id),
            None,
        )
        .await;
        assert!(reconcile.status.success());
        let reconciled: Value = serde_json::from_slice(&reconcile.stdout).unwrap();
        assert!(
            cli(
                &fixture,
                name,
                "apply",
                &config,
                &sha,
                &state,
                Some(id),
                reconciled["expectedPlanSha256"].as_str()
            )
            .await
            .status
            .success()
        );
        assert_eq!(fs::read(state.join("supplied.json")).unwrap(), marker);
        private(&original_receipt, receipt_bytes);
        for (bad, restore) in [
            (
                format!("ALTER ROLE {role} NOLOGIN"),
                format!("ALTER ROLE {role} LOGIN"),
            ),
            (
                format!("GRANT SELECT ON outside.protected TO {role}"),
                format!("REVOKE SELECT ON outside.protected FROM {role}"),
            ),
            (
                format!("GRANT pg_read_all_data TO {role}"),
                format!("REVOKE pg_read_all_data FROM {role}"),
            ),
            (
                format!(
                    "ALTER ROLE {role} IN DATABASE {} SET search_path=pg_catalog,public",
                    identifier(database.as_str().unwrap())
                ),
                format!(
                    "ALTER ROLE {role} IN DATABASE {} RESET ALL",
                    identifier(database.as_str().unwrap())
                ),
            ),
            (
                format!("ALTER ROLE {role} PASSWORD 'changed-password'"),
                format!(
                    "ALTER ROLE {role} PASSWORD {}",
                    literal(&postgres_protocol::password::scram_sha_256(
                        identity.password.as_bytes()
                    ))
                ),
            ),
        ] {
            execute_sql(&mut admin, &bad).await.unwrap();
            rejected_all(&fixture, name, &config, &sha, &state, id, p).await;
            fault_cases += 1;
            execute_sql(&mut admin, &restore).await.unwrap();
            assert!(
                cli(
                    &fixture,
                    name,
                    "verify",
                    &config,
                    &sha,
                    &state,
                    Some(id),
                    None
                )
                .await
                .status
                .success()
            );
        }
        execute_sql(
            &mut admin,
            &format!("DROP OWNED BY {role}; DROP ROLE {role}"),
        )
        .await
        .unwrap();
        rejected_all(&fixture, name, &config, &sha, &state, id, p).await;
        fault_cases += 1;
        execute_sql(
            &mut admin,
            &format!(
                "CREATE ROLE {role} LOGIN PASSWORD {}",
                literal(&identity.password)
            ),
        )
        .await
        .unwrap();
        rejected_all(&fixture, name, &config, &sha, &state, id, p).await;
        fault_cases += 1;
        assert_eq!(fs::read(state.join("identity.json")).unwrap(), raw);
    }
    let pending = root.join("stale");
    let prepared = prepare(&config, &sha, &pending, Profile::QueryReader, false).unwrap();
    let id = prepared["identitySha256"].as_str().unwrap();
    let old = cli(
        &fixture,
        "query-reader",
        "plan",
        &config,
        &sha,
        &pending,
        Some(id),
        None,
    )
    .await;
    assert!(old.status.success());
    let old: Value = serde_json::from_slice(&old.stdout).unwrap();
    execute_sql(
        &mut admin,
        "CREATE TABLE outside.changed_after_plan(value bigint)",
    )
    .await
    .unwrap();
    let rejected = cli(
        &fixture,
        "query-reader",
        "apply",
        &config,
        &sha,
        &pending,
        Some(id),
        old["expectedPlanSha256"].as_str(),
    )
    .await;
    assert_eq!(rejected.status.code(), Some(2));
    let exists = query(
        &mut admin,
        &format!(
            "SELECT to_jsonb(EXISTS(SELECT 1 FROM pg_roles WHERE rolname={}))::text AS value",
            literal(prepared["role"].as_str().unwrap())
        ),
    )
    .await
    .unwrap();
    assert_eq!(exists, false);
    assert!(!super::has_journal(&pending, "intent-").unwrap());
    let fresh = cli(
        &fixture,
        "query-reader",
        "plan",
        &config,
        &sha,
        &pending,
        Some(id),
        None,
    )
    .await;
    assert!(fresh.status.success());
    let fresh: Value = serde_json::from_slice(&fresh.stdout).unwrap();
    assert!(
        cli(
            &fixture,
            "query-reader",
            "apply",
            &config,
            &sha,
            &pending,
            Some(id),
            fresh["expectedPlanSha256"].as_str()
        )
        .await
        .status
        .success()
    );
    let result = json!({"passed":true,"actualTls":true,"profiles":3,"faultCases":fault_cases,"realWriterSql":true,"readersCannotWriteAfterReadOnlyDisabled":true,"suppliedMarkerLossRejected":true,"droppedAndReplacedRoleNeverResurrected":true,"oldReceiptsReverified":true,"sameIdentityUnknownCommitContinuation":true,"stalePlanCreatesNoRole":true,"canonicalMutated":false});
    private(&root.join("result.json"), result.to_string());
    assert_eq!(fault_cases, 21);
    assert!(
        execute(
            "unknown",
            "prepare",
            &config,
            &sha,
            &root.join("invalid"),
            None,
            None,
            false
        )
        .await
        .is_err()
    );
}
