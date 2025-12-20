// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use clap::*;
use ethers::types::Address as EthAddress;
use prometheus::Registry;
use std::collections::HashSet;
use std::env;
use std::net::IpAddr;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use rtd_bridge::eth_client::EthClient;
use rtd_bridge::metered_eth_provider::{MeteredEthHttpProvider, new_metered_eth_provider};
use rtd_bridge::rtd_bridge_watchdog::Observable;
use rtd_bridge::rtd_client::RtdBridgeClient;
use rtd_bridge::utils::get_eth_contract_addresses;
use rtd_config::Config;
use tokio::task::JoinHandle;
use tracing::info;

use linku_metrics::metered_channel::channel;
use linku_metrics::spawn_logged_monitored_task;
use linku_metrics::start_prometheus_server;

use rtd_bridge::metrics::BridgeMetrics;
use rtd_bridge::rtd_bridge_watchdog::{
    BridgeWatchDog,
    eth_bridge_status::EthBridgeStatus,
    eth_vault_balance::{EthereumVaultBalance, VaultAsset},
    metrics::WatchdogMetrics,
    rtd_bridge_status::RtdBridgeStatus,
};
use rtd_bridge_indexer::config::IndexerConfig;
use rtd_bridge_indexer::metrics::BridgeIndexerMetrics;
use rtd_bridge_indexer::postgres_manager::{get_connection_pool, read_rtd_progress_store};
use rtd_bridge_indexer::rtd_transaction_handler::handle_rtd_transactions_loop;
use rtd_bridge_indexer::rtd_transaction_queries::start_rtd_tx_polling_task;
use rtd_bridge_indexer::{
    create_eth_subscription_indexer, create_eth_sync_indexer, create_rtd_indexer,
};
use rtd_data_ingestion_core::DataIngestionMetrics;
use rtd_sdk::RtdClientBuilder;

#[derive(Parser, Clone, Debug)]
struct Args {
    /// Path to a yaml config
    #[clap(long, short)]
    config_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _guard = telemetry_subscribers::TelemetryConfig::new()
        .with_env()
        .init();

    let args = Args::parse();

    // load config
    let config_path = if let Some(path) = args.config_path {
        path
    } else {
        env::current_dir()
            .expect("Couldn't get current directory")
            .join("config.yaml")
    };
    let config = IndexerConfig::load(&config_path)?;

    // Init metrics server
    let metrics_address =
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), config.metric_port);
    let registry_service = start_prometheus_server(metrics_address);
    let registry = registry_service.default_registry();
    linku_metrics::init_metrics(&registry);
    info!("Metrics server started at port {}", config.metric_port);

    let indexer_meterics = BridgeIndexerMetrics::new(&registry);
    let ingestion_metrics = DataIngestionMetrics::new(&registry);
    let bridge_metrics = Arc::new(BridgeMetrics::new(&registry));

    let db_url = config.db_url.clone();
    let pool = get_connection_pool(db_url.clone()).await;

    let eth_client: Arc<EthClient<MeteredEthHttpProvider>> = Arc::new(
        EthClient::<MeteredEthHttpProvider>::new(
            &config.eth_rpc_url,
            HashSet::from_iter(vec![]), // dummy
            bridge_metrics.clone(),
        )
        .await?,
    );
    let eth_bridge_proxy_address = EthAddress::from_str(&config.eth_rtd_bridge_contract_address)?;
    let mut tasks = vec![];
    // Start the eth subscription indexer
    let eth_subscription_indexer = create_eth_subscription_indexer(
        pool.clone(),
        indexer_meterics.clone(),
        &config,
        eth_client.clone(),
    )
    .await?;
    tasks.push(spawn_logged_monitored_task!(
        eth_subscription_indexer.start()
    ));

    // Start the eth sync data source
    let eth_sync_indexer = create_eth_sync_indexer(
        pool.clone(),
        indexer_meterics.clone(),
        bridge_metrics.clone(),
        &config,
        eth_client,
    )
    .await?;
    tasks.push(spawn_logged_monitored_task!(eth_sync_indexer.start()));

    if !config.eth_only {
        let indexer =
            create_rtd_indexer(pool, indexer_meterics, ingestion_metrics, &config).await?;
        tasks.push(spawn_logged_monitored_task!(indexer.start()));
    }

    let rtd_bridge_client =
        Arc::new(RtdBridgeClient::new(&config.rtd_rpc_url, bridge_metrics.clone()).await?);
    start_watchdog(
        config,
        eth_bridge_proxy_address,
        rtd_bridge_client,
        &registry,
        bridge_metrics.clone(),
    )
    .await?;

    // Wait for tasks in `tasks` to finish. Return when anyone of them returns an error.
    futures::future::try_join_all(tasks).await?;
    unreachable!("Indexer tasks finished unexpectedly");
}

