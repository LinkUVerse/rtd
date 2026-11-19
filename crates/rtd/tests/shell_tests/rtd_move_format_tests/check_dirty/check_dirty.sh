# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# Exit-code propagation: stub `prettier-move` exits 1 (as real prettier does
# in `--check` mode when files would be reformatted). `rtd move format` must
# surface the same exit code.

chmod +x stubs/prettier-move
export PATH="$PWD/stubs:$(dirname "$(command -v rtd)")"

rtd move --client.config $CONFIG format -c example
