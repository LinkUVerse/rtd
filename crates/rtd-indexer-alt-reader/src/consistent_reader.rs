// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use anyhow::anyhow;
use prometheus::Registry;
use rtd_indexer_alt_consistent_api::proto::rpc::consistent::v1alpha::consistent_service_client::ConsistentServiceClient;
use rtd_types::base_types::ObjectDigest;
use rtd_types::base_types::ObjectID;
use rtd_types::base_types::ObjectRef;
use rtd_types::base_types::SequenceNumber;
use tonic::transport::Channel;
use tracing::instrument;
use url::Url;

pub use rtd_indexer_alt_consistent_api::proto::rpc::consistent::v1alpha as proto;

use crate::error::is_grpc_dependency_unavailable;
use crate::grpc_tls;
use crate::metrics::ConsistentReaderMetrics;

#[derive(clap::Args, Debug, Clone, Default)]
pub struct ConsistentReaderArgs {
    /// URL of the consistent store gRPC service
    #[arg(long)]
    pub consistent_store_url: Option<Url>,

    /// Time spent waiting for a request to complete, in milliseconds
    #[arg(long)]
    pub consistent_store_statement_timeout_ms: Option<u64>,
}

/// A reader backed by the consistent store gRPC service.
#[derive(Clone)]
pub struct ConsistentReader {
    client: Option<Client>,
    timeout: Option<Duration>,
    metrics: Arc<ConsistentReaderMetrics>,
}

/// Response from a paginated query.
pub struct Page<T> {
    pub results: Vec<Edge<T>>,
    pub has_previous_page: bool,
    pub has_next_page: bool,
}

/// An individual element from a paginated query, includes the value and its token (pagination
/// cursor).
pub struct Edge<T> {
    pub token: Vec<u8>,
    pub value: T,
}

