// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Generated protobuf types backing `rtd-rpc-store` values.
//!
//! Regenerate with `cargo +nightly -Zscript codegen.rs` from the
//! crate root; see `codegen.rs` for details.

#[allow(clippy::all)]
pub mod rtd {
    pub mod rpc_store {
        pub mod v1alpha {
            include!("generated/rtd.rpc_store.v1alpha.rs");
            include!("generated/rtd.rpc_store.v1alpha.accessors.rs");
        }
    }
}

pub use rtd::rpc_store::v1alpha::BalanceDelta;
pub use rtd::rpc_store::v1alpha::BitmapBlob;
pub use rtd::rpc_store::v1alpha::ObjectVersionInfo;
pub use rtd::rpc_store::v1alpha::PackageVersionInfo;
pub use rtd::rpc_store::v1alpha::PruningWatermarks;
pub use rtd::rpc_store::v1alpha::StoredCheckpointContents;
pub use rtd::rpc_store::v1alpha::StoredCheckpointSummary;
pub use rtd::rpc_store::v1alpha::StoredEffects;
pub use rtd::rpc_store::v1alpha::StoredEpoch;
pub use rtd::rpc_store::v1alpha::StoredEvents;
pub use rtd::rpc_store::v1alpha::StoredObject;
pub use rtd::rpc_store::v1alpha::StoredObjectTombstone;
pub use rtd::rpc_store::v1alpha::StoredObjectTombstoneKind;
pub use rtd::rpc_store::v1alpha::StoredTransaction;
pub use rtd::rpc_store::v1alpha::TxMetadata;
pub use rtd::rpc_store::v1alpha::stored_object;
