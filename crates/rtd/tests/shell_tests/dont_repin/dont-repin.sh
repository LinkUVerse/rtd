# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# This should succeed - the manifest has a broken dep, but the lockfile has it pinned to the correct location
rtd move --client.config $CONFIG build
