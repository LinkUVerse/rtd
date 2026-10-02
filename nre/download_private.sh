#!/bin/bash
# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

if [ "$#" -ne 1 ] || [[ ! "$1" =~ ^[0-9a-fA-F]{7,40}$ ]]; then
    echo "Usage: RTD_RELEASE_BASE_URL=https://<trusted-host>/releases RTD_RELEASE_PUBLIC_KEY=/path/to/key.pem $0 <commit-sha>" >&2
    exit 1
fi

commit_sha=$1
base_url=${RTD_RELEASE_BASE_URL:-}
pub_key=${RTD_RELEASE_PUBLIC_KEY:-}

if [[ ! "$base_url" =~ ^https://[^/]+/.+ ]] || [ ! -f "$pub_key" ]; then
    echo "Error: set RTD_RELEASE_BASE_URL to a trusted HTTPS artifact directory and RTD_RELEASE_PUBLIC_KEY to a local trusted public key file." >&2
    exit 1
fi
if ! command -v cosign >/dev/null 2>&1 || ! command -v curl >/dev/null 2>&1; then
    echo "Error: cosign and curl are required." >&2
    exit 1
fi

url=${base_url%/}/$commit_sha
tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT

echo "[+] Downloading rtd binaries for $commit_sha ..."
for binary in rtd rtd-node rtd-tool; do
    curl --fail --show-error --silent --location --proto '=https' --proto-redir '=https' \
        --connect-timeout 10 --max-time 300 "$url/$binary" -o "$tmp_dir/$binary"
    curl --fail --show-error --silent --location --proto '=https' --proto-redir '=https' \
        --connect-timeout 10 --max-time 300 "$url/$binary.sig" -o "$tmp_dir/$binary.sig"
done

echo "[+] Verifying rtd binaries for $commit_sha ..."
for binary in rtd rtd-node rtd-tool; do
    if ! cosign verify-blob --insecure-ignore-tlog --key "$pub_key" \
        --signature "$tmp_dir/$binary.sig" "$tmp_dir/$binary"; then
        echo "Error: signature verification failed for $binary" >&2
        exit 1
    fi
done

for binary in rtd rtd-node rtd-tool; do
    mv "$tmp_dir/$binary" "./$binary"
done
