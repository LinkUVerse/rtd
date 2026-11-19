# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# This tests the error message when you set your local client to an ephemeral network and then do `rtd client publish`

echo "== should fail and suggest test-publish or adding env to manifest =="
rtd client --client.config client.yaml publish
