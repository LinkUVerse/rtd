// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use axum::Router;
use axum::extract::Request;
use axum::response::IntoResponse;
use axum_server::Handle;
use axum_server::tls_rustls::RustlsConfig;
use futures::future::BoxFuture;
use linku_network::request_log::GrpcRequestLogLayer;
use metrics::RpcMetrics;
use middleware::metrics::MakeMetricsHandler;
use middleware::panic::CatchPanicLayer;
use middleware::version::Version;
use prometheus::Registry;
use rtd_futures::service::Service;
use rtd_http::middleware::callback::CallbackLayer;
use rustls::RootCertStore;
use rustls::ServerConfig;
use rustls::pki_types::CertificateDer;
use rustls::pki_types::PrivateKeyDer;
use rustls::pki_types::pem::PemObject;
use rustls::server::WebPkiClientVerifier;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tonic::server::NamedService;
use tonic_health::ServingStatus;
use tracing::info;

pub(crate) mod consistent_service;
mod error;
mod metrics;
mod middleware;
pub(crate) mod pagination;
pub(crate) mod state;
mod type_filter;

#[derive(clap::Args, Clone, Debug)]
pub struct RpcArgs {
    /// Address to accept incoming RPC connections on.
    #[clap(long, default_value_t = Self::default().rpc_listen_address)]
    pub rpc_listen_address: SocketAddr,

    /// TLS configuration
    #[clap(flatten)]
    pub tls: TlsArgs,
}

#[derive(clap::Args, Clone, Debug, Default)]
pub struct TlsArgs {
    /// Address to accept incoming TLS/HTTPS connections on
    #[clap(long, requires_all = &["tls_cert", "tls_key"])]
    pub rpc_tls_listen_address: Option<SocketAddr>,

    /// Path to TLS certificate file (PEM format)
    #[clap(long, requires_all = &["rpc_tls_listen_address", "tls_key"])]
    pub tls_cert: Option<PathBuf>,

    /// Path to TLS private key file (PEM format)
    #[clap(long, requires_all = &["rpc_tls_listen_address", "tls_cert"])]
    pub tls_key: Option<PathBuf>,

    /// Client CA for mutual TLS on the HTTPS listener.
    #[clap(long, requires_all = &["rpc_tls_listen_address", "tls_cert", "tls_key"])]
    pub tls_client_ca: Option<PathBuf>,
}

/// Responsible for the set-up of a gRPC service -- adding services, configuring reflection,
/// health-checks, logging and metrics middleware, etc.
pub(crate) struct RpcService<'d> {
    /// Address to accept incoming RPC connections on.
    rpc_listen_address: SocketAddr,

    /// Optional address to accept incoming TLS RPC connections on.
    rpc_tls_listen_address: Option<SocketAddr>,

    /// TLS configuration
    tls_config: Option<RustlsConfig>,

    /// The version string to report with each response, as an HTTP header.
    version: &'static str,

    /// File descriptors are added to these builders to eventually be exposed via the reflection
    /// service.
    reflection_v1: tonic_reflection::server::Builder<'d>,
    reflection_v1alpha: tonic_reflection::server::Builder<'d>,

    /// The same file descriptor sets, retained to build the request-log middleware's descriptor
    /// pool, so it cannot drift from what the reflection service exposes.
    file_descriptor_sets: Vec<&'d [u8]>,

    /// Names of gRPC services and associated readiness futures registered with this instance.
    service_futures: Vec<(&'static str, BoxFuture<'static, ()>)>,

    /// The axum router that wil handle incoming requests.
    router: Router,

    /// Metrics for the RPC service.
    metrics: Arc<RpcMetrics>,
}

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

impl<'d> RpcService<'d> {
    pub(crate) async fn new(
        args: RpcArgs,
        version: &'static str,
        registry: &Registry,
    ) -> anyhow::Result<Self> {
        let RpcArgs {
            rpc_listen_address,
            tls,
        } = args;

        let TlsArgs {
            rpc_tls_listen_address,
            tls_cert,
            tls_key,
            tls_client_ca,
        } = tls;

        let tls_config = if let (Some(cert), Some(key)) = (tls_cert, tls_key) {
            Some(if let Some(client_ca) = tls_client_ca {
                mutual_tls_config(&cert, &key, &client_ca)
                    .context("Failed to load mutual TLS configuration")?
            } else {
                RustlsConfig::from_pem_file(cert, key)
                    .await
                    .context("Failed to load TLS configuration")?
            })
        } else {
            None
        };

        Ok(Self {
            rpc_listen_address,
            rpc_tls_listen_address,
            tls_config,
            version,
            reflection_v1: tonic_reflection::server::Builder::configure(),
            reflection_v1alpha: tonic_reflection::server::Builder::configure(),
            file_descriptor_sets: vec![],
            service_futures: vec![],
            router: Router::new(),
            metrics: Arc::new(RpcMetrics::new(registry)),
        })
    }

