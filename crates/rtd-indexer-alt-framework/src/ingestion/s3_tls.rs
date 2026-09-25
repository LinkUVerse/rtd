// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use anyhow::Context as _;
use anyhow::ensure;
use object_store::Certificate;
use object_store::ClientOptions;
use url::Url;

/// Add the private CA used by a self-hosted S3 endpoint. Rustls clients do
/// not automatically read SSL_CERT_FILE, so production callers must opt in.
pub fn with_archive_s3_ca(options: ClientOptions, endpoint: &Url) -> anyhow::Result<ClientOptions> {
    match std::env::var("RTD_ARCHIVE_S3_CA_CERT_FILE") {
        Ok(path) => with_ca_file(options, endpoint, Path::new(&path)),
        Err(std::env::VarError::NotPresent) => Ok(options),
        Err(error) => Err(error).context("invalid RTD_ARCHIVE_S3_CA_CERT_FILE"),
    }
}

fn with_ca_file(
    mut options: ClientOptions,
    endpoint: &Url,
    path: &Path,
) -> anyhow::Result<ClientOptions> {
    ensure!(
        endpoint.scheme() == "https",
        "RTD_ARCHIVE_S3_CA_CERT_FILE requires an HTTPS S3 endpoint"
    );
    ensure!(
        !path.as_os_str().is_empty(),
        "S3 CA certificate path is empty"
    );
    let pem = std::fs::read(path)
        .with_context(|| format!("cannot read S3 CA certificate {}", path.display()))?;
    let certificates = Certificate::from_pem_bundle(&pem)
        .context("S3 CA certificate file must contain PEM certificates")?;
    ensure!(!certificates.is_empty(), "S3 CA certificate file is empty");
    for certificate in certificates {
        options = options.with_root_certificate(certificate);
    }
    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_ca_requires_https_and_valid_pem() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("ca.pem");
        std::fs::write(
            &path,
            include_bytes!("../../../rtd-types/src/nitro_root_certificate.pem"),
        )
        .unwrap();
        let https = Url::parse("https://minio.internal:9000").unwrap();
        let http = Url::parse("http://minio.internal:9000").unwrap();
        assert!(with_ca_file(ClientOptions::default(), &https, &path).is_ok());
        assert!(with_ca_file(ClientOptions::default(), &http, &path).is_err());
        std::fs::write(&path, b"not a PEM certificate").unwrap();
        assert!(with_ca_file(ClientOptions::default(), &https, &path).is_err());
        std::fs::write(&path, b"").unwrap();
        assert!(with_ca_file(ClientOptions::default(), &https, &path).is_err());
        assert!(
            with_ca_file(
                ClientOptions::default(),
                &https,
                &directory.path().join("missing")
            )
            .is_err()
        );
    }
}