type Client = ConsistentServiceClient<Channel>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Internal(#[from] anyhow::Error),

    #[error("{}", .0.message())]
    OutOfRange(#[source] tonic::Status),

    #[error("Consistent store unavailable: {}", .0.message())]
    Unavailable(#[source] tonic::Status),

    #[error("Consistent store client not configured")]
    NotConfigured,
}

impl ConsistentReaderArgs {
    pub fn statement_timeout(&self) -> Option<Duration> {
        self.consistent_store_statement_timeout_ms
            .map(Duration::from_millis)
    }
}

impl ConsistentReader {
    pub async fn new(
        prefix: Option<&str>,
        args: ConsistentReaderArgs,
        registry: &Registry,
    ) -> Result<Self, Error> {
        let client = if let Some(url) = &args.consistent_store_url {
            let uri = url
                .as_str()
                .parse()
                .context("invalid Consistent gRPC URL")?;
            let mut endpoint = grpc_tls::channel_endpoint(uri, "RTD_ARCHIVE_CONSISTENT_GRPC")
                .context("Failed to create channel for gRPC endpoint")?;

            if let Some(timeout) = args.statement_timeout() {
                endpoint = endpoint.timeout(timeout);
            }

            let channel = endpoint.connect_lazy();

            Some(ConsistentServiceClient::new(channel))
        } else {
            None
        };

        let timeout = args.statement_timeout();
        let metrics = ConsistentReaderMetrics::new(prefix, registry);

        Ok(Self {
            client,
            timeout,
            metrics,
        })
    }

    /// Get the consistent store's watermarks, as of the given `checkpoint`.
    #[instrument(skip(self), level = "debug")]
    pub async fn available_range(
        &self,
        checkpoint: u64,
    ) -> Result<proto::AvailableRangeResponse, Error> {
        self.request(
            "available_range",
            Some(checkpoint),
            |mut client, request| async move { client.available_range(request).await },
            proto::AvailableRangeRequest {},
        )
        .await
    }

    /// Batch get balances for multiple (owner, coin_type) pairs at a given checkpoint.
    #[instrument(skip(self), level = "debug")]
    pub async fn batch_get_balances(
        &self,
        checkpoint: u64,
        requests: Vec<(String, String)>,
    ) -> Result<Vec<proto::Balance>, Error> {
        let response = self
            .request(
                "batch_get_balances",
                Some(checkpoint),
                |mut client, request| async move { client.batch_get_balances(request).await },
                proto::BatchGetBalancesRequest {
                    requests: requests
                        .into_iter()
                        .map(|(owner, coin_type)| proto::GetBalanceRequest {
                            owner: Some(owner),
                            coin_type: Some(coin_type),
                        })
                        .collect(),
                },
            )
            .await?;

        let mut results = vec![];
        for balance in response.balances {
            results.push(balance);
        }

        Ok(results)
    }

    /// Get the balance for a single (owner, coin_type) pair at a given checkpoint.
    #[instrument(skip(self), level = "debug")]
    pub async fn get_balance(
        &self,
        checkpoint: Option<u64>,
        address: String,
        coin_type: String,
    ) -> Result<proto::Balance, Error> {
        self.request(
            "get_balance",
            checkpoint,
            |mut client, request| async move { client.get_balance(request).await },
            proto::GetBalanceRequest {
                owner: Some(address),
                coin_type: Some(coin_type),
            },
        )
        .await
    }

    /// Paginate coin balances for `address`, at checkpoint `checkpoint`.
    #[instrument(skip(self), level = "debug")]
    pub async fn list_balances(
        &self,
        checkpoint: Option<u64>,
        address: String,
        page_size: Option<u32>,
        after_token: Option<Vec<u8>>,
        before_token: Option<Vec<u8>>,
        is_from_front: bool,
    ) -> Result<Page<proto::Balance>, Error> {
        let response = self
            .request(
                "list_balances",
                checkpoint,
                |mut client, request| async move { client.list_balances(request).await },
                proto::ListBalancesRequest {
                    owner: Some(address),
                    page_size,
                    after_token: after_token.map(Into::into),
                    before_token: before_token.map(Into::into),
                    end: if is_from_front {
                        Some(proto::End::Front.into())
                    } else {
                        Some(proto::End::Back.into())
                    },
                },
            )
            .await?;

        let has_next_page = response.has_next_page();
        let has_previous_page = response.has_previous_page();

        let results = response
            .balances
            .into_iter()
            .map(|b| Edge {
                token: b.page_token.clone().unwrap_or_default().into(),
                value: b,
            })
            .collect();

        Ok(Page {
            results,
            has_next_page,
            has_previous_page,
        })
    }

    /// Paginate live objects with type filter `object_type`, at checkpoint `checkpoint`.
    #[instrument(skip(self), level = "debug")]
    pub async fn list_objects_by_type(
        &self,
        checkpoint: Option<u64>,
        object_type: String,
        page_size: Option<u32>,
        after_token: Option<Vec<u8>>,
        before_token: Option<Vec<u8>>,
        is_from_front: bool,
    ) -> Result<Page<ObjectRef>, Error> {
        let response = self
            .request(
                "list_objects_by_type",
                checkpoint,
                |mut client, request| async move { client.list_objects_by_type(request).await },
                proto::ListObjectsByTypeRequest {
                    object_type: Some(object_type),
                    page_size,
                    after_token: after_token.map(Into::into),
                    before_token: before_token.map(Into::into),
                    end: if is_from_front {
                        Some(proto::End::Front.into())
                    } else {
                        Some(proto::End::Back.into())
                    },
                },
            )
            .await?;

        let has_next_page = response.has_next_page();
        let has_previous_page = response.has_previous_page();

        let results = response
            .objects
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Page {
            results,
            has_next_page,
            has_previous_page,
        })
    }

    /// Paginate live objects at `checkpoint`, with owner described by `kind` and `address`, and an
    /// optional, `object_type`. If `object_type` is prefaced with `!`, objects of the type prefix
    /// will be excluded from the page of results.
    #[instrument(skip(self), level = "debug")]
    pub async fn list_owned_objects(
        &self,
        checkpoint: Option<u64>,
        kind: proto::owner::OwnerKind,
        address: Option<String>,
        object_type: Option<String>,
        page_size: Option<u32>,
        after_token: Option<Vec<u8>>,
        before_token: Option<Vec<u8>>,
        is_from_front: bool,
    ) -> Result<Page<ObjectRef>, Error> {
        let response = self
            .request(
                "list_owned_objects",
                checkpoint,
                |mut client, request| async move { client.list_owned_objects(request).await },
                proto::ListOwnedObjectsRequest {
                    owner: Some(proto::Owner {
                        kind: Some(kind.into()),
                        address,
                    }),
                    object_type,
                    page_size,
                    after_token: after_token.map(Into::into),
                    before_token: before_token.map(Into::into),
                    end: if is_from_front {
                        Some(proto::End::Front.into())
                    } else {
                        Some(proto::End::Back.into())
                    },
                },
            )
            .await?;

        let has_next_page = response.has_next_page();
        let has_previous_page = response.has_previous_page();

        let results = response
            .objects
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Page {
            results,
            has_next_page,
            has_previous_page,
        })
    }

    async fn request<I, O, Fut, F>(
        &self,
        method: &str,
        checkpoint: Option<u64>,
        response: F,
        input: I,
    ) -> Result<O, Error>
    where
        F: FnOnce(Client, tonic::Request<I>) -> Fut,
        Fut: Future<Output = Result<tonic::Response<O>, tonic::Status>>,
    {
        let Some(client) = self.client.clone() else {
            return Err(Error::NotConfigured);
        };

        self.metrics
            .requests_received
            .with_label_values(&[method])
            .inc();

        let _timer = self
            .metrics
            .latency
            .with_label_values(&[method])
            .start_timer();

        let mut request = tonic::Request::new(input);

        if let Some(timeout) = self.timeout {
            request.set_timeout(timeout);
        }

        if let Some(checkpoint) = checkpoint {
            request.metadata_mut().insert(
                proto::CHECKPOINT_HEIGHT_METADATA,
                checkpoint
                    .to_string()
                    .parse()
                    .with_context(|| format!("Invalid checkpoint {checkpoint}"))?,
            );
        }

        let response = response(client, request)
            .await
            .map(|r| r.into_inner())
            .map_err(Into::into);

        if response.is_ok() {
            self.metrics
                .requests_succeeded
                .with_label_values(&[method])
                .inc();
        } else {
            self.metrics
                .requests_failed
                .with_label_values(&[method])
                .inc()
        }

        response
    }
}

