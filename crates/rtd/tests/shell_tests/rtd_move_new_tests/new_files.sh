# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# basic test that rtd move new outputs correct files
rtd move --client.config $CONFIG new example
echo ==== files in project ====
ls -A example
echo ==== files in sources ====
ls -A example/sources
echo ==== files in tests =====
ls -A example/tests
