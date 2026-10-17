// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use rtd_types::error::ErrorCategory;
use tonic::Code;

use crate::proto::google::rpc::{BadRequest, ErrorInfo, RetryInfo};
pub use rtd_rpc::proto::rtd::rpc::v2::ErrorReason;

pub type Result<T, E = RpcError> = std::result::Result<T, E>;

/// An error encountered while serving an RPC request.
///
/// General error type used by top-level RPC service methods. The main purpose of this error type
/// is to provide a convenient type for converting between internal errors and a response that
/// needs to be sent to a calling client.
#[derive(Debug)]
pub struct RpcError {
    code: Code,
    message: Option<String>,
    details: Option<Box<ErrorDetails>>,
}

impl RpcError {
    pub fn new<T: Into<String>>(code: Code, message: T) -> Self {
        Self {
            code,
            message: Some(message.into()),
            details: None,
        }
    }

    pub fn not_found() -> Self {
        Self {
            code: Code::NotFound,
            message: None,
            details: None,
        }
    }

    pub fn into_status_proto(self) -> crate::proto::google::rpc::Status {
        crate::proto::google::rpc::Status {
            code: self.code.into(),
            message: self.message.unwrap_or_default(),
            details: self
                .details
                .map(ErrorDetails::into_status_details)
                .unwrap_or_default(),
        }
    }
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.message {
            Some(m) => write!(f, "{}: {}", self.code, m),
            None => write!(f, "{}", self.code),
        }
    }
}

impl std::error::Error for RpcError {}

impl From<RpcError> for tonic::Status {
    fn from(value: RpcError) -> Self {
        use prost::Message;

        let code = value.code;
        let status = value.into_status_proto();
        let details = status.encode_to_vec().into();
        let message = status.message;

        tonic::Status::with_details(code, message, details)
    }
}

impl From<rtd_types::storage::error::Error> for RpcError {
    fn from(value: rtd_types::storage::error::Error) -> Self {
        use rtd_types::storage::error::Kind;

        let code = match value.kind() {
            Kind::Missing => Code::NotFound,
            _ => Code::Internal,
        };

        Self {
            code,
            message: Some(value.to_string()),
            details: None,
        }
    }
}

impl From<anyhow::Error> for RpcError {
    fn from(value: anyhow::Error) -> Self {
        // Pipeline stages may wrap an already classified RPC error in anyhow.
        // Recover the owned error (including structured details) through context.
        let value = match value.downcast::<Self>() {
            Ok(error) => return error,
            Err(value) => value,
        };
        // Keep transient dependency failures retryable even through anyhow context.
        // Do not expose upstream transport details or reinterpret data errors as absence.
        if let Some(status) = value
            .chain()
            .find_map(|cause| cause.downcast_ref::<tonic::Status>())
        {
            // HTTP/2 may report a broken body as Unknown while retaining an
            // I/O source. Only typed connection failures become retryable.
            let transport_broken = value.chain().any(|cause| {
                cause
                    .downcast_ref::<std::io::Error>()
                    .or_else(|| {
                        cause
                            .downcast_ref::<h2::Error>()
                            .and_then(h2::Error::get_io)
                    })
                    .is_some_and(|error| {
                        matches!(
                            error.kind(),
                            std::io::ErrorKind::UnexpectedEof
                                | std::io::ErrorKind::ConnectionReset
                                | std::io::ErrorKind::ConnectionAborted
                                | std::io::ErrorKind::BrokenPipe
                                | std::io::ErrorKind::NotConnected
                        )
                    })
            });
            if status.code() == Code::Unknown && transport_broken {
                return Self::new(Code::Unavailable, "Dependency connection interrupted");
            }
            let message = match status.code() {
                Code::Unavailable => Some("Dependency unavailable"),
                Code::DeadlineExceeded => Some("Dependency request timed out"),
                Code::ResourceExhausted => Some("Dependency capacity exhausted"),
                _ => None,
            };
            if let Some(message) = message {
                return Self::new(status.code(), message);
            }
        }
        Self {
            code: Code::Internal,
            message: Some(value.to_string()),
            details: None,
        }
    }
}

impl From<rtd_types::rtd_sdk_types_conversions::SdkTypeConversionError> for RpcError {
    fn from(value: rtd_types::rtd_sdk_types_conversions::SdkTypeConversionError) -> Self {
        Self {
            code: Code::Internal,
            message: Some(value.to_string()),
            details: None,
        }
    }
}

