#!/bin/bash
# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

if ! cosign version &> /dev/null
then
    echo "cosign in not installed, Please install cosign for binary verification."
    echo "https://docs.sigstore.dev/cosign/installation"
    exit
fi

commit_sha=$1
pub_key=https://rtd-private.s3.us-west-2.amazonaws.com/rtd_security_release.pem
url=https://rtd-releases.s3-accelerate.amazonaws.com/$commit_sha

echo "[+] Downloading rtd binaries for $commit_sha ..."
curl $url/rtd -o rtd
curl $url/rtd-indexer -o rtd-indexer
curl $url/rtd-node -o rtd-node
curl $url/rtd-tool -o rtd-tool

echo "[+] Verifying rtd binaries for $commit_sha ..."
cosign verify-blob --insecure-ignore-tlog --key $pub_key --signature $url/rtd.sig rtd
cosign verify-blob --insecure-ignore-tlog --key $pub_key --signature $url/rtd-indexer.sig rtd-indexer
cosign verify-blob --insecure-ignore-tlog --key $pub_key --signature $url/rtd-node.sig rtd-node
cosign verify-blob --insecure-ignore-tlog --key $pub_key --signature $url/rtd-tool.sig rtd-tool
