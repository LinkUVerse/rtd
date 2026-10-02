// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use clap::Parser;
use prometheus::Registry;
use rtd_indexer_alt::args::Args;
use rtd_indexer_alt::args::Command;
use rtd_indexer_alt::config::IndexerConfig;
use rtd_indexer_alt::config::Merge;
use rtd_indexer_alt::setup_indexer;
use rtd_indexer_alt_framework::postgres::reset_database;
use rtd_indexer_alt_framework::service::Error;
use rtd_indexer_alt_framework::service::terminate;
use rtd_indexer_alt_metrics::MetricsService;
use rtd_indexer_alt_metrics::uptime;
use rtd_indexer_alt_schema::MIGRATIONS;
use tokio::fs;
use tracing::info;

mod writer_lease;

// Define the `GIT_REVISION` const
bin_version::git_revision!();

static VERSION: &str = const_str::concat!(
    env!("CARGO_PKG_VERSION_MAJOR"),
    ".",
    env!("CARGO_PKG_VERSION_MINOR"),
    ".",
    env!("CARGO_PKG_VERSION_PATCH"),
    "-",
    GIT_REVISION
);

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Enable tracing, configured by environment variables.
    let _guard = telemetry_subscribers::TelemetryConfig::new()
        .with_env()
        .init();

    match args.command {
        Command::Indexer {
            database_url,
            db_args,
            client_args,
            indexer_args,
            metrics_args,
            config,
            require_exclusive_writer,
        } => {
            let is_bounded = indexer_args.last_checkpoint.is_some();

            let indexer_config = read_config(&config).await?;
            info!("Starting indexer with config: {:#?}", indexer_config);

            let registry = Registry::new_custom(Some("indexer_alt".into()), None)
                .context("Failed to create Prometheus registry.")?;

            let metrics = MetricsService::new(metrics_args, registry);

            metrics
                .registry()
                .register(uptime(VERSION)?)
                .context("Failed to register uptime metric.")?;

            // Hold the PostgreSQL session lock throughout setup, ingestion and graceful
            // shutdown. A competing process must not run migrations or ingest first.
            let mut exclusive_writer = if require_exclusive_writer {
                Some(writer_lease::acquire(database_url.clone(), db_args.clone()).await?)
            } else {
                None
            };
            let mut db_args = db_args;
            db_args.writer_epoch = exclusive_writer
                .as_ref()
                .map(writer_lease::WriterLease::epoch);

            let indexer = tokio::select! {
                _ = terminate() => {
                    info!("Indexer terminated during setup");
                    return Ok(());
                }

                indexer = setup_indexer(
                    database_url,
                    db_args,
                    indexer_args,
                    client_args,
                    indexer_config,
                    None,
                    metrics.registry(),
                ) => {
                    indexer?
                }

                error = writer_lease::fail_if_ended(exclusive_writer.as_mut()) => {
                    return Err(error).context("Exclusive Alt writer lock failed during setup");
                }
            };

            let (s_indexer, s_metrics) = tokio::select! {
                result = async {
                    let s_indexer = indexer.run().await?;
                    let s_metrics = metrics.run().await?;
                    Ok::<_, anyhow::Error>((s_indexer, s_metrics))
                } => result?,
                error = writer_lease::fail_if_ended(exclusive_writer.as_mut()) => {
                    return Err(error).context("Exclusive Alt writer lock failed during startup");
                }
            };

            let service_result = tokio::select! {
                result = s_indexer.attach(s_metrics).main() => result,
                error = writer_lease::fail_if_ended(exclusive_writer.as_mut()) => {
                    tracing::error!("Exclusive Alt writer lock lost: {error:#}");
                    std::process::exit(2);
                }
            };
            // The service has stopped all writer tasks before releasing its lock.
            drop(exclusive_writer);

            match service_result {
                Ok(()) => {}
                Err(Error::Terminated) => {
                    if is_bounded {
                        std::process::exit(1);
                    }
                }
                Err(Error::Aborted) => {
                    std::process::exit(1);
                }
                Err(Error::Task(_)) => {
                    std::process::exit(2);
                }
            }
        }

        Command::GenerateConfig => {
            let config = IndexerConfig::example();
            let config_toml = toml::to_string_pretty(&config)
                .context("Failed to serialize default configuration to TOML.")?;

            println!("{config_toml}");
        }

        Command::MergeConfigs { config } => {
            let mut files = config.into_iter();

            let Some(file) = files.next() else {
                bail!("At least one configuration file must be provided.");
            };

            let mut indexer_config = read_config(&file).await?;
            for file in files {
                indexer_config =
                    indexer_config.merge(read_config(&file).await.with_context(|| {
                        format!("Failed to read configuration file: {}", file.display())
                    })?)?;
            }

            let config_toml = toml::to_string_pretty(&indexer_config)
                .context("Failed to serialize merged configuration to TOML.")?;

            println!("{config_toml}");
        }

        Command::ResetDatabase {
            database_url,
            db_args,
            skip_migrations,
        } => {
            reset_database(
                database_url,
                db_args,
                if !skip_migrations {
                    Some(&MIGRATIONS)
                } else {
                    None
                },
            )
            .await?;
        }

        #[cfg(feature = "benchmark")]
        Command::Benchmark {
            database_url,
            db_args,
            benchmark_args,
            config,
        } => {
            let indexer_config = read_config(&config).await?;
            rtd_indexer_alt::benchmark::run_benchmark(
                database_url,
                db_args,
                benchmark_args,
                indexer_config,
            )
            .await?;
        }
    }

    Ok(())
}

async fn read_config(path: &Path) -> Result<IndexerConfig> {
    let config_contents = fs::read_to_string(path)
        .await
        .context("Failed to read configuration TOML file")?;

    toml::from_str(&config_contents).context("Failed to parse configuration TOML file.")
}
