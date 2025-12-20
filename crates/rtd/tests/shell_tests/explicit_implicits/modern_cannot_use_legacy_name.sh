# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# tests that building a package that has explicit deps on legacy system names errors
rtd move --client.config $CONFIG build -p modern_using_legacy_name
