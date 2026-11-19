// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Generated protobuf types backing the opaque cursor wire format.
//!
//! These are the on-wire format only. The public API is the native
//! [`CursorToken`](super::CursorToken); it converts to and from these messages at the encode/decode
//! boundary.
//!
//! Regenerate via the `bootstrap` test (`cargo test -p rtd-rpc-api --test bootstrap`) after
//! changing any file under `crates/rtd-rpc-api/proto/`, then commit the output.

#[allow(clippy::all)]
pub mod rtd {
    pub mod rpc {
        pub mod cursor {
            pub mod v1 {
                include!("generated/rtd.rpc.cursor.v1.rs");
            }
        }
    }
}
