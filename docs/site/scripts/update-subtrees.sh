#!/usr/bin/env bash

# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

# The inherited subtree URLs include unverified RTD mirrors. Pulling them here
# would replace reviewed status pages with unreviewed third-party claims.
echo "RTD docs subtree updates require a verified source and a manual content review." >&2
exit 1
