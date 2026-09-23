# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# tests that building a package that implicitly depends on `rtd` can build
rtd move --client.config $CONFIG build -p example 2> /dev/null