impl TryFrom<proto::Object> for Edge<ObjectRef> {
    type Error = Error;

    fn try_from(proto: proto::Object) -> Result<Self, Error> {
        let object_id: ObjectID = proto
            .object_id
            .context("object ID missing")?
            .parse()
            .context("invalid object ID")?;

        let digest: ObjectDigest = proto
            .digest
            .context("digest missing")?
            .parse()
            .context("invalid digest")?;

        let version: SequenceNumber = proto.version.context("version missing")?.into();
        let token: Vec<u8> = proto.page_token.unwrap_or_default().into();

        Ok(Edge {
            token,
            value: (object_id, version, digest),
        })
    }
}

impl From<tonic::Status> for Error {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::OutOfRange => Error::OutOfRange(status),
            _ if is_grpc_dependency_unavailable(&status) => Error::Unavailable(status),
            _ => Error::Internal(anyhow!(status.code()).context(status.message().to_string())),
        }
    }
}

#[cfg(test)]
mod availability_tests {
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicI32;
    use std::sync::atomic::Ordering;

    use prometheus::Registry;
    use tokio::net::TcpListener;
    use tokio_stream::wrappers::TcpListenerStream;
    use tonic::Code;
    use tonic::Request;
    use tonic::Response;
    use tonic::Status;

    use super::ConsistentReader;
    use super::ConsistentReaderArgs;
    use super::Error;
    use super::proto;
    use super::proto::consistent_service_server::ConsistentService;
    use super::proto::consistent_service_server::ConsistentServiceServer;

    #[derive(Clone, Default)]
    struct Backend {
        code: Arc<AtomicI32>,
        checkpoints: Arc<Mutex<Vec<String>>>,
    }

    #[tonic::async_trait]
    impl ConsistentService for Backend {
        async fn available_range(
            &self,
            request: Request<proto::AvailableRangeRequest>,
        ) -> Result<Response<proto::AvailableRangeResponse>, Status> {
            let checkpoint = request
                .metadata()
                .get(proto::CHECKPOINT_HEIGHT_METADATA)
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
            assert!(request.metadata().contains_key("grpc-timeout"));
            self.checkpoints.lock().unwrap().push(checkpoint);
            let code = Code::from_i32(self.code.load(Ordering::Acquire));
            if code != Code::Ok {
                return Err(Status::new(code, "private backend fixture"));
            }
            Ok(Response::new(proto::AvailableRangeResponse {
                min_checkpoint: Some(10),
                max_checkpoint: Some(21),
                ..Default::default()
            }))
        }

