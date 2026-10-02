// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::env;

use anyhow::Context as _;
use object_store::aws::AmazonS3Builder;
use object_store::path::Path;
use object_store::ClientOptions;
use object_store::ObjectStoreExt as _;
use rtd_indexer_alt_framework::ingestion::s3_tls::with_archive_s3_ca;
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let endpoint = env::var("AWS_ENDPOINT").context("AWS_ENDPOINT is required")?;
    let endpoint = Url::parse(&endpoint)?;
    let store = AmazonS3Builder::from_env()
        .with_client_options(with_archive_s3_ca(ClientOptions::default(), &endpoint)?)
        .with_bucket_name("rtd-checkpoints-published-event")
        .build()?;
    let result = store.get(&Path::from("0.binpb.zst")).await?;
    let metadata = result.meta.clone();
    let content = result.bytes().await?;
    if content.is_empty() {
        anyhow::bail!("published checkpoint 0 is empty");
    }
    println!(
        "archive_s3_tls_read=pass bytes={} etag={} version={}",
        content.len(),
        metadata.e_tag.as_deref().unwrap_or("missing"),
        metadata.version.as_deref().unwrap_or("missing")
    );
    Ok(())
}