    /// Register a file descriptor set to be exposed via the reflection service.
    pub(crate) fn register_encoded_file_descriptor_set(mut self, fds: &'d [u8]) -> Self {
        self.reflection_v1 = self.reflection_v1.register_encoded_file_descriptor_set(fds);
        self.reflection_v1alpha = self
            .reflection_v1alpha
            .register_encoded_file_descriptor_set(fds);
        self.file_descriptor_sets.push(fds);
        self
    }

    /// Register a new gRPC service.
    pub(crate) fn add_service<S, F>(mut self, s: S, ready: F) -> Self
    where
        S: Clone + Send + Sync + 'static,
        S: NamedService,
        S: tower::Service<Request, Response: IntoResponse, Error = Infallible>,
        S::Future: Send + 'static,
        S::Error: Send + Into<BoxError>,
        F: Future<Output = ()> + Send + 'static,
    {
        self.service_futures.push((S::NAME, Box::pin(ready)));
        self.router = add_service(self.router, s);
        self
    }

    /// Run the RPC service. This binds the listener and exposes handlers for the RPC service.
    pub(crate) async fn run(self) -> anyhow::Result<Service> {
        let Self {
            rpc_listen_address,
            rpc_tls_listen_address,
            tls_config,
            version,
            reflection_v1,
            reflection_v1alpha,
            file_descriptor_sets,
            service_futures,
            mut router,
            metrics,
        } = self;

        let request_log = GrpcRequestLogLayer::from_encoded_file_descriptor_sets(
            file_descriptor_sets
                .iter()
                .copied()
                .chain([tonic_health::pb::FILE_DESCRIPTOR_SET]),
        )
        .context("Failed to build request-log descriptor pool")?;

        let reflection_v1 = reflection_v1
            .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
            .build_v1()
            .unwrap();

        let reflection_v1alpha = reflection_v1alpha
            .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
            .build_v1alpha()
            .unwrap();

        let (health_reporter, health_service) = tonic_health::server::health_reporter();

        let internal_services = vec![
            service_name(&reflection_v1),
            service_name(&reflection_v1alpha),
            service_name(&health_service),
        ];

        router = add_service(router, reflection_v1);
        router = add_service(router, reflection_v1alpha);
        router = add_service(router, health_service);
        router = router
            .layer(request_log)
            .layer(CallbackLayer::new(MakeMetricsHandler::new(metrics.clone())))
            .layer(axum::middleware::from_fn_with_state(
                Version(version),
                middleware::version::set_version,
            ))
            .layer(CatchPanicLayer::new(metrics));

        for service_name in internal_services {
            health_reporter
                .set_service_status(service_name, ServingStatus::Serving)
                .await;
        }

        // Create a Service to be attached as secondary to the main service
        let mut readiness_checks = Service::new();

        for (name, ready) in service_futures {
            health_reporter
                .set_service_status(name, ServingStatus::NotServing)
                .await;

            let reporter = health_reporter.clone();
            readiness_checks = readiness_checks.spawn(async move {
                ready.await;
                reporter
                    .set_service_status(name, ServingStatus::Serving)
                    .await;
                info!("gRPC service {name} is now SERVING");
                Ok(())
            });
        }

        let mut service = Service::new();
        service = service.attach(readiness_checks);

        // Start HTTPS server if TLS is configured
        if let (Some(listen_address), Some(config)) = (rpc_tls_listen_address, tls_config) {
            info!("Starting Consistent RPC TLS service on {listen_address}");
            let handle = Handle::new();
            let tls_router = router.clone();

            service = service
                .with_shutdown_signal({
                    let handle = handle.clone();
                    async move {
                        handle.graceful_shutdown(None);
                    }
                })
                .spawn(async move {
                    axum_server::bind_rustls(listen_address, config)
                        .handle(handle)
                        .serve(tls_router.into_make_service())
                        .await
                        .context("Consistent RPC TLS service failed")?;
                    Ok(())
                });
        }

        // Start HTTP server
        info!("Starting Consistent RPC service on {rpc_listen_address}");
        let listener = TcpListener::bind(rpc_listen_address)
            .await
            .context("Failed to bind Consistent RPC to listen address")?;

        let (stx, srx) = oneshot::channel::<()>();
        service = service
            .with_shutdown_signal(async move {
                let _ = stx.send(());
            })
            .spawn(async move {
                axum::serve(listener, router)
                    .with_graceful_shutdown(async move {
                        let _ = srx.await;
                    })
                    .await
                    .context("Consistent RPC HTTP service failed")
            });

        Ok(service)
    }
}

