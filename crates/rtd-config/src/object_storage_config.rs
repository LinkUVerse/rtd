// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result, anyhow};

use clap::*;
use object_store::aws::AmazonS3Builder;
use object_store::{ClientOptions, DynObjectStore};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, fs};
use tracing::info;

/// Object-store type.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, ValueEnum)]
pub enum ObjectStoreType {
    /// Local file system
    File,
    /// AWS S3
    S3,
    /// Google Cloud Store
    GCS,
    /// Azure Blob Store
    Azure,
}

#[derive(Default, Debug, Clone, Deserialize, Serialize, Args)]
#[serde(rename_all = "kebab-case")]
pub struct ObjectStoreConfig {
    /// Which object storage to use. If not specified, defaults to local file system.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(value_enum)]
    pub object_store: Option<ObjectStoreType>,
    /// Path of the local directory. Only relevant is `--object-store` is File
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub directory: Option<PathBuf>,
    /// Name of the bucket to use for the object store. Must also set
    /// `--object-store` to a cloud object storage to have any effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub bucket: Option<String>,
    /// When using Amazon S3 as the object store, set this to an access key that
    /// has permission to read from and write to the specified S3 bucket.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub aws_access_key_id: Option<String>,
    /// When using Amazon S3 as the object store, set this to the secret access
    /// key that goes with the specified access key ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub aws_secret_access_key: Option<String>,
    /// When using Amazon S3 as the object store, set this to bucket endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub aws_endpoint: Option<String>,
    /// When using Amazon S3 as the object store, set this to the region
    /// that goes with the specified bucket
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub aws_region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub aws_profile: Option<String>,
    /// Enable virtual hosted style requests
    #[serde(default)]
    #[arg(long, default_value_t = true)]
    pub aws_virtual_hosted_style_request: bool,
    /// Allow unencrypted HTTP connection to AWS.
    #[serde(default)]
    #[arg(long, default_value_t = true)]
    pub aws_allow_http: bool,
    /// When using Google Cloud Storage as the object store, set this to the
    /// path to the JSON file that contains the Google credentials.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub google_service_account: Option<String>,
    /// When using Google Cloud Storage as the object store and writing to a
    /// bucket with Requester Pays enabled, set this to the project_id
    /// you want to associate the write cost with.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub google_project_id: Option<String>,
    /// When using Microsoft Azure as the object store, set this to the
    /// azure account name
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub azure_storage_account: Option<String>,
    /// When using Microsoft Azure as the object store, set this to one of the
    /// keys in storage account settings
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub azure_storage_access_key: Option<String>,
    #[serde(default = "default_object_store_connection_limit")]
    #[arg(long, default_value_t = 20)]
    pub object_store_connection_limit: usize,
    #[serde(default)]
    #[arg(long, default_value_t = false)]
    pub no_sign_request: bool,
}

fn default_object_store_connection_limit() -> usize {
    20
}

fn no_timeout_options() -> ClientOptions {
    ClientOptions::new()
        .with_timeout_disabled()
        .with_connect_timeout_disabled()
        .with_pool_idle_timeout(std::time::Duration::from_secs(300))
}

