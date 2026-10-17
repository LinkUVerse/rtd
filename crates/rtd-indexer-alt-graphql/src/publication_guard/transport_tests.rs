// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use fastcrypto::encoding::Base58;
use fastcrypto::encoding::Encoding;
use futures::StreamExt;
use prometheus::Registry;
use rcgen::BasicConstraints;
use rcgen::Certificate;
use rcgen::CertificateParams;
use rcgen::ExtendedKeyUsagePurpose;
use rcgen::IsCa;
use rcgen::KeyPair;
use rcgen::KeyUsagePurpose;
use rtd_indexer_alt_reader::pg_reader::PgReader;
use rtd_pg_db::DbArgs;
use rtd_types::digests::ChainIdentifier as NativeChainIdentifier;
use rustls::RootCertStore;
use rustls::ServerConfig;
use rustls::pki_types::PrivatePkcs8KeyDer;
use rustls::server::WebPkiClientVerifier;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::Notify;
use tokio::sync::oneshot;
use tokio::task::JoinSet;
use tokio_rustls::TlsAcceptor;

use crate::args::SubscriptionArgs;
use crate::task::chain_identifier;

use super::PublicationGuard;

#[derive(Default)]
struct State {
    mode: AtomicUsize,
    requests: AtomicUsize,
    stalled: Notify,
    cancelled: Notify,
}

fn ca() -> (Certificate, KeyPair) {
    let mut params = CertificateParams::new(Vec::<String>::new()).unwrap();
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::DigitalSignature,
    ];
    let key = KeyPair::generate().unwrap();
    (params.self_signed(&key).unwrap(), key)
}

fn identity(ca: &Certificate, key: &KeyPair, client: bool) -> (Certificate, KeyPair) {
    let mut params = CertificateParams::new(vec!["localhost".to_owned()]).unwrap();
    params.extended_key_usages = vec![if client {
        ExtendedKeyUsagePurpose::ClientAuth
    } else {
        ExtendedKeyUsagePurpose::ServerAuth
    }];
    let identity = KeyPair::generate().unwrap();
    (params.signed_by(&identity, ca, key).unwrap(), identity)
}

