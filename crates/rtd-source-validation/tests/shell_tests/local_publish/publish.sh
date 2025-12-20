# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

chain_id=$(rtd client --client.config $CONFIG chain-identifier)
echo "[environments]" >> a/Move.toml
echo "localnet = \"$chain_id\"" >> a/Move.toml
echo "[environments]" >> b/Move.toml
echo "localnet = \"$chain_id\"" >> b/Move.toml

rtd client --client.config $CONFIG publish "b" -e localnet 2>&1 > output.log
rtd client --client.config $CONFIG verify-source "b" -e localnet


rtd client --client.config $CONFIG publish "a" -e localnet 2>&1 > output.log
rtd client --client.config $CONFIG verify-source "a" -e localnet
rtd client --client.config $CONFIG verify-source "a" -e localnet --verify-deps
