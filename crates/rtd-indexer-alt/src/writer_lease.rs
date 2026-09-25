// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Optional process-level admission lock for the Alt PostgreSQL writer.
//! It is held on one dedicated database session until every indexer task stops.
//! The database also checks the generation of every Alt data write.

use std::future::pending;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use anyhow::ensure;
use diesel::QueryableByName;
use diesel::sql_types::Bool;
use diesel::sql_types::Integer;
use diesel_async::RunQueryDsl;
use rtd_indexer_alt::writer_fence;
use rtd_indexer_alt_framework::postgres::Db;
use rtd_indexer_alt_framework::postgres::DbArgs;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;
use tokio::time::interval;
use tracing::info;
use url::Url;

// PostgreSQL advisory locks are scoped to one database. The fixed pair reserves
// one main Alt writer per database, independently of the network using that DB.
const LOCK_SQL: &str =
    "SELECT pg_try_advisory_lock(1381254145, 1) AS acquired, pg_backend_pid() AS pid";
const PING_SQL: &str = "SELECT pg_backend_pid() AS pid";

#[derive(QueryableByName)]
struct Admission {
    #[diesel(sql_type = Bool)]
    acquired: bool,
    #[diesel(sql_type = Integer)]
    pid: i32,
}

#[derive(QueryableByName)]
struct Session {
    #[diesel(sql_type = Integer)]
    pid: i32,
}

pub struct WriterLease {
    task: JoinHandle<Result<()>>,
    epoch: i64,
}

impl Drop for WriterLease {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl WriterLease {
    pub fn epoch(&self) -> i64 {
        self.epoch
    }

    async fn failure(&mut self) -> anyhow::Error {
        match (&mut self.task).await {
            Ok(Err(error)) => error,
            Ok(Ok(())) => anyhow::anyhow!("Exclusive writer lock monitor exited unexpectedly"),
            Err(error) => error.into(),
        }
    }
}

pub async fn fail_if_ended(lease: Option<&mut WriterLease>) -> anyhow::Error {
    match lease {
        Some(lease) => lease.failure().await,
        None => pending().await,
    }
}

pub async fn acquire(database_url: Url, mut db_args: DbArgs) -> Result<WriterLease> {
    db_args.db_connection_pool_size = 1;
    let (sender, receiver) = oneshot::channel::<Result<(i32, i64), String>>();
    let task = tokio::spawn(async move {
        let db = Db::for_write(database_url, db_args)
            .await
            .context("Failed to connect for Alt writer lock")?;
        let mut conn = db
            .connect()
            .await
            .context("Failed to obtain Alt writer lock session")?;
        let admission: Admission = diesel::sql_query(LOCK_SQL)
            .get_result(&mut conn)
            .await
            .context("Failed to request Alt writer lock")?;
        if !admission.acquired {
            let message = "Another Alt writer holds this PostgreSQL database lock";
            let _ = sender.send(Err(message.to_owned()));
            bail!("{message}");
        }
        let epoch = writer_fence::advance_epoch(&mut conn).await?;
        let _ = sender.send(Ok((admission.pid, epoch)));

        let mut tick = interval(Duration::from_secs(1));
        tick.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            tick.tick().await;
            let current: Session = diesel::sql_query(PING_SQL)
                .get_result(&mut conn)
                .await
                .context("Alt writer lock session lost")?;
            ensure!(
                current.pid == admission.pid,
                "Alt writer lock session changed unexpectedly"
            );
        }
    });

    let (pid, epoch) = match receiver.await {
        Ok(Ok(admission)) => admission,
        Ok(Err(message)) => {
            task.abort();
            bail!("{message}");
        }
        Err(_) => {
            return match task.await {
                Ok(Err(error)) => Err(error),
                Ok(Ok(())) => bail!("Alt writer lock monitor exited before admission"),
                Err(error) => Err(error.into()),
            };
        }
    };
    info!(pid, epoch, "Exclusive Alt writer lock acquired");
    Ok(WriterLease { task, epoch })
}