        async fn batch_get_balances(
            &self,
            _: Request<proto::BatchGetBalancesRequest>,
        ) -> Result<Response<proto::BatchGetBalancesResponse>, Status> {
            Err(Status::unimplemented("unused fixture method"))
        }

        async fn get_balance(
            &self,
            _: Request<proto::GetBalanceRequest>,
        ) -> Result<Response<proto::Balance>, Status> {
            Err(Status::unimplemented("unused fixture method"))
        }

        async fn list_balances(
            &self,
            _: Request<proto::ListBalancesRequest>,
        ) -> Result<Response<proto::ListBalancesResponse>, Status> {
            Err(Status::unimplemented("unused fixture method"))
        }

        async fn list_objects_by_type(
            &self,
            _: Request<proto::ListObjectsByTypeRequest>,
        ) -> Result<Response<proto::ListObjectsResponse>, Status> {
            Err(Status::unimplemented("unused fixture method"))
        }

        async fn list_owned_objects(
            &self,
            _: Request<proto::ListOwnedObjectsRequest>,
        ) -> Result<Response<proto::ListObjectsResponse>, Status> {
            Err(Status::unimplemented("unused fixture method"))
        }

        async fn service_config(
            &self,
            _: Request<proto::ServiceConfigRequest>,
        ) -> Result<Response<proto::ServiceConfigResponse>, Status> {
            Err(Status::unimplemented("unused fixture method"))
        }
    }

    #[tokio::test]
    async fn transport_failures_are_not_internal_or_out_of_range() {
        for status in [
            Status::cancelled("pooled transport closed"),
            Status::unavailable("offline"),
            Status::deadline_exceeded("timeout"),
        ] {
            assert!(matches!(Error::from(status), Error::Unavailable(_)));
        }
        assert!(matches!(
            Error::from(Status::out_of_range("pruned")),
            Error::OutOfRange(_)
        ));
        assert!(matches!(
            Error::from(Status::invalid_argument("invalid")),
            Error::Internal(_)
        ));
    }

    #[tokio::test]
    async fn live_grpc_failures_preserve_bounds_and_recover_without_cached_success() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let backend = Backend::default();
        let service = ConsistentServiceServer::new(backend.clone());
        let (shutdown, stopped) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(service)
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async move {
                    let _ = stopped.await;
                })
                .await
                .unwrap();
        });
        let reader = ConsistentReader::new(
            None,
            ConsistentReaderArgs {
                consistent_store_url: Some(format!("http://{addr}").parse().unwrap()),
                consistent_store_statement_timeout_ms: Some(1000),
            },
            &Registry::new(),
        )
        .await
        .unwrap();
        assert_eq!(
            reader.available_range(21).await.unwrap().max_checkpoint,
            Some(21)
        );
        for code in [
            Code::Cancelled,
            Code::Unavailable,
            Code::DeadlineExceeded,
            Code::OutOfRange,
            Code::PermissionDenied,
            Code::InvalidArgument,
        ] {
            backend.code.store(code as i32, Ordering::Release);
            let error = reader.available_range(21).await.unwrap_err();
            match (code, error) {
                (
                    Code::Cancelled | Code::Unavailable | Code::DeadlineExceeded,
                    Error::Unavailable(status),
                )
                | (Code::OutOfRange, Error::OutOfRange(status)) => assert_eq!(status.code(), code),
                (Code::PermissionDenied | Code::InvalidArgument, Error::Internal(_)) => {}
                (code, error) => panic!("unexpected {code:?}: {error:?}"),
            }
        }
        backend.code.store(Code::Ok as i32, Ordering::Release);
        assert_eq!(
            reader.available_range(21).await.unwrap().min_checkpoint,
            Some(10)
        );
        assert_eq!(*backend.checkpoints.lock().unwrap(), vec!["21"; 8]);
        shutdown.send(()).unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(5), server)
            .await
            .unwrap()
            .unwrap();
        let stopped = reader.available_range(21).await;
        assert!(
            matches!(&stopped, Err(Error::Unavailable(_))),
            "{stopped:?}"
        );
        assert_eq!(
            reader
                .metrics
                .requests_succeeded
                .with_label_values(&["available_range"])
                .get(),
            2
        );
        assert_eq!(
            reader
                .metrics
                .requests_failed
                .with_label_values(&["available_range"])
                .get(),
            7
        );
    }
}
