#!/bin/sh
# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# fast fail.
set -e

DIR="$( cd "$( dirname "$0" )" && pwd )"
REPO_ROOT="$(git rev-parse --show-toplevel)"
DOCKERFILE="$DIR/Dockerfile"
GIT_REVISION="$(git describe --always --abbrev=12 --dirty --exclude '*')"
BUILD_DATE="$(date -u +'%Y-%m-%d')"
: "${RTD_GENESIS_BLOB_URL:?Set RTD_GENESIS_BLOB_URL to a verified RTD genesis blob URL before building}"
: "${RTD_GENESIS_BLOB_SHA256:?Set RTD_GENESIS_BLOB_SHA256 to the verified genesis blob digest before building}"
if ! printf '%s\n' "$RTD_GENESIS_BLOB_SHA256" | grep -Eq '^[[:xdigit:]]{64}$'; then
    echo "RTD_GENESIS_BLOB_SHA256 must be a 64-character SHA-256 digest" >&2
    exit 1
fi

echo
echo "Building rtd-rosetta docker image"
echo "Dockerfile: \t$DOCKERFILE"
echo "docker context: $REPO_ROOT"
echo "build date: \t$BUILD_DATE"
echo "git revision: \t$GIT_REVISION"
echo

docker build -f "$DOCKERFILE" "$REPO_ROOT" \
  -t linku/rtd-rosetta-devnet \
	--build-arg GIT_REVISION="$GIT_REVISION" \
	--build-arg BUILD_DATE="$BUILD_DATE" \
	--build-arg RTD_GENESIS_BLOB_URL="$RTD_GENESIS_BLOB_URL" \
	--build-arg RTD_GENESIS_BLOB_SHA256="$RTD_GENESIS_BLOB_SHA256" \
	"$@"