fn private(path: &Path, contents: impl AsRef<[u8]>) {
    std::fs::write(path, contents).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn configured_authority_requires_mtls_and_bounds_failure_cancellation_and_time() {
    let dir = tempfile::tempdir().unwrap();
    let (trusted, key) = ca();
    let (server_cert, server_key) = identity(&trusted, &key, false);
    let (client_cert, client_key) = identity(&trusted, &key, true);
    let (foreign, foreign_key) = ca();
    let (foreign_client, foreign_identity) = identity(&foreign, &foreign_key, true);
    let ca_path = dir.path().join("ca.pem");
    let client_path = dir.path().join("client.pem");
    private(&ca_path, trusted.pem());
    private(
        &client_path,
        format!("{}{}", client_cert.pem(), client_key.serialize_pem()),
    );
    let foreign_ca = dir.path().join("foreign-ca.pem");
    let foreign_client_path = dir.path().join("foreign-client.pem");
    private(&foreign_ca, foreign.pem());
    private(
        &foreign_client_path,
        format!(
            "{}{}",
            foreign_client.pem(),
            foreign_identity.serialize_pem()
        ),
    );

    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut roots = RootCertStore::empty();
    roots.add(trusted.der().clone()).unwrap();
    let verifier = WebPkiClientVerifier::builder_with_provider(Arc::new(roots), provider.clone())
        .build()
        .unwrap();
    let tls = ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_client_cert_verifier(verifier)
        .with_single_cert(
            vec![server_cert.der().clone()],
            PrivatePkcs8KeyDer::from(server_key.serialize_der()).into(),
        )
        .unwrap();
    let acceptor = TlsAcceptor::from(Arc::new(tls));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let url: url::Url = format!("https://localhost:{port}/authorize")
        .parse()
        .unwrap();
    let state = Arc::new(State::default());
    let source = state.clone();
    let chain = Base58::encode(NativeChainIdentifier::default().as_bytes());
    let (stop, stopped) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut connections = JoinSet::new();
        tokio::pin!(stopped);
        loop {
            tokio::select! {
                _ = &mut stopped => break,
                incoming = listener.accept() => {
                    let (socket, _) = incoming.unwrap();
                    let acceptor = acceptor.clone();
                    let source = source.clone();
                    let chain = chain.clone();
                    connections.spawn(async move {
                        let Ok(mut tls) = acceptor.accept(socket).await else { return; };
                        let mut request = Vec::new();
                        let mut byte = [0; 1];
                        while request.len() < 8192 && !request.ends_with(b"\r\n\r\n") {
                            if tls.read(&mut byte).await.unwrap_or(0) == 0 { return; }
                            request.push(byte[0]);
                        }
                        assert!(request.starts_with(b"GET /authorize HTTP/1.1\r\n"));
                        source.requests.fetch_add(1, Ordering::SeqCst);
                        let mode = source.mode.load(Ordering::SeqCst);
                        if mode == 5 {
                            source.stalled.notify_one();
                            let _ = tls.read(&mut byte).await;
                            source.cancelled.notify_one();
                            return;
                        }
                        let status = match mode { 1 => "503 Unavailable", 4 => "302 Found", _ => "204 No Content" };
                        let chain = if mode == 2 { "other-chain" } else { &chain };
                        let extra = match mode { 3 => "x-rtd-visible-checkpoint: 7\r\n", 4 => "location: /elsewhere\r\n", _ => "" };
                        let response = format!("HTTP/1.1 {status}\r\nx-rtd-chain-id: {chain}\r\nx-rtd-visible-checkpoint: 7\r\n{extra}content-length: 0\r\nconnection: close\r\n\r\n");
                        let _ = tls.write_all(response.as_bytes()).await;
                        let _ = tls.shutdown().await;
                    });
                }
                Some(result) = connections.join_next(), if !connections.is_empty() => { result.unwrap(); }
            }
        }
        connections.abort_all();
        while connections.join_next().await.is_some() {}
    });
    let pg = PgReader::new(None, None, DbArgs::default(), &Registry::new())
        .await
        .unwrap();
    let (chain, _chain_task) = chain_identifier::task(pg, Duration::from_millis(10));
    let args = SubscriptionArgs {
        checkpoint_stream_url: Some("http://127.0.0.1:9".parse().unwrap()),
        publication_guard_url: Some(url.clone()),
        publication_guard_ca: Some(ca_path),
        publication_guard_identity: Some(client_path),
    };
    let guard = PublicationGuard::from_args(&args, &chain).await.unwrap();
    assert_eq!(guard.height(Some(7)).await.unwrap(), Some(7));
    let anonymous = reqwest::Client::builder()
        .no_proxy()
        .tls_built_in_root_certs(false)
        .add_root_certificate(reqwest::Certificate::from_pem(trusted.pem().as_bytes()).unwrap())
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    assert!(anonymous.get(url.clone()).send().await.is_err());
    for bad in [
        SubscriptionArgs {
            publication_guard_ca: Some(foreign_ca),
            ..args.clone()
        },
        SubscriptionArgs {
            publication_guard_identity: Some(foreign_client_path),
            ..args.clone()
        },
        SubscriptionArgs {
            publication_guard_url: Some(
                format!("https://127.0.0.1:{port}/authorize")
                    .parse()
                    .unwrap(),
            ),
            ..args.clone()
        },
    ] {
        assert!(
            PublicationGuard::from_args(&bad, &chain)
                .await
                .unwrap()
                .height(Some(7))
                .await
                .is_err()
        );
    }
    for mode in 1..=4 {
        state.mode.store(mode, Ordering::SeqCst);
        let before = state.requests.load(Ordering::SeqCst);
        assert!(guard.height(Some(7)).await.is_err());
        assert_eq!(state.requests.load(Ordering::SeqCst), before + 1);
    }
    state.mode.store(0, Ordering::SeqCst);
    assert_eq!(guard.height(Some(7)).await.unwrap(), Some(7));

    state.mode.store(5, Ordering::SeqCst);
    let stream = guard
        .clone()
        .protect_follow(futures::stream::pending(), Some(7), None);
    let reading = tokio::spawn(async move {
        futures::pin_mut!(stream);
        stream.next().await
    });
    tokio::time::timeout(Duration::from_secs(3), state.stalled.notified())
        .await
        .unwrap();
    reading.abort();
    assert!(reading.await.unwrap_err().is_cancelled());
    tokio::time::timeout(Duration::from_secs(3), state.cancelled.notified())
        .await
        .unwrap();

    let started = Instant::now();
    assert!(
        tokio::time::timeout(Duration::from_secs(22), guard.height(Some(7)))
            .await
            .unwrap()
            .is_err()
    );
    assert!(started.elapsed() >= Duration::from_secs(17));
    tokio::time::timeout(Duration::from_secs(3), state.cancelled.notified())
        .await
        .unwrap();
    stop.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(3), server)
        .await
        .unwrap()
        .unwrap();
}
