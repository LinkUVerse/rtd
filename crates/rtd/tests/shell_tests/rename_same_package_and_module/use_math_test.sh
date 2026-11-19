# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# tests also work for the same scenario (one package uses two packages that name themselves `math`)
rtd move --client.config $CONFIG test -p use_math
