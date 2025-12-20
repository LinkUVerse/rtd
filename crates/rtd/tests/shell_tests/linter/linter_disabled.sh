# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# Should succeed with linting disabled (but stats should be summarized)
rtd move --client.config $CONFIG test -p example --silence-warnings
