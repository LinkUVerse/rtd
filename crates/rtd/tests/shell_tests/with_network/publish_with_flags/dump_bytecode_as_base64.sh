# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

chain_id=$(rtd client --client.config $CONFIG chain-identifier --format=hex)
echo "[environments]" >> test_pkg/Move.toml
echo "localnet = \"$chain_id\"" >> test_pkg/Move.toml

# Calling move build (with dump-bytecode-as-base64 flags)
rtd move --client.config "$CONFIG" build -p test_pkg --dump-bytecode-as-base64
