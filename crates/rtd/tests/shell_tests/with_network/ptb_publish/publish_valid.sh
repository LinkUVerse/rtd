# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

chain_id=$(rtd client --client.config $CONFIG chain-identifier)
echo "[environments]" >> test_pkg/Move.toml
echo "localnet = \"$chain_id\"" >> test_pkg/Move.toml

rtd client --client.config $CONFIG ptb \
 --move-call rtd::tx_context::sender \
 --assign sender \
 --publish "test_pkg" \
 --assign upgrade_cap \
 --transfer-objects "[upgrade_cap]" sender \
 2>&1 > output.log
