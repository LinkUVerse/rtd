// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::error::Error as StdError;
use std::sync::Arc;

use anyhow::anyhow;

/// Error type for DataLoader implementations that wraps Arc<anyhow::Error>
/// for efficient cloning while preserving ergonomic error handling.
#[derive(Debug, Clone, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] Arc<anyhow::Error>);

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error(Arc::new(error))
    }
}

impl From<tonic::Status> for Error {
    fn from(error: tonic::Status) -> Self {
        Error(Arc::new(anyhow!(error).context("gRPC error")))
    }
}

/// SDK channels can wrap a broken connection in Unknown instead of Unavailable.
/// Only a typed local transport cause can reclassify an otherwise generic status.
pub fn is_grpc_dependency_unavailable(status: &tonic::Status) -> bool {
    match status.code() {
        tonic::Code::Cancelled | tonic::Code::Unavailable | tonic::Code::DeadlineExceeded => true,
        tonic::Code::Unknown | tonic::Code::Internal => {
            let mut cause = status.source();
            while let Some(error) = cause {
                if error.is::<tonic::transport::Error>() {
                    return true;
                }
                if let Some(error) = error.downcast_ref::<std::io::Error>() {
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::BrokenPipe
                            | std::io::ErrorKind::ConnectionAborted
                            | std::io::ErrorKind::ConnectionRefused
                            | std::io::ErrorKind::ConnectionReset
                            | std::io::ErrorKind::NotConnected
                            | std::io::ErrorKind::TimedOut
                            | std::io::ErrorKind::UnexpectedEof
                    ) {
                        return true;
                    }
                }
                cause = error.source();
            }
            false
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::Arc;

    use tonic::Code;
    use tonic::Status;

    use super::is_grpc_dependency_unavailable;

    #[tokio::test]
    async fn grpc_transport_classification_requires_typed_causes_for_generic_statuses() {
        for code in [Code::Cancelled, Code::Unavailable, Code::DeadlineExceeded] {
            assert!(is_grpc_dependency_unavailable(&Status::new(
                code,
                "private cause"
            )));
        }
        for code in [Code::Unknown, Code::Internal] {
            let mut status = Status::new(code, "transport error");
            assert!(!is_grpc_dependency_unavailable(&status));
            status.set_source(Arc::new(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "private pipe",
            )));
            assert!(is_grpc_dependency_unavailable(&status));
            status.set_source(Arc::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "bad business response",
            )));
            assert!(!is_grpc_dependency_unavailable(&status));
        }
        for code in [
            Code::InvalidArgument,
            Code::PermissionDenied,
            Code::OutOfRange,
        ] {
            let mut status = Status::new(code, "transport error");
            status.set_source(Arc::new(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "private pipe",
            )));
            assert!(!is_grpc_dependency_unavailable(&status));
        }
    }

    #[tokio::test]
    async fn grpc_unknown_wrapping_actual_channel_failure_is_unavailable() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let error = tonic::transport::Endpoint::from_shared(format!("http://{address}"))
            .unwrap()
            .connect()
            .await
            .unwrap_err();
        let mut status = Status::unknown("transport error");
        status.set_source(Arc::new(error));
        assert!(is_grpc_dependency_unavailable(&status));
    }
}