impl From<bcs::Error> for RpcError {
    fn from(value: bcs::Error) -> Self {
        Self {
            code: Code::Internal,
            message: Some(value.to_string()),
            details: None,
        }
    }
}

impl From<rtd_types::transaction_driver_types::TransactionSubmissionError> for RpcError {
    fn from(error: rtd_types::transaction_driver_types::TransactionSubmissionError) -> Self {
        use itertools::Itertools;
        use rtd_types::error::RtdErrorKind;
        use rtd_types::transaction_driver_types::TransactionSubmissionError::*;

        match error {
            InvalidUserSignature(err) => {
                let message = {
                    let err = match err.as_inner() {
                        RtdErrorKind::UserInputError { error } => error.to_string(),
                        _ => err.to_string(),
                    };
                    format!("Invalid user signature: {err}")
                };

                RpcError::new(Code::InvalidArgument, message)
            }
            TransactionDriverInternalError(err) => RpcError::new(Code::Internal, err.to_string()),
            ObjectsDoubleUsed { conflicting_txes } => {
                let new_map = conflicting_txes
                    .into_iter()
                    .map(|(digest, (pairs, _))| {
                        (
                            digest,
                            pairs.into_iter().map(|(_, obj_ref)| obj_ref).collect(),
                        )
                    })
                    .collect::<std::collections::BTreeMap<_, Vec<_>>>();

                let message = format!(
                    "Failed to sign transaction by a quorum of validators because of locked objects. Conflicting Transactions:\n{new_map:#?}",
                );

                RpcError::new(Code::FailedPrecondition, message)
            }
            TimeoutBeforeFinality | FailedWithTransientErrorAfterMaximumAttempts { .. } => {
                // TODO add a Retry-After header
                RpcError::new(
                    Code::Unavailable,
                    "timed-out before finality could be reached",
                )
            }
            TimeoutBeforeFinalityWithErrors {
                last_error,
                attempts,
                timeout,
            } => {
                // TODO add a Retry-After header
                RpcError::new(
                    Code::Unavailable,
                    format!(
                        "Transaction timed out before finality could be reached. Attempts: {attempts} & timeout: {timeout:?}. Last error: {last_error}"
                    ),
                )
            }
            NonRecoverableTransactionError { errors } => {
                let new_errors: Vec<String> = errors
                    .into_iter()
                    // sort by total stake, descending, so users see the most prominent one first
                    .sorted_by(|(_, a, _), (_, b, _)| b.cmp(a))
                    .filter_map(|(err, _, _)| {
                        match err.as_inner() {
                            // Special handling of UserInputError:
                            // ObjectNotFound and DependentPackageNotFound are considered
                            // retryable errors but they have different treatment
                            // in AuthorityAggregator.
                            // The optimal fix would be to examine if the total stake
                            // of ObjectNotFound/DependentPackageNotFound exceeds the
                            // quorum threshold, but it takes a Committee here.
                            // So, we take an easier route and consider them non-retryable
                            // at all. Combining this with the sorting above, clients will
                            // see the dominant error first.
                            RtdErrorKind::UserInputError { error } => Some(error.to_string()),
                            _ => {
                                if err.is_retryable().0 {
                                    None
                                } else {
                                    Some(err.to_string())
                                }
                            }
                        }
                    })
                    .collect();

                assert!(
                    !new_errors.is_empty(),
                    "NonRecoverableTransactionError should have at least one non-retryable error"
                );

                let error_list = new_errors.join(", ");
                let error_msg = format!(
                    "Transaction execution failed due to issues with transaction inputs, please review the errors and try again: {}.",
                    error_list
                );

                RpcError::new(Code::InvalidArgument, error_msg)
            }
            TxAlreadyFinalizedWithDifferentUserSignatures => RpcError::new(
                Code::Aborted,
                "The transaction is already finalized but with different user signatures",
            ),
            SystemOverload { .. } | SystemOverloadRetryAfter { .. } => {
                // TODO add a Retry-After header
                RpcError::new(Code::Unavailable, "system is overloaded")
            }
            TransactionFailed { category, details } => RpcError::new(
                // TODO(fastpath): add a Retry-After header.
                match category {
                    ErrorCategory::Internal => Code::Internal,
                    ErrorCategory::Aborted => Code::Aborted,
                    ErrorCategory::InvalidTransaction => Code::InvalidArgument,
                    ErrorCategory::LockConflict => Code::FailedPrecondition,
                    ErrorCategory::ValidatorOverloaded => Code::ResourceExhausted,
                    ErrorCategory::Unavailable => Code::Unavailable,
                },
                details,
            ),
        }
    }
}

