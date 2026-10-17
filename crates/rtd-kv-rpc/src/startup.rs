// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Wait for the writer's first checkpoint without opening Ledger listeners early.
use std::future::Future;
use std::time::Duration;

use anyhow::{Context, Result, ensure};

pub(crate) const DEFAULT_GENESIS_WAIT_MS: u64 = 30_000;
pub(crate) const MAX_GENESIS_WAIT_MS: u64 = 300_000;

pub(crate) fn validate_budget(budget: Duration) -> Result<()> {
    ensure!(
        budget >= Duration::from_millis(1) && budget <= Duration::from_millis(MAX_GENESIS_WAIT_MS),
        "genesis wait must be within 1..=300000 milliseconds"
    );
    Ok(())
}

pub(crate) async fn wait_for_genesis<T, F, Fut>(budget: Duration, mut fetch: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<Option<T>>>,
{
    validate_budget(budget)?;
    tokio::time::timeout(budget, async {
        loop {
            // Authentication, transport and decode errors fail immediately. Only a
            // successful read with an absent checkpoint can mean writer warm-up.
            if let Some(genesis) = fetch().await? {
                return Ok(genesis);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .context("genesis checkpoint was not available within the configured startup budget")?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn invalid_budget_is_rejected_before_fetch() {
        let mut calls = 0;
        for budget in [
            Duration::ZERO,
            Duration::from_nanos(1),
            Duration::from_millis(300_001),
        ] {
            assert!(
                wait_for_genesis::<u8, _, _>(budget, || {
                    calls += 1;
                    async { Ok(None) }
                })
                .await
                .is_err()
            );
        }
        assert_eq!(calls, 0);
    }

    #[tokio::test(start_paused = true)]
    async fn existing_genesis_does_not_wait() {
        let began = tokio::time::Instant::now();
        assert_eq!(
            wait_for_genesis(Duration::from_secs(1), || async { Ok(Some(7)) })
                .await
                .unwrap(),
            7
        );
        assert_eq!(began.elapsed(), Duration::ZERO);
    }

    #[tokio::test(start_paused = true)]
    async fn missing_genesis_is_retried_until_writer_supplies_it() {
        let mut calls = 0;
        let began = tokio::time::Instant::now();
        let result = wait_for_genesis(Duration::from_secs(1), || {
            calls += 1;
            let value = (calls == 3).then_some(7);
            async move { Ok(value) }
        })
        .await
        .unwrap();
        assert_eq!(result, 7);
        assert_eq!(calls, 3);
        assert_eq!(began.elapsed(), Duration::from_millis(200));
    }

    #[tokio::test(start_paused = true)]
    async fn backend_error_is_not_retried_or_hidden() {
        let mut calls = 0;
        let result = wait_for_genesis::<u8, _, _>(Duration::from_secs(1), || {
            calls += 1;
            async { anyhow::bail!("permission denied") }
        })
        .await;
        assert_eq!(calls, 1);
        assert_eq!(result.unwrap_err().to_string(), "permission denied");
    }

    #[tokio::test(start_paused = true)]
    async fn absent_genesis_has_one_total_deadline() {
        let mut calls = 0;
        let began = tokio::time::Instant::now();
        let result = wait_for_genesis::<u8, _, _>(Duration::from_millis(250), || {
            calls += 1;
            async { Ok(None) }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(calls, 3);
        assert_eq!(began.elapsed(), Duration::from_millis(250));
    }

    #[tokio::test(start_paused = true)]
    async fn stalled_fetch_is_cancelled_at_the_total_deadline() {
        let began = tokio::time::Instant::now();
        let result =
            wait_for_genesis::<u8, _, _>(Duration::from_secs(1), std::future::pending).await;
        assert!(result.is_err());
        assert_eq!(began.elapsed(), Duration::from_secs(1));
    }
}
