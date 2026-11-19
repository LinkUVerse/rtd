# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# Tests that `rtd move lint` enables Rtd mode and runs the Rtd-specific linters (here
# `self_transfer`), not just the generic Move linters. `COLOR_MODE=NONE` disables ANSI
# color codes in the compiler diagnostics so the snapshot is stable.
COLOR_MODE=NONE rtd move --client.config $CONFIG lint -p example
