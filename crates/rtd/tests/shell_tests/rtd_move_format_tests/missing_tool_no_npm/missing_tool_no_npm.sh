# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# Neither `prettier-move` nor `npm` are on PATH. `rtd move format` should
# bail with the Node.js install hint and a non-zero exit code.

# Strip PATH down to the rtd binary's directory so neither `prettier-move`
# nor `npm` (which the host may have in /usr/local/bin, NVM dirs, ...) can
# leak through.
export PATH="$(dirname "$(command -v rtd)")"

rtd move --client.config $CONFIG format -c