fn mutual_tls_config(
    cert: &PathBuf,
    key: &PathBuf,
    client_ca: &PathBuf,
) -> anyhow::Result<RustlsConfig> {
    let certs = CertificateDer::pem_file_iter(cert)
        .context("failed to open Consistent TLS certificate")?
        .collect::<Result<Vec<_>, _>>()
        .context("failed to parse Consistent TLS certificate")?;
    anyhow::ensure!(!certs.is_empty(), "Consistent TLS certificate is empty");
    let private_key =
        PrivateKeyDer::from_pem_file(key).context("failed to read Consistent TLS private key")?;
    let client_certs = CertificateDer::pem_file_iter(client_ca)
        .context("failed to open Consistent client CA")?
        .collect::<Result<Vec<_>, _>>()
        .context("failed to parse Consistent client CA")?;
    anyhow::ensure!(!client_certs.is_empty(), "Consistent client CA is empty");
    let mut roots = RootCertStore::empty();
    for client_cert in client_certs {
        roots
            .add(client_cert)
            .context("invalid Consistent client CA")?;
    }
    let verifier = WebPkiClientVerifier::builder(Arc::new(roots)).build()?;
    let mut tls =
        ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_protocol_versions(rustls::DEFAULT_VERSIONS)?
            .with_client_cert_verifier(verifier)
            .with_single_cert(certs, private_key)?;
    tls.alpn_protocols = vec![b"h2".to_vec()];
    Ok(RustlsConfig::from_config(Arc::new(tls)))
}

impl Default for RpcArgs {
    fn default() -> Self {
        Self {
            rpc_listen_address: "0.0.0.0:7001".parse().unwrap(),
            tls: TlsArgs::default(),
        }
    }
}

fn service_name<S: NamedService>(_: &S) -> &'static str {
    S::NAME
}

fn add_service<S>(router: Router, s: S) -> Router
where
    S: Clone + Send + Sync + 'static,
    S: NamedService,
    S: tower::Service<Request, Response: IntoResponse, Error = Infallible>,
    S::Future: Send + 'static,
    S::Error: Send + Into<BoxError>,
{
    router.route_service(&format!("/{}/{{*rest}}", S::NAME), s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcgen::BasicConstraints;
    use rcgen::CertificateParams;
    use rcgen::IsCa;
    use rcgen::KeyPair;

    /// `run` builds the request-log layer's descriptor pool from the registered file descriptor
    /// sets plus `tonic_health`'s, so they must always merge into one valid pool.
    #[test]
    fn request_log_pool_builds_from_registered_file_descriptor_sets() {
        GrpcRequestLogLayer::from_encoded_file_descriptor_sets([
            rtd_indexer_alt_consistent_api::proto::rpc::consistent::v1alpha::FILE_DESCRIPTOR_SET,
            tonic_health::pb::FILE_DESCRIPTOR_SET,
        ])
        .unwrap();
    }

    #[tokio::test]
    async fn mutual_tls_config_accepts_ca_and_rejects_empty_trust_roots() {
        let directory = tempfile::tempdir().unwrap();
        let cert_file = directory.path().join("server.crt");
        let key_file = directory.path().join("server.key");
        let ca_file = directory.path().join("clients.ca.crt");
        let mut ca_params = CertificateParams::new(vec![]).unwrap();
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        let ca_key = KeyPair::generate().unwrap();
        let ca = ca_params.self_signed(&ca_key).unwrap();
        let server_key = KeyPair::generate().unwrap();
        let server = CertificateParams::new(vec!["localhost".to_owned()])
            .unwrap()
            .signed_by(&server_key, &ca, &ca_key)
            .unwrap();
        std::fs::write(&cert_file, server.pem()).unwrap();
        std::fs::write(&key_file, server_key.serialize_pem()).unwrap();
        std::fs::write(&ca_file, ca.pem()).unwrap();

        let config = mutual_tls_config(&cert_file, &key_file, &ca_file).unwrap();
        assert_eq!(config.get_inner().alpn_protocols, vec![b"h2".to_vec()]);
        std::fs::write(&ca_file, "").unwrap();
        assert!(mutual_tls_config(&cert_file, &key_file, &ca_file).is_err());
    }
}
