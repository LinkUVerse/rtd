// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use object_store::aws::AmazonS3ConfigKey;
use object_store::path::Path;
use object_store::{ClientOptions, ObjectStore, ObjectStoreExt, RetryConfig};
use rtd_types::messages_checkpoint::CheckpointSequenceNumber;
use std::str::FromStr;
use std::time::Duration;
use url::Url;

pub fn create_remote_store_client(
    url: String,
    remote_store_options: Vec<(String, String)>,
    timeout_secs: u64,
) -> Result<Box<dyn ObjectStore>> {
    let retry_config = RetryConfig {
        max_retries: 10,
        retry_timeout: Duration::from_secs(timeout_secs + 1),
        ..Default::default()
    };
    let client_options = ClientOptions::new()
        .with_timeout(Duration::from_secs(timeout_secs))
        .with_allow_http(true);
    let url = Url::parse(&url)?;
    let mut scheme = url.scheme();
    if url.host_str().unwrap_or_default().starts_with("s3") {
        scheme = "s3";
    }
    match scheme {
        "http" | "https" => {
            validate_private_endpoint(&url)?;
            let http_store = object_store::http::HttpBuilder::new()
                .with_url(url)
                .with_client_options(client_options)
                .with_retry(retry_config)
                .build()?;
            Ok(Box::new(http_store))
        }
        "gs" => Err(anyhow::anyhow!(
            "GCS is not an RTD ingestion source; use self-hosted Ceph RGW"
        )),
        "s3" => {
            let option = |name: &str| {
                remote_store_options
                    .iter()
                    .find(|(key, _)| key == name)
                    .map(|(_, value)| value.as_str())
            };
            let required = |name: &str, fallback: &str| -> Result<String> {
                option(name)
                    .map(str::to_owned)
                    .or_else(|| std::env::var(fallback).ok())
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| anyhow::anyhow!("Explicit self-hosted S3 {name} is required"))
            };
            let endpoint = required("aws_endpoint", "AWS_ENDPOINT")?;
            validate_private_endpoint(&Url::parse(&endpoint)?)?;
            let access_key = required("aws_access_key_id", "AWS_ACCESS_KEY_ID")?;
            let secret_key = required("aws_secret_access_key", "AWS_SECRET_ACCESS_KEY")?;
            let region = required("aws_region", "AWS_DEFAULT_REGION")?;
            let mut builder = object_store::aws::AmazonS3Builder::new()
                .with_url(url.as_str())
                .with_retry(retry_config)
                .with_client_options(client_options);
            for (key, value) in remote_store_options {
                builder = builder.with_config(AmazonS3ConfigKey::from_str(&key)?, value);
            }
            builder = builder
                .with_endpoint(endpoint)
                .with_access_key_id(access_key)
                .with_secret_access_key(secret_key)
                .with_region(region);
            Ok(Box::new(builder.build()?))
        }
        "file" => Ok(Box::new(
            object_store::local::LocalFileSystem::new_with_prefix(url.path())?,
        )),
        _ => Err(anyhow::anyhow!("Unsupported URL scheme: {}", url.scheme())),
    }
}

fn validate_private_endpoint(url: &Url) -> Result<()> {
    let host = url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("RTD archive URL needs a host"))?
        .trim_end_matches('.');
    anyhow::ensure!(
        matches!(url.scheme(), "http" | "https"),
        "RTD archive URL must use HTTP(S)"
    );
    anyhow::ensure!(
        !["amazonaws.com", "googleapis.com", "blob.core.windows.net"]
            .iter()
            .any(|suffix| host == *suffix || host.ends_with(&format!(".{suffix}"))),
        "RTD archive URL must point to self-hosted storage"
    );
    anyhow::ensure!(
        url.scheme() == "https"
            || std::env::var("RTD_ARCHIVE_ALLOW_INSECURE_S3_DEV").as_deref() == Ok("1"),
        "Plaintext RTD archive URLs are restricted to explicit local development"
    );
    Ok(())
}

pub async fn end_of_epoch_data(
    url: String,
    remote_store_options: Vec<(String, String)>,
    timeout_secs: u64,
) -> Result<Vec<CheckpointSequenceNumber>> {
    let client = create_remote_store_client(url, remote_store_options, timeout_secs)?;
    let response = client.get(&Path::from("epochs.json")).await?;
    Ok(serde_json::from_slice(response.bytes().await?.as_ref())?)
}

#[cfg(test)]
mod tests {
    use super::create_remote_store_client;
    use super::validate_private_endpoint;
    use url::Url;

    #[test]
    fn cloud_ingestion_sources_fail_closed() {
        assert!(create_remote_store_client("gs://bucket".into(), vec![], 5).is_err());
        assert!(
            create_remote_store_client("https://storage.googleapis.com/rtd".into(), vec![], 5)
                .is_err()
        );
        assert!(
            create_remote_store_client("https://storage.googleapis.com./rtd".into(), vec![], 5)
                .is_err()
        );
        assert!(
            validate_private_endpoint(&Url::parse("https://ceph.rtd.internal").unwrap()).is_ok()
        );
    }
}
