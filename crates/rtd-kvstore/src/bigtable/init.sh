# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0
INSTANCE_ID=${1:-rtd}
command=(
  cbt
  -instance
  "$INSTANCE_ID"
)
if [[ -n $BIGTABLE_EMULATOR_HOST ]]; then
  command+=(-project emulator)
fi

for table in objects transactions checkpoints checkpoints_by_digest watermark watermark_alt epochs; do
  (
    set -x
    "${command[@]}" createtable $table
    "${command[@]}" createfamily $table rtd
    "${command[@]}" setgcpolicy $table rtd maxversions=1
  )
done
"${command[@]}" setgcpolicy watermark rtd maxage=2d
