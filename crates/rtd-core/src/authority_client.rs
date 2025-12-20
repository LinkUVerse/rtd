// Copyright (c) 2021, Facebook, Inc. and its affiliates
// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::anyhow;
use async_trait::async_trait;
use linku_network::config::Config;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::time::Duration;
use rtd_network::{api::ValidatorClient, tonic};
use rtd_types::base_types::AuthorityName;
use rtd_types::committee::CommitteeWithNetworkMetadata;
use rtd_types::crypto::NetworkPublicKey;
use rtd_types::messages_checkpoint::{
    CheckpointRequest, CheckpointRequestV2, CheckpointResponse, CheckpointResponseV2,
};
use rtd_types::multiaddr::Multiaddr;
use rtd_types::rtd_system_state::RtdSystemState;
use rtd_types::{
    error::{RtdError, RtdResult},
    transaction::*,
};
use tap::TapFallible;

use crate::authority_client::tonic::IntoRequest;
use rtd_network::tonic::metadata::KeyAndValueRef;
use rtd_network::tonic::transport::Channel;
use rtd_types::messages_grpc::{
    HandleCertificateRequestV3, HandleCertificateResponseV2, HandleCertificateResponseV3,
    HandleSoftBundleCertificatesRequestV3, HandleSoftBundleCertificatesResponseV3,
    HandleTransactionResponse, ObjectInfoRequest, ObjectInfoResponse, RawValidatorHealthRequest,
    RawWaitForEffectsRequest, SubmitTxRequest, SubmitTxResponse, SystemStateRequest,
    TransactionInfoRequest, TransactionInfoResponse, ValidatorHealthRequest,
    ValidatorHealthResponse, WaitForEffectsRequest, WaitForEffectsResponse,
};

#[async_trait]
pub trait AuthorityAPI {
    /// Submits a transaction to validators for sequencing and execution.
    async fn submit_transaction(
        &self,
        request: SubmitTxRequest,
        client_addr: Option<SocketAddr>,
    ) -> Result<SubmitTxResponse, RtdError>;

    /// Waits for effects of a transaction that has been submitted to the network
    /// through the `submit_transaction` API.
    async fn wait_for_effects(
        &self,
        request: WaitForEffectsRequest,
        client_addr: Option<SocketAddr>,
    ) -> Result<WaitForEffectsResponse, RtdError>;

    // TODO(fastpath): Add a soft bundle path for mfp which will return the list of consensus positions

    /// Initiate a new transaction to a Rtd or Primary account.
    async fn handle_transaction(
        &self,
        transaction: Transaction,
        client_addr: Option<SocketAddr>,
    ) -> Result<HandleTransactionResponse, RtdError>;

    /// Execute a certificate.
    async fn handle_certificate_v2(
        &self,
        certificate: CertifiedTransaction,
        client_addr: Option<SocketAddr>,
    ) -> Result<HandleCertificateResponseV2, RtdError>;

    /// Execute a certificate.
    async fn handle_certificate_v3(
        &self,
        request: HandleCertificateRequestV3,
        client_addr: Option<SocketAddr>,
    ) -> Result<HandleCertificateResponseV3, RtdError>;

    /// Execute a Soft Bundle with multiple certificates.
    async fn handle_soft_bundle_certificates_v3(
        &self,
        request: HandleSoftBundleCertificatesRequestV3,
        client_addr: Option<SocketAddr>,
    ) -> Result<HandleSoftBundleCertificatesResponseV3, RtdError>;

    /// Handle Object information requests for this account.
    async fn handle_object_info_request(
        &self,
        request: ObjectInfoRequest,
    ) -> Result<ObjectInfoResponse, RtdError>;

    /// Handle Object information requests for this account.
    async fn handle_transaction_info_request(
        &self,
        request: TransactionInfoRequest,
    ) -> Result<TransactionInfoResponse, RtdError>;

    async fn handle_checkpoint(
        &self,
        request: CheckpointRequest,
    ) -> Result<CheckpointResponse, RtdError>;

    async fn handle_checkpoint_v2(
        &self,
        request: CheckpointRequestV2,
    ) -> Result<CheckpointResponseV2, RtdError>;

    // This API is exclusively used by the benchmark code.
    // Hence it's OK to return a fixed system state type.
    async fn handle_system_state_object(
        &self,
        request: SystemStateRequest,
    ) -> Result<RtdSystemState, RtdError>;

