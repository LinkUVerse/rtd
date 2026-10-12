// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::pin::pin;

use bytes::Bytes;
use futures::stream;
use prost::Message;
use tonic::codec::{Codec, CompressionEncoding, EncodeBody, SingleMessageCompressionOverride};
use tonic::codegen::Body;
use tonic::{Code, Status};

#[derive(Clone, PartialEq, Message)]
struct Payload {
    #[prost(bytes = "bytes", tag = "1")]
    bytes: Bytes,
}

async fn encode(messages: Vec<Payload>, limit: usize, compressed: bool) -> (Vec<u8>, Code) {
    let mut codec = tonic_prost::ProstCodec::<Payload, Payload>::default();
    let body = EncodeBody::new_server(
        codec.encoder(),
        stream::iter(messages.into_iter().map(Ok::<_, Status>)),
        compressed.then_some(CompressionEncoding::Zstd),
        SingleMessageCompressionOverride::default(),
        Some(limit),
    );
    let mut body = pin!(body);
    let mut data = Vec::new();
    let mut code = None;
    while let Some(frame) = futures::future::poll_fn(|cx| body.as_mut().poll_frame(cx)).await {
        let frame = frame.unwrap();
        if let Some(bytes) = frame.data_ref() {
            data.extend_from_slice(bytes);
        }
        if let Some(trailers) = frame.trailers_ref() {
            code = Some(Status::from_header_map(trailers).unwrap().code());
            assert!(body.is_end_stream());
            break;
        }
    }
    (data, code.expect("server must emit terminal status"))
}

#[tokio::test]
async fn exact_encoded_boundary_includes_protobuf_but_excludes_frame_header() {
    let payload = Payload {
        bytes: Bytes::from(vec![42; 125]),
    };
    assert_eq!(payload.encoded_len(), 127);
    let (data, code) = encode(vec![payload.clone()], 127, false).await;
    assert_eq!(code, Code::Ok);
    assert_eq!(data.len(), 132);
    assert_eq!(&data[..5], &[0, 0, 0, 0, 127]);
    let (data, code) = encode(vec![payload], 126, false).await;
    assert_eq!(code, Code::OutOfRange);
    assert!(data.is_empty());
}

#[tokio::test]
async fn oversized_stream_item_never_emits_its_frame() {
    let small = Payload {
        bytes: Bytes::from(vec![42; 125]),
    };
    let big = Payload {
        bytes: Bytes::from(vec![43; 126]),
    };
    let (data, code) = encode(vec![small.clone(), big, small], 127, false).await;
    assert_eq!(code, Code::OutOfRange);
    // HTTP consumers stop at terminal trailers; any prefix emitted before them must be legal.
    assert!(data.is_empty() || data.len() == 132);
    if !data.is_empty() {
        assert_eq!(&data[..5], &[0, 0, 0, 0, 127]);
    }
}

#[tokio::test]
async fn compression_limit_is_on_encoded_wire_body() {
    let payload = Payload {
        bytes: Bytes::from(vec![42; 4096]),
    };
    let (data, code) = encode(vec![payload], 127, true).await;
    assert_eq!(code, Code::Ok);
    assert_eq!(data[0], 1);
    assert!(data.len() <= 132);
}

#[tokio::test]
async fn production_limit_rejects_a_real_64_mib_message_without_data() {
    let payload = Payload {
        bytes: Bytes::from(vec![42; crate::MAX_ENCODED_RESPONSE_MESSAGE_SIZE]),
    };
    assert!(payload.encoded_len() > crate::MAX_ENCODED_RESPONSE_MESSAGE_SIZE);
    let (data, code) = encode(
        vec![payload],
        crate::MAX_ENCODED_RESPONSE_MESSAGE_SIZE,
        false,
    )
    .await;
    assert_eq!(code, Code::OutOfRange);
    assert!(data.is_empty());
}
