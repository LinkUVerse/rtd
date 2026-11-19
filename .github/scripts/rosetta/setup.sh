#!/bin/bash
# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

echo "Install binaries"
cargo install --locked --bin rtd --path crates/rtd
cargo install --locked --bin rtd-rosetta --path crates/rtd-rosetta

echo "run Rtd genesis"
rtd genesis

echo "generate rosetta configuration"
rtd-rosetta generate-rosetta-cli-config --online-url http://127.0.0.1:9002 --offline-url http://127.0.0.1:9003

echo "install rosetta-cli"
curl -sSfL https://raw.githubusercontent.com/coinbase/rosetta-cli/master/scripts/install.sh | sh -s