    /// Get validator health metrics (for latency measurement)
    async fn validator_health(
        &self,
        request: ValidatorHealthRequest,
    ) -> Result<ValidatorHealthResponse, RtdError>;
}

#[derive(Clone)]
pub struct NetworkAuthorityClient {
    client: RtdResult<ValidatorClient<Channel>>,
}

impl NetworkAuthorityClient {
    pub async fn connect(
        address: &Multiaddr,
        tls_target: NetworkPublicKey,
    ) -> anyhow::Result<Self> {
        let tls_config = rtd_tls::create_rustls_client_config(
            tls_target,
            rtd_tls::RTD_VALIDATOR_SERVER_NAME.to_string(),
            None,
        );
        let channel = linku_network::client::connect(address, tls_config)
            .await
            .map_err(|err| anyhow!(err.to_string()))?;
        Ok(Self::new(channel))
    }

    pub fn connect_lazy(address: &Multiaddr, tls_target: NetworkPublicKey) -> Self {
        let tls_config = rtd_tls::create_rustls_client_config(
            tls_target,
            rtd_tls::RTD_VALIDATOR_SERVER_NAME.to_string(),
            None,
        );
        let client: RtdResult<_> = linku_network::client::connect_lazy(address, tls_config)
            .map(ValidatorClient::new)
            .map_err(|err| err.to_string().into());
        Self { client }
    }

    pub fn new(channel: Channel) -> Self {
        Self {
            client: Ok(ValidatorClient::new(channel)),
        }
    }

    fn new_lazy(client: RtdResult<Channel>) -> Self {
        Self {
            client: client.map(ValidatorClient::new),
        }
    }

    pub(crate) fn client(&self) -> RtdResult<ValidatorClient<Channel>> {
        self.client.clone()
    }

    pub fn get_client_for_testing(&self) -> RtdResult<ValidatorClient<Channel>> {
        self.client()
    }
}

