#!/bin/bash
# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

if [ "$#" -ne 2 ] || [[ ! "$1" =~ ^[0-9a-fA-F]{7,40}$ ]] || [[ ! "$2" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]]; then
    echo "Usage: RTD_RELEASE_BASE_URL=https://<trusted-host>/releases RTD_RELEASE_PUBLIC_KEY=/path/to/key.pem $0 <commit-sha> <binary-name>" >&2
    exit 1
fi

commit_sha=$1
binary_name=$2
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

echo "[+] Downloading binary '$binary_name' for $commit_sha ..."
curl --fail --show-error --silent --location --proto '=https' --proto-redir '=https' \
    --connect-timeout 10 --max-time 300 "$url/$binary_name" -o "$tmp_dir/$binary_name"
curl --fail --show-error --silent --location --proto '=https' --proto-redir '=https' \
    --connect-timeout 10 --max-time 300 "$url/$binary_name.sig" -o "$tmp_dir/$binary_name.sig"

echo "[+] Verifying binary '$binary_name' for $commit_sha ..."
cosign verify-blob --insecure-ignore-tlog --key "$pub_key" \
    --signature "$tmp_dir/$binary_name.sig" "$tmp_dir/$binary_name"
mv "$tmp_dir/$binary_name" "./$binary_name"
