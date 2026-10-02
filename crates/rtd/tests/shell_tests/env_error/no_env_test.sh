# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# This tests the error message when you set your local client to an ephemeral network and then do `rtd move test`

echo "== should fail and ask user to provide -e =="
rtd move --client.config client.yaml test
