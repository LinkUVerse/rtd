# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# simple test just to make sure the test runner works
echo "dummy test"
cat data/data.txt
rtd move --client.config $CONFIG new dummy
