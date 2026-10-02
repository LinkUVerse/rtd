// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use rtd_kvstore::BigTableClient;
use rtd_kvstore::PoolConfig;

/// Run with RTD_HBASE_GATEWAY_ENDPOINT=http://127.0.0.1:38080 and
/// RTD_HBASE_GATEWAY_ALLOW_INSECURE_DEV=1 against the local Compose stack.
#[tokio::test]
async fn self_hosted_gateway_primes_and_reads_hbase() -> anyhow::Result<()> {
    if std::env::var_os("RTD_HBASE_GATEWAY_ENDPOINT").is_none() {
        return Ok(());
    }

    let mut client = BigTableClient::new_remote(
        "archive-dev".to_string(),
        Some("rtd".to_string()),
        true,
        None,
        None,
        "hbase-gateway-smoke".to_string(),
        None,
        None,
        PoolConfig::singleton(),
        false,
    )
    .await?;

    // Both queries must pass through the real gateway and two separate HBase tables.
    let _ = client.get_checkpoints_filtered(&[0], None).await?;
    let _ = client
        .get_pipeline_watermark_rows("__rtd_read_only_smoke__")
        .await?;
    Ok(())
}
