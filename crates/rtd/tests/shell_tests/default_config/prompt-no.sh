# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# If the config file doesn't exist, we prompt and bail if the user says no
echo "nope" | rtd move --client.config ./client.yaml new example
cat client.yaml
cat rtd.keystore
