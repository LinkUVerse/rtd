# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# tests that rtd move new followed by rtd move disassemble succeeds


rtd move --client.config $CONFIG new example
cat > example/sources/example.move <<EOF
module example::example;

public fun foo(_ctx: &mut TxContext) {}
EOF
cd example

echo "=== Build ===" >&2
rtd move --client.config $CONFIG build

echo "=== Disassemble ===" >&2
rtd move --client.config $CONFIG disassemble build/example/bytecode_modules/example.mv