impl From<crate::proto::google::rpc::bad_request::FieldViolation> for RpcError {
    fn from(value: crate::proto::google::rpc::bad_request::FieldViolation) -> Self {
        BadRequest::from(value).into()
    }
}

impl From<BadRequest> for RpcError {
    fn from(value: BadRequest) -> Self {
        let message = value
            .field_violations
            .first()
            .map(|violation| violation.description.clone());
        let details = ErrorDetails::new().with_bad_request(value);

        RpcError {
            code: Code::InvalidArgument,
            message,
            details: Some(Box::new(details)),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ErrorDetails {
    error_info: Option<ErrorInfo>,
    bad_request: Option<BadRequest>,
    retry_info: Option<RetryInfo>,
}

impl ErrorDetails {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn error_info(&self) -> Option<&ErrorInfo> {
        self.error_info.as_ref()
    }

    pub fn bad_request(&self) -> Option<&BadRequest> {
        self.bad_request.as_ref()
    }

    pub fn retry_info(&self) -> Option<&RetryInfo> {
        self.retry_info.as_ref()
    }

    pub fn details(&self) -> &[prost_types::Any] {
        &[]
    }

    pub fn with_bad_request(mut self, bad_request: BadRequest) -> Self {
        self.bad_request = Some(bad_request);
        self
    }

    #[allow(clippy::boxed_local)]
    fn into_status_details(self: Box<Self>) -> Vec<prost_types::Any> {
        let mut details = Vec::new();

        if let Some(error_info) = &self.error_info {
            details.push(
                prost_types::Any::from_msg(error_info).expect("Message encoding cannot fail"),
            );
        }

        if let Some(bad_request) = &self.bad_request {
            details.push(
                prost_types::Any::from_msg(bad_request).expect("Message encoding cannot fail"),
            );
        }

        if let Some(retry_info) = &self.retry_info {
            details.push(
                prost_types::Any::from_msg(retry_info).expect("Message encoding cannot fail"),
            );
        }
        details
    }
}

#[derive(Debug)]
pub struct ObjectNotFoundError {
    object_id: rtd_sdk_types::Address,
    version: Option<rtd_sdk_types::Version>,
}

impl ObjectNotFoundError {
    pub fn new(object_id: rtd_sdk_types::Address) -> Self {
        Self {
            object_id,
            version: None,
        }
    }

    pub fn new_with_version(
        object_id: rtd_sdk_types::Address,
        version: rtd_sdk_types::Version,
    ) -> Self {
        Self {
            object_id,
            version: Some(version),
        }
    }
}

impl std::fmt::Display for ObjectNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Object {}", self.object_id)?;

        if let Some(version) = self.version {
            write!(f, " with version {version}")?;
        }

        write!(f, " not found")
    }
}

impl std::error::Error for ObjectNotFoundError {}

impl From<ObjectNotFoundError> for crate::RpcError {
    fn from(value: ObjectNotFoundError) -> Self {
        Self::new(tonic::Code::NotFound, value.to_string())
    }
}

#[derive(Debug)]
pub struct CheckpointNotFoundError {
    sequence_number: Option<u64>,
    digest: Option<rtd_sdk_types::Digest>,
}

impl CheckpointNotFoundError {
    pub fn sequence_number(sequence_number: u64) -> Self {
        Self {
            sequence_number: Some(sequence_number),
            digest: None,
        }
    }

    pub fn digest(digest: rtd_sdk_types::Digest) -> Self {
        Self {
            sequence_number: None,
            digest: Some(digest),
        }
    }
}

impl std::fmt::Display for CheckpointNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Checkpoint ")?;

        if let Some(s) = self.sequence_number {
            write!(f, "{s} ")?;
        }

        if let Some(d) = &self.digest {
            write!(f, "{d} ")?;
        }

        write!(f, "not found")
    }
}

impl std::error::Error for CheckpointNotFoundError {}

impl From<CheckpointNotFoundError> for crate::RpcError {
    fn from(value: CheckpointNotFoundError) -> Self {
        Self::new(tonic::Code::NotFound, value.to_string())
    }
}

