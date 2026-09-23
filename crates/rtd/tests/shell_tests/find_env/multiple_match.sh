# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# Active environment chain ID matches multiple envs in the manifest
echo 'duplicate_env = "1234"' >> Move.toml
rtd move --client.config configs/name_mismatch_id_match.yaml build
