# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# Run the `cache-package` command. Redact `path` because its value depends on
# the (platform-specific) temp directory — it's covered by the unit tests.
rtd move --client.config $CONFIG cache-package testnet 01234567 "{ local="\"$(pwd -W 2>/dev/null || pwd)/a\"" }" \
| sed 's/"path":"[^"]*"/"path":"<PATH>"/'
