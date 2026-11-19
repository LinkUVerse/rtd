# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# If the config file is a directory, we fail nicely
mkdir client.yaml
rtd move --client.config ./client.yaml new example \
  | sed 's/Err:.*$/Err: <REDACTED>/'
