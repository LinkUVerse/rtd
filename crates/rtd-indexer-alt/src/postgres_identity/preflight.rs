// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Cross-project checks against the real archive operation engine. Only the
//! fixture database and newly owned operation directories are changed.
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Output;

use rtd_indexer_alt_framework::postgres::Connection;
use serde_json::Value;
use serde_json::json;
use tokio::process::Command;

use super::cli;
use super::execute_sql;
use super::private;
use super::query;
use super::sha;

async fn archive(fixture: &Value, args: &[&str]) -> Output {
    tokio::time::timeout(
        std::time::Duration::from_secs(60),
        Command::new(fixture["nativeAdmin"].as_str().unwrap())
            .args(args)
            .env_clear()
            .env("PATH", "/no-tools")
            .env("RUST_LOG", "error")
            .kill_on_drop(true)
            .output(),
    )
    .await
    .unwrap()
    .unwrap()
}

fn pin(path: &Path) -> Value {
    json!({"path":path,"sha256":sha(&fs::read(path).unwrap())})
}

fn identity_files(state: &Path) -> BTreeMap<std::path::PathBuf, Vec<u8>> {
    fs::read_dir(state)
        .unwrap()
        .map(|e| {
            let p = e.unwrap().path();
            let bytes = fs::read(&p).unwrap();
            (p, bytes)
        })
        .collect()
}

pub(super) async fn check(
    fixture: &Value,
    profile: &str,
    config: &Path,
    config_sha: &str,
    state: &Path,
    identity: &str,
    admin: &mut Connection<'_>,
    role: &str,
) {
    let root = Path::new(fixture["evidence"].as_str().unwrap());
    let before = identity_files(state);
    let binding = cli(
        fixture,
        profile,
        "binding",
        config,
        config_sha,
        state,
        Some(identity),
        None,
    )
    .await;
    assert!(binding.status.success(), "native offline binding rejected");
    let again = cli(
        fixture,
        profile,
        "binding",
        config,
        config_sha,
        state,
        Some(identity),
        None,
    )
    .await;
    assert_eq!(binding.stdout, again.stdout);
    let exported: Value = serde_json::from_slice(&binding.stdout).unwrap();
    assert_eq!(exported["productionTls"], true);
    assert!(!String::from_utf8_lossy(&binding.stdout).contains("postgres://"));
    let binding_path = root.join(format!("{profile}-binding.json"));
    private(&binding_path, &binding.stdout);
    let kind = match profile {
        "writer" => "alt-writer-postgres",
        "query-reader" => "alt-query-postgres",
        _ => "alt-coverage-postgres",
    };
    for fail in [false, true] {
        let custody = root.join(format!(
            "preflight-{profile}-{}",
            if fail { "nologin" } else { "healthy" }
        ));
        fs::create_dir(&custody).unwrap();
        fs::set_permissions(&custody, fs::Permissions::from_mode(0o700)).unwrap();
        let entry = custody.join("entry.json");
        let entry_sha = private(&entry, json!({"version":1,"environmentId":"alt-pg-native","environmentDirectory":custody,"chainId":exported["chainId"],"resetScope":null,"readiness":null,"resetPreflight":null}).to_string());
        let request = custody.join("phase.json");
        let request_sha = private(&request, json!({"version":1,"kind":kind,"config":{"path":config,"sha256":config_sha},"stateDirectory":state,"identitySha256":identity,"nativeBinding":pin(&binding_path),"nativeExecutable":pin(Path::new(fixture["indexer"].as_str().unwrap())),"entry":pin(&entry),"executionTimeoutMs":35000,"shutdownTimeoutMs":1000}).to_string());
        if fail {
            execute_sql(admin, &format!("ALTER ROLE {role} NOLOGIN"))
                .await
                .unwrap();
        }
        let authority_sql = "SELECT jsonb_build_object('roles',(SELECT jsonb_agg(to_jsonb(r) ORDER BY r.oid) FROM pg_authid r),'acl',(SELECT jsonb_agg(jsonb_build_array(c.oid,c.relacl) ORDER BY c.oid) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public'))::text AS value";
        let authority = query(admin, authority_sql).await.unwrap();
        let generated = archive(
            fixture,
            &[
                "storage-verification-plan",
                "--request",
                request.to_str().unwrap(),
                "--expected-sha256",
                &request_sha,
            ],
        )
        .await;
        private(&custody.join("generated.out"), &generated.stdout);
        private(&custody.join("generated.err"), &generated.stderr);
        assert!(
            generated.status.success(),
            "Alt offline stage generation rejected"
        );
        let generated: Value = serde_json::from_slice(&generated.stdout).unwrap();
        assert_eq!(generated["remoteVerified"], false);
        assert_eq!(generated["plan"]["executable"], fixture["indexer"]);
        assert_eq!(generated["plan"]["arguments"][0], "postgres-identity");
        assert_eq!(generated["plan"]["arguments"][4], "verify");
        let marker = custody.join("later-service");
        let touch = fs::canonicalize("/usr/bin/touch").unwrap();
        let operation = custody.join("start.json");
        let operation_sha = private(&operation, json!({"version":1,"operationId":uuid::Uuid::new_v4(),"environmentId":"alt-pg-native","environmentDirectory":custody,"action":"start","gracefulTimeoutMs":1000,"stages":[generated["plan"],{"version":1,"executionTimeoutMs":1000,"shutdownTimeoutMs":1000,"environment":"alt-pg-native","role":"after-alt-preflight","executable":touch,"executableSha256":pin(&touch)["sha256"],"arguments":[marker],"environmentVariables":{},"workingDirectory":custody,"inputs":{entry.to_str().unwrap():entry_sha}}]}).to_string());
        let output = archive(
            fixture,
            &[
                "environment-entry",
                "start",
                "--config",
                entry.to_str().unwrap(),
                "--expected-sha256",
                &entry_sha,
                "--request",
                operation.to_str().unwrap(),
                "--expected-request-sha256",
                &operation_sha,
            ],
        )
        .await;
        private(&custody.join("start.out"), &output.stdout);
        private(&custody.join("start.err"), &output.stderr);
        if fail {
            assert_eq!(output.status.code(), Some(2));
            let report: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(
                report["detail"]["lastEvent"]["stage"],
                format!("alt-{profile}-postgres-preflight")
            );
            assert!(
                !marker.exists(),
                "revoked Alt identity admitted later service"
            );
        } else {
            assert!(output.status.success(), "healthy Alt preflight blocked");
            assert!(marker.exists());
        }
        assert!(
            authority == query(admin, authority_sql).await.unwrap(),
            "ordinary start changed database authority"
        );
        assert!(
            before == identity_files(state),
            "ordinary start changed identity files"
        );
        if fail {
            execute_sql(admin, &format!("ALTER ROLE {role} LOGIN"))
                .await
                .unwrap();
            assert!(
                cli(
                    fixture,
                    profile,
                    "verify",
                    config,
                    config_sha,
                    state,
                    Some(identity),
                    None
                )
                .await
                .status
                .success()
            );
        }
    }
    if profile == "coverage-reader" {
        private(&root.join("preflight-result.json"), json!({"passed":true,"profiles":3,"startOperations":6,"noRepair":true,"offlineBindingsStable":true,"blockedLaterService":true,"noInterpreterFallback":true,"sameFailedDirectoryRecoveryClaimed":false}).to_string());
    }
}
