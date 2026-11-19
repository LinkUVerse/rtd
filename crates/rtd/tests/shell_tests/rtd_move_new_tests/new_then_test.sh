# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# check that rtd move new followed by rtd move test succeeds
rtd move --client.config $CONFIG new example
cd example && rtd move --client.config $CONFIG test