async fn start_watchdog(
    config: IndexerConfig,
    eth_bridge_proxy_address: EthAddress,
    rtd_client: Arc<RtdBridgeClient>,
    registry: &Registry,
    bridge_metrics: Arc<BridgeMetrics>,
) -> Result<()> {
    let watchdog_metrics = WatchdogMetrics::new(registry);
    let eth_provider =
        Arc::new(new_metered_eth_provider(&config.eth_rpc_url, bridge_metrics.clone()).unwrap());
    let (
        _committee_address,
        _limiter_address,
        vault_address,
        _config_address,
        weth_address,
        usdt_address,
        wbtc_address,
        lbtc_address,
    ) = get_eth_contract_addresses(eth_bridge_proxy_address, &eth_provider).await?;

    let eth_vault_balance = EthereumVaultBalance::new(
        eth_provider.clone(),
        vault_address,
        weth_address,
        VaultAsset::WETH,
        watchdog_metrics.eth_vault_balance.clone(),
    )
    .await
    .unwrap_or_else(|e| panic!("Failed to create eth vault balance: {}", e));

    let usdt_vault_balance = EthereumVaultBalance::new(
        eth_provider.clone(),
        vault_address,
        usdt_address,
        VaultAsset::USDT,
        watchdog_metrics.usdt_vault_balance.clone(),
    )
    .await
    .unwrap_or_else(|e| panic!("Failed to create usdt vault balance: {}", e));

    let wbtc_vault_balance = EthereumVaultBalance::new(
        eth_provider.clone(),
        vault_address,
        wbtc_address,
        VaultAsset::WBTC,
        watchdog_metrics.wbtc_vault_balance.clone(),
    )
    .await
    .unwrap_or_else(|e| panic!("Failed to create wbtc vault balance: {}", e));

    let lbtc_vault_balance = if !lbtc_address.is_zero() {
        Some(
            EthereumVaultBalance::new(
                eth_provider.clone(),
                vault_address,
                lbtc_address,
                VaultAsset::LBTC,
                watchdog_metrics.lbtc_vault_balance.clone(),
            )
            .await
            .unwrap_or_else(|e| panic!("Failed to create lbtc vault balance: {}", e)),
        )
    } else {
        None
    };

    let eth_bridge_status = EthBridgeStatus::new(
        eth_provider,
        eth_bridge_proxy_address,
        watchdog_metrics.eth_bridge_paused.clone(),
    );

    let rtd_bridge_status =
        RtdBridgeStatus::new(rtd_client, watchdog_metrics.rtd_bridge_paused.clone());
    let mut observables: Vec<Box<dyn Observable + Send + Sync>> = vec![
        Box::new(eth_vault_balance),
        Box::new(usdt_vault_balance),
        Box::new(wbtc_vault_balance),
        Box::new(eth_bridge_status),
        Box::new(rtd_bridge_status),
    ];

    // Add lbtc_vault_balance if it's available
    if let Some(balance) = lbtc_vault_balance {
        observables.push(Box::new(balance));
    }

    BridgeWatchDog::new(observables).run().await;

    Ok(())
}

#[allow(unused)]
async fn start_processing_rtd_checkpoints_by_querying_txns(
    rtd_rpc_url: String,
    db_url: String,
    indexer_metrics: BridgeIndexerMetrics,
) -> Result<Vec<JoinHandle<()>>> {
    let pg_pool = get_connection_pool(db_url.clone()).await;
    let (tx, rx) = channel(
        100,
        &linku_metrics::get_metrics()
            .unwrap()
            .channel_inflight
            .with_label_values(&["rtd_transaction_processing_queue"]),
    );
    let mut handles = vec![];
    let cursor = read_rtd_progress_store(&pg_pool)
        .await
        .expect("Failed to read cursor from rtd progress store");
    let rtd_client = RtdClientBuilder::default().build(rtd_rpc_url).await?;
    handles.push(spawn_logged_monitored_task!(
        start_rtd_tx_polling_task(rtd_client, cursor, tx),
        "start_rtd_tx_polling_task"
    ));
    handles.push(spawn_logged_monitored_task!(
        handle_rtd_transactions_loop(pg_pool.clone(), rx, indexer_metrics.clone()),
        "handle_rtd_transcations_loop"
    ));
    Ok(handles)
}
