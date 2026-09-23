// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Utilities for working with chain identifier strings.
//!
//! A chain identifier can appear in two formats:
//! - the legacy short form: the first 4 bytes of the digest, hex-encoded. This is
//!   the format returned by JSON-RPC, the format rendered by `ChainIdentifier`'s `Display`, and the
//!   only format this tooling ever *writes* — into `Move.toml` and `client.yaml`.
//! - the full Base58 form: the full genesis checkpoint digest, Base58-encoded (the format returned
//!   by the gRPC and GraphQL APIs).
//!
//! We never write the Base58 form, but a user may paste it (e.g. copied from a gRPC/GraphQL
//! response or an explorer) into a `Move.toml` `[environments]` entry, while the CLI environment
//! and `client.yaml` cache hold the hex short form. Comparisons must therefore treat the two
//! encodings of the same digest as equal.

use std::str::FromStr;

use rtd_types::digests::{ChainIdentifier, CheckpointDigest};

/// The full Base58-encoded genesis checkpoint digest for `chain_id` — the canonical chain
/// identifier format. (`ChainIdentifier`'s `Display` renders the legacy 4-byte hex short form.)
pub fn chain_id_base58(chain_id: &ChainIdentifier) -> String {
    CheckpointDigest::new(*chain_id.as_bytes()).base58_encode()
}

/// Compare two chain identifier strings, each of which may independently be in the canonical
/// Base58 form or the legacy hex short form. A short form matches a full form if it encodes the
/// first 4 bytes of the digest. Strings in neither format (e.g. ad-hoc identifiers used by tests
/// or other flavors) only match by exact string equality.
pub fn chain_ids_match(a: &str, b: &str) -> bool {
    match (decode_chain_id(a), decode_chain_id(b)) {
        (Some(a), Some(b)) => {
            let len = a.len().min(b.len());
            a[..len] == b[..len]
        }
        _ => a == b,
    }
}

/// Decode a chain identifier string into digest bytes: all 32 bytes for the canonical Base58
/// form, the first 4 for the legacy hex short form; `None` if the string is in neither format.
fn decode_chain_id(s: &str) -> Option<Vec<u8>> {
    if let Ok(digest) = CheckpointDigest::from_str(s) {
        return Some(digest.inner().to_vec());
    }

    let hex = s.strip_prefix("0x").unwrap_or(s);
    if hex.len() == 8 {
        let bytes: Option<Vec<u8>> = (0..8)
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
            .collect();
        return bytes;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST_SHORT: &str = "12121212";
    const SECOND_SHORT: &str = "34343434";

    fn sample_chain_ids() -> (String, String) {
        (
            chain_id_base58(&ChainIdentifier::from(CheckpointDigest::new([0x12; 32]))),
            chain_id_base58(&ChainIdentifier::from(CheckpointDigest::new([0x34; 32]))),
        )
    }

    #[test]
    fn base58_form_of_sample_chain() {
        let (first, _) = sample_chain_ids();
        assert_eq!(
            CheckpointDigest::from_str(&first).unwrap(),
            CheckpointDigest::new([0x12; 32])
        );
    }

    #[test]
    fn full_matches_full() {
        let (first, second) = sample_chain_ids();
        assert!(chain_ids_match(&first, &first));
        assert!(!chain_ids_match(&first, &second));
    }

    #[test]
    fn short_matches_full_in_either_order() {
        let (first, _) = sample_chain_ids();
        assert!(chain_ids_match(FIRST_SHORT, &first));
        assert!(chain_ids_match(&first, FIRST_SHORT));
        assert!(!chain_ids_match(SECOND_SHORT, &first));
    }

    #[test]
    fn short_matches_short() {
        assert!(chain_ids_match(FIRST_SHORT, FIRST_SHORT));
        assert!(chain_ids_match(FIRST_SHORT, "12121212"));
        assert!(chain_ids_match("0x12121212", FIRST_SHORT));
        assert!(!chain_ids_match(FIRST_SHORT, SECOND_SHORT));
    }

    #[test]
    fn opaque_ids_match_exactly() {
        assert!(chain_ids_match("localnet", "localnet"));
        assert!(!chain_ids_match("localnet", "Localnet"));
        let (first, _) = sample_chain_ids();
        assert!(!chain_ids_match("localnet", &first));
    }
}
