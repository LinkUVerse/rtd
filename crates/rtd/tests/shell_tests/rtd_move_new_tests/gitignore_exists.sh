# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# check that rtd move new correctly updates existing .gitignore
mkdir example
echo "existing_ignore" > example/.gitignore
rtd move --client.config $CONFIG new example
cat example/.gitignore