impl ObjectStoreConfig {
    fn new_local_fs(&self) -> Result<Arc<DynObjectStore>, anyhow::Error> {
        info!(directory=?self.directory, object_store_type="File", "Object Store");
        if let Some(path) = &self.directory {
            fs::create_dir_all(path).context(anyhow!(
                "Failed to create local directory: {}",
                path.display()
            ))?;
            let store = object_store::local::LocalFileSystem::new_with_prefix(path)
                .context(anyhow!("Failed to create local object store"))?;
            Ok(Arc::new(store))
        } else {
            Err(anyhow!("No directory provided for local fs storage"))
        }
    }
    fn new_s3(&self) -> Result<Arc<DynObjectStore>, anyhow::Error> {
        use object_store::limit::LimitStore;

        info!(bucket=?self.bucket, object_store_type="S3", "Object Store");

        let endpoint = self
            .aws_endpoint
            .as_deref()
            .context("Self-hosted S3 endpoint is required")?;
        validate_self_hosted_endpoint(endpoint)?;
        let region = self
            .aws_region
            .as_deref()
            .filter(|value| !value.is_empty())
            .context("Self-hosted S3 region is required")?;
        let bucket = self
            .bucket
            .as_deref()
            .filter(|value| !value.is_empty())
            .context("Self-hosted S3 bucket is required")?;
        let access_key_id = self
            .aws_access_key_id
            .clone()
            .or_else(|| env::var("ARCHIVE_READ_AWS_ACCESS_KEY_ID").ok())
            .or_else(|| env::var("FORMAL_SNAPSHOT_WRITE_AWS_ACCESS_KEY_ID").ok())
            .or_else(|| env::var("DB_SNAPSHOT_READ_AWS_ACCESS_KEY_ID").ok())
            .filter(|value| !value.is_empty())
            .context("Explicit self-hosted S3 access key is required")?;
        let secret_access_key = self
            .aws_secret_access_key
            .clone()
            .or_else(|| env::var("ARCHIVE_READ_AWS_SECRET_ACCESS_KEY").ok())
            .or_else(|| env::var("FORMAL_SNAPSHOT_WRITE_AWS_SECRET_ACCESS_KEY").ok())
            .or_else(|| env::var("DB_SNAPSHOT_READ_AWS_SECRET_ACCESS_KEY").ok())
            .filter(|value| !value.is_empty())
            .context("Explicit self-hosted S3 secret is required")?;

        let mut builder = AmazonS3Builder::new()
            .with_client_options(no_timeout_options())
            .with_region(region)
            .with_bucket_name(bucket)
            .with_access_key_id(access_key_id)
            .with_secret_access_key(secret_access_key)
            .with_endpoint(endpoint);

        if self.aws_virtual_hosted_style_request {
            builder = builder.with_virtual_hosted_style_request(true);
        }
        if self.aws_allow_http {
            builder = builder.with_allow_http(true);
        }
        Ok(Arc::new(LimitStore::new(
            builder.build().context("Invalid s3 config")?,
            self.object_store_connection_limit,
        )))
    }
    pub fn make(&self) -> Result<Arc<DynObjectStore>, anyhow::Error> {
        match &self.object_store {
            Some(ObjectStoreType::File) => self.new_local_fs(),
            Some(ObjectStoreType::S3) => self.new_s3(),
            Some(ObjectStoreType::GCS | ObjectStoreType::Azure) => Err(anyhow!(
                "RTD only permits local or self-hosted S3 object storage"
            )),
            _ => Err(anyhow!("At least one storage backend should be provided")),
        }
    }
}

pub fn validate_self_hosted_endpoint(endpoint: &str) -> Result<()> {
    let url = Url::parse(endpoint).context("Self-hosted S3 endpoint must be a URL")?;
    let host = url
        .host_str()
        .context("Self-hosted S3 endpoint needs a host")?
        .trim_end_matches('.');
    anyhow::ensure!(
        matches!(url.scheme(), "http" | "https"),
        "S3 endpoint must use HTTP(S)"
    );
    anyhow::ensure!(
        !["amazonaws.com", "googleapis.com", "blob.core.windows.net"]
            .iter()
            .any(|suffix| host == *suffix || host.ends_with(&format!(".{suffix}"))),
        "S3 endpoint must point to RTD self-hosted storage"
    );
    anyhow::ensure!(
        url.scheme() == "https"
            || env::var("RTD_ARCHIVE_ALLOW_INSECURE_S3_DEV").as_deref() == Ok("1"),
        "Plaintext S3 is restricted to explicit local development"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::ObjectStoreConfig;
    use super::ObjectStoreType;
    use super::validate_self_hosted_endpoint;

    #[test]
    fn object_store_rejects_cloud_backends_and_implicit_s3() {
        for backend in [ObjectStoreType::GCS, ObjectStoreType::Azure] {
            let config = ObjectStoreConfig {
                object_store: Some(backend),
                ..Default::default()
            };
            assert!(config.make().is_err());
        }
        let config = ObjectStoreConfig {
            object_store: Some(ObjectStoreType::S3),
            ..Default::default()
        };
        assert!(
            config
                .make()
                .err()
                .unwrap()
                .to_string()
                .contains("endpoint")
        );
    }

    #[test]
    fn object_store_requires_private_endpoint() {
        assert!(validate_self_hosted_endpoint("https://ceph.rtd.internal").is_ok());
        for endpoint in [
            "https://s3.us-east-1.amazonaws.com",
            "https://storage.googleapis.com",
            "https://account.blob.core.windows.net",
            "https://storage.googleapis.com.",
            "file:///tmp/archive",
        ] {
            assert!(
                validate_self_hosted_endpoint(endpoint).is_err(),
                "{endpoint}"
            );
        }
    }
}