#[derive(Debug)]
pub struct EpochNotFoundError {
    epoch: rtd_sdk_types::EpochId,
}

impl EpochNotFoundError {
    pub fn new(epoch: rtd_sdk_types::EpochId) -> Self {
        Self { epoch }
    }
}

impl std::fmt::Display for EpochNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Epoch {} not found", self.epoch)
    }
}

impl std::error::Error for EpochNotFoundError {}

impl From<EpochNotFoundError> for crate::RpcError {
    fn from(value: EpochNotFoundError) -> Self {
        Self::new(tonic::Code::NotFound, value.to_string())
    }
}

#[cfg(test)]
mod dependency_error_tests {
    use super::*;

    #[tokio::test]
    async fn classified_bad_request_keeps_structured_details() {
        let bad_request = BadRequest::from(
            crate::proto::google::rpc::bad_request::FieldViolation::new_at("digests", 2)
                .with_description("invalid digest")
                .with_reason(ErrorReason::FieldInvalid),
        );
        let expected = RpcError::from(bad_request.clone()).into_status_proto();
        let wrapped = anyhow::Error::new(RpcError::from(bad_request))
            .context("private read stage")
            .context("private rendering stage");
        let actual = RpcError::from(wrapped).into_status_proto();
        assert_eq!(actual, expected);
        assert!(!actual.details.is_empty());
    }

    #[test]
    fn classified_rpc_error_survives_pipeline_anyhow_roundtrip() {
        for code in [
            Code::Unavailable,
            Code::DeadlineExceeded,
            Code::ResourceExhausted,
            Code::NotFound,
            Code::InvalidArgument,
            Code::Internal,
        ] {
            let original = RpcError::new(code, "classified public error");
            let wrapped = anyhow::Error::new(original)
                .context("private pipeline stage")
                .context("outer context");
            let status: tonic::Status = RpcError::from(wrapped).into();
            assert_eq!(status.code(), code);
            assert_eq!(status.message(), "classified public error");
        }
    }

    #[test]
    fn dependency_status_survives_nested_context_without_transport_details() {
        for code in [
            Code::Unavailable,
            Code::DeadlineExceeded,
            Code::ResourceExhausted,
        ] {
            let error = anyhow::Error::new(tonic::Status::new(code, "private transport endpoint"))
                .context("read rows")
                .context("resolve object");
            let status: tonic::Status = RpcError::from(error).into();
            assert_eq!(status.code(), code);
            assert!(!status.message().contains("private"));
            assert!(!status.message().contains("resolve object"));
        }
    }

    #[tokio::test]
    async fn unknown_status_requires_typed_transport_failure_to_be_retryable() {
        let mut upstream = tonic::Status::unknown("private h2 response body");
        upstream.set_source(std::sync::Arc::new(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "private peer",
        )));
        let status: tonic::Status =
            RpcError::from(anyhow::Error::new(upstream).context("read checkpoint")).into();
        assert_eq!(status.code(), Code::Unavailable);
        assert_eq!(status.message(), "Dependency connection interrupted");
        let mut upstream = tonic::Status::unknown("h2 body failure");
        let (client, peer) = tokio::io::duplex(1024);
        drop(peer);
        let transport_error = match h2::client::handshake(client).await {
            Err(error) => error,
            Ok((_, connection)) => connection.await.expect_err("closed peer must fail"),
        };
        assert!(transport_error.get_io().is_some());
        upstream.set_source(std::sync::Arc::new(transport_error));
        let status: tonic::Status = RpcError::from(anyhow::Error::new(upstream)).into();
        assert_eq!(status.code(), Code::Unavailable);
        let status: tonic::Status = RpcError::from(anyhow::Error::new(tonic::Status::unknown(
            "UnexpectedEof is just a message",
        )))
        .into();
        assert_eq!(status.code(), Code::Internal);
    }

    #[test]
    fn dependency_data_errors_are_not_reclassified_as_retryable_or_missing() {
        for code in [
            Code::Internal,
            Code::DataLoss,
            Code::NotFound,
            Code::PermissionDenied,
        ] {
            let status: tonic::Status = RpcError::from(anyhow::Error::new(tonic::Status::new(
                code,
                "invalid stored data",
            )))
            .into();
            assert_eq!(status.code(), Code::Internal);
        }
        let status: tonic::Status = RpcError::from(anyhow::anyhow!("corrupt object")).into();
        assert_eq!(status.code(), Code::Internal);
    }
}
