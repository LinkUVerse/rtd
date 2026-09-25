// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::env;
use std::path::Path;
use std::time::Duration;

use anyhow::Context;
use anyhow::ensure;
use rtd_rpc::Client;
use tonic::transport::Certificate;
use tonic::transport::ClientTlsConfig;
use tonic::transport::Endpoint;
use tonic::transport::Identity;
use tonic::transport::Uri;

/// The SDK client does not expose its TLS configuration. Keep its transport
/// defaults when a private CA requires constructing a custom endpoint.
fn sdk_endpoint(uri: Uri, tls: ClientTlsConfig) -> anyhow::Result<Endpoint> {
    Ok(Endpoint::from(uri)
        .tls_config(tls)?
        .connect_timeout(Duration::from_secs(5))
        .tcp_keepalive(Some(Duration::from_secs(15)))
        .tcp_keepalive_interval(Some(Duration::from_secs(5)))
        .tcp_keepalive_retries(Some(3))
        .http2_keep_alive_interval(Duration::from_secs(5))
        .keep_alive_timeout(Duration::from_secs(20))
        .initial_stream_window_size(2 * 1024 * 1024)
        .initial_connection_window_size(64 * 1024 * 1024))
}

fn environment_path(name: &str) -> anyhow::Result<Option<String>> {
    match env::var(name) {
        Ok(path) if !path.is_empty() => Ok(Some(path)),
        Ok(_) => anyhow::bail!("{name} must not be empty"),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(error).with_context(|| format!("invalid {name}")),
    }
}

fn tls_from_paths(
    uri: &Uri,
    ca: Option<&Path>,
    cert: Option<&Path>,
    key: Option<&Path>,
) -> anyhow::Result<Option<ClientTlsConfig>> {
    ensure!(
        cert.is_some() == key.is_some(),
        "gRPC client certificate and key must be paired"
    );
    ensure!(
        ca.is_some() || cert.is_none(),
        "gRPC client identity requires an explicit CA"
    );
    if ca.is_none() {
        return Ok(None);
    }
    ensure!(
        uri.scheme_str() == Some("https"),
        "private gRPC TLS requires an https endpoint"
    );
    let ca = ca.expect("checked above");
    let pem = std::fs::read(ca).with_context(|| format!("read gRPC CA {}", ca.display()))?;
    ensure!(!pem.is_empty(), "gRPC CA PEM must not be empty");
    let mut tls = ClientTlsConfig::new().ca_certificate(Certificate::from_pem(pem));
    if let (Some(cert), Some(key)) = (cert, key) {
        let cert_pem = std::fs::read(cert)
            .with_context(|| format!("read gRPC client certificate {}", cert.display()))?;
        let key_pem = std::fs::read(key)
            .with_context(|| format!("read gRPC client key {}", key.display()))?;
        ensure!(
            !cert_pem.is_empty() && !key_pem.is_empty(),
            "gRPC client identity PEM must not be empty"
        );
        tls = tls.identity(Identity::from_pem(cert_pem, key_pem));
    }
    Ok(Some(tls))
}

fn configured_tls(uri: &Uri, prefix: &str) -> anyhow::Result<Option<ClientTlsConfig>> {
    let ca = environment_path(&format!("{prefix}_CA_CERT_FILE"))?;
    let cert = environment_path(&format!("{prefix}_CLIENT_CERT_FILE"))?;
    let key = environment_path(&format!("{prefix}_CLIENT_KEY_FILE"))?;
    tls_from_paths(
        uri,
        ca.as_deref().map(Path::new),
        cert.as_deref().map(Path::new),
        key.as_deref().map(Path::new),
    )
}

pub(crate) fn sdk_client(uri: Uri, prefix: &str) -> anyhow::Result<Client> {
    match configured_tls(&uri, prefix)? {
        Some(tls) => Ok(Client::from_endpoint(&sdk_endpoint(uri, tls)?)),
        None => Client::new(uri).context("create gRPC SDK client"),
    }
}

pub(crate) fn channel_endpoint(uri: Uri, prefix: &str) -> anyhow::Result<Endpoint> {
    let tls = configured_tls(&uri, prefix)?;
    let mut endpoint = Endpoint::from(uri.clone());
    if let Some(tls) = tls {
        endpoint = endpoint.tls_config(tls)?;
    } else if uri.scheme_str() == Some("https") {
        endpoint = endpoint.tls_config(ClientTlsConfig::new().with_enabled_roots())?;
    }
    Ok(endpoint)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::tls_from_paths;

    #[test]
    fn tls_requires_https_and_complete_client_identity() {
        let ca = Path::new("/nonexistent/rtd-ca.pem");
        let cert = Path::new("/nonexistent/rtd-client.pem");
        let http = "http://localhost:7001".parse().unwrap();
        let https = "https://localhost:7001".parse().unwrap();
        assert!(tls_from_paths(&http, Some(ca), None, None).is_err());
        assert!(tls_from_paths(&https, Some(ca), Some(cert), None).is_err());
        assert!(tls_from_paths(&https, None, Some(cert), Some(cert)).is_err());
        assert!(tls_from_paths(&https, Some(ca), None, None).is_err());
        assert!(tls_from_paths(&http, None, None, None).unwrap().is_none());
    }
}