#[async_trait]
impl AuthorityAPI for NetworkAuthorityClient {
    /// Submits a transaction to the Rtd network for certification and execution.
    async fn submit_transaction(
        &self,
        request: SubmitTxRequest,
        client_addr: Option<SocketAddr>,
    ) -> Result<SubmitTxResponse, RtdError> {
        let mut request = request.into_raw()?.into_request();
        insert_metadata(&mut request, client_addr);

        self.client()?
            .submit_transaction(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(Into::<RtdError>::into)?
            .try_into()
    }

    async fn wait_for_effects(
        &self,
        request: WaitForEffectsRequest,
        client_addr: Option<SocketAddr>,
    ) -> Result<WaitForEffectsResponse, RtdError> {
        let raw_request: RawWaitForEffectsRequest = request.try_into()?;
        let mut request = raw_request.into_request();
        insert_metadata(&mut request, client_addr);

        self.client()?
            .wait_for_effects(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(Into::<RtdError>::into)?
            .try_into()
    }

    /// Initiate a new transfer to a Rtd or Primary account.
    async fn handle_transaction(
        &self,
        transaction: Transaction,
        client_addr: Option<SocketAddr>,
    ) -> Result<HandleTransactionResponse, RtdError> {
        let mut request = transaction.into_request();
        insert_metadata(&mut request, client_addr);

        self.client()?
            .transaction(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(Into::into)
    }

    /// Execute a certificate.
    async fn handle_certificate_v2(
        &self,
        certificate: CertifiedTransaction,
        client_addr: Option<SocketAddr>,
    ) -> Result<HandleCertificateResponseV2, RtdError> {
        let mut request = certificate.into_request();
        insert_metadata(&mut request, client_addr);

        let response = self
            .client()?
            .handle_certificate_v2(request)
            .await
            .map(tonic::Response::into_inner);

        response.map_err(Into::into)
    }

    async fn handle_certificate_v3(
        &self,
        request: HandleCertificateRequestV3,
        client_addr: Option<SocketAddr>,
    ) -> Result<HandleCertificateResponseV3, RtdError> {
        let mut request = request.into_request();
        insert_metadata(&mut request, client_addr);

        let response = self
            .client()?
            .handle_certificate_v3(request)
            .await
            .map(tonic::Response::into_inner);

        response.map_err(Into::into)
    }

    async fn handle_soft_bundle_certificates_v3(
        &self,
        request: HandleSoftBundleCertificatesRequestV3,
        client_addr: Option<SocketAddr>,
    ) -> Result<HandleSoftBundleCertificatesResponseV3, RtdError> {
        let mut request = request.into_request();
        insert_metadata(&mut request, client_addr);

        let response = self
            .client()?
            .handle_soft_bundle_certificates_v3(request)
            .await
            .map(tonic::Response::into_inner);

        response.map_err(Into::into)
    }

    async fn handle_object_info_request(
        &self,
        request: ObjectInfoRequest,
    ) -> Result<ObjectInfoResponse, RtdError> {
        self.client()?
            .object_info(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(Into::into)
    }

    /// Handle Object information requests for this account.
    async fn handle_transaction_info_request(
        &self,
        request: TransactionInfoRequest,
    ) -> Result<TransactionInfoResponse, RtdError> {
        self.client()?
            .transaction_info(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(Into::into)
    }

    /// Handle Object information requests for this account.
    async fn handle_checkpoint(
        &self,
        request: CheckpointRequest,
    ) -> Result<CheckpointResponse, RtdError> {
        self.client()?
            .checkpoint(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(Into::into)
    }

    /// Handle Object information requests for this account.
    async fn handle_checkpoint_v2(
        &self,
        request: CheckpointRequestV2,
    ) -> Result<CheckpointResponseV2, RtdError> {
        self.client()?
            .checkpoint_v2(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(Into::into)
    }

    async fn handle_system_state_object(
        &self,
        request: SystemStateRequest,
    ) -> Result<RtdSystemState, RtdError> {
        self.client()?
            .get_system_state_object(request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(Into::into)
    }

    async fn validator_health(
        &self,
        request: ValidatorHealthRequest,
    ) -> Result<ValidatorHealthResponse, RtdError> {
        let raw_request: RawValidatorHealthRequest = request.try_into()?;

        self.client()?
            .validator_health(raw_request)
            .await
            .map(tonic::Response::into_inner)
            .map_err(Into::<RtdError>::into)?
            .try_into()
    }
}

pub fn make_network_authority_clients_with_network_config(
    committee: &CommitteeWithNetworkMetadata,
    network_config: &Config,
) -> BTreeMap<AuthorityName, NetworkAuthorityClient> {
    let mut authority_clients = BTreeMap::new();
    for (name, (_state, network_metadata)) in committee.validators() {
        let address = network_metadata
            .network_address
            .clone()
            .rewrite_udp_to_tcp()
            .rewrite_http_to_https();
        let tls_config = network_metadata
            .network_public_key
            .as_ref()
            .map(|key| {
                rtd_tls::create_rustls_client_config(
                    key.clone(),
                    rtd_tls::RTD_VALIDATOR_SERVER_NAME.to_string(),
                    None,
                )
            })
            .ok_or(RtdError::from("network public key is not available"));
        let maybe_channel = tls_config
            .and_then(|tls_config| {
                network_config
                    .connect_lazy(&address, tls_config)
                    .map_err(|e| e.to_string().into())
            })
            .tap_err(|e| {
                tracing::error!(
                    address = %address,
                    name = %name,
                    "unable to create authority client: {e}"
                )
            });
        let client = NetworkAuthorityClient::new_lazy(maybe_channel);
        authority_clients.insert(*name, client);
    }
    authority_clients
}

pub fn make_authority_clients_with_timeout_config(
    committee: &CommitteeWithNetworkMetadata,
    connect_timeout: Duration,
    request_timeout: Duration,
) -> BTreeMap<AuthorityName, NetworkAuthorityClient> {
    let mut network_config = linku_network::config::Config::new();
    network_config.connect_timeout = Some(connect_timeout);
    network_config.request_timeout = Some(request_timeout);
    network_config.http2_keepalive_interval = Some(connect_timeout);
    network_config.http2_keepalive_timeout = Some(connect_timeout);
    make_network_authority_clients_with_network_config(committee, &network_config)
}

fn insert_metadata<T>(request: &mut tonic::Request<T>, client_addr: Option<SocketAddr>) {
    if let Some(client_addr) = client_addr {
        let mut metadata = tonic::metadata::MetadataMap::new();
        metadata.insert("x-forwarded-for", client_addr.to_string().parse().unwrap());
        metadata
            .iter()
            .for_each(|key_and_value| match key_and_value {
                KeyAndValueRef::Ascii(key, value) => {
                    request.metadata_mut().insert(key, value.clone());
                }
                KeyAndValueRef::Binary(key, value) => {
                    request.metadata_mut().insert_bin(key, value.clone());
                }
            });
    }
}
