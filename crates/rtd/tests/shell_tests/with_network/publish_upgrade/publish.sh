#!/usr/bin/env bash
# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# Test a regular publish flow. Each package should have its own `Published.toml`
# for the specified environment.

# B --> A
# C --> B
# C --> A
#
# D --> B
# D --> A
#
# E --> C
# E --> D
#
# We publish A, B, C, D, E in order

add_env_to_toml() {
  echo "[environments]" >> $1/Move.toml
  echo "localnet = \"$chain_id\"" >> $1/Move.toml
}

rtd_version=$(rtd --version | sed 's/rtd \([^-]*\)-.*$/\1/g')
extract_published() {
  echo "=== $@ ==="
  cat "$@" \
    | grep -v 0x \
    | grep -v "^#" \
    | sed "s/$chain_id/CHAIN_ID/g" \
    | sed "s/$rtd_version/RTD_VERSION/g"
  echo "=== End Published.toml ==="
}

chain_id=$(rtd client --client.config $CONFIG chain-identifier --format=hex)

for i in a b c d e
do
  echo ""
  echo "=== publishing $i ==="
  add_env_to_toml $i

  rtd client --client.config $CONFIG publish $i > output.log 2>&1 || cat output.log
  extract_published $i/Published.toml
done
