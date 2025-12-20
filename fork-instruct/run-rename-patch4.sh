#!/bin/bash
set -e

# 修复 macOS sed 的 "illegal byte sequence" 错误
export LC_ALL=C
export LANG=C

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "========================================"
echo "RTD 品牌重命名补丁脚本 (Patch 4)"
echo "修复遗漏的 sui.* 文件和 sui* 目录"
echo "========================================"

# 1. 重命名 sui.* 文件
echo "重命名 sui.* 文件..."

# 逐个处理已知的文件
declare -a FILES_TO_RENAME=(
    "nre/ansible/roles/rtd-node/tasks/sui.yaml:rtd.yaml"
    "crates/rtd-aws-orchestrator/src/protocol/sui.rs:rtd.rs"
    "crates/rtd-framework/docs/sui/sui.md:rtd.md"
    "crates/rtd-core/tests/staged/sui.yaml:rtd.yaml"
    "crates/rtd-rosetta/resources/sui.ros:rtd.ros"
    "docker/rtd-antithesis/genesis/static/sui.keystore:rtd.keystore"
    "chocolatey/sui.nuspec:rtd.nuspec"
)

for entry in "${FILES_TO_RENAME[@]}"; do
    old_path="${entry%%:*}"
    new_name="${entry##*:}"

    if [ -f "$old_path" ]; then
        dir=$(dirname "$old_path")
        echo "  $old_path -> $dir/$new_name"
        mv "$old_path" "$dir/$new_name"
    fi
done

# PDF 和文档文件保持原名（这些是发布的文档，不需要重命名）
echo "  注意: sui.pdf 文件保持原名（历史文档）"

# 2. 重命名 sui* 目录
echo ""
echo "重命名 sui* 目录..."

declare -a DIRS_TO_RENAME=(
    "external-crates/move/crates/move-stackless-bytecode-2/tests/move/sui_move_2024:rtd_move_2024"
    "external-crates/move/crates/move-compiler/tests/sui_mode:rtd_mode"
    "external-crates/move/crates/move-compiler/tests/move_2024/sui_mode:rtd_mode"
    "external-crates/move/crates/move-compiler/src/sui_mode:rtd_mode"
    "crates/rtd-rpc-api/proto/sui:rtd"
    "crates/rtd-bridge/src/sui_bridge_watchdog:rtd_bridge_watchdog"
    "crates/rtd-framework/docs/sui:rtd"
    "crates/rtd-framework/docs/sui_system:rtd_system"
    "crates/rtd-graphql-rpc/examples/sui_system_state_summary:rtd_system_state_summary"
    "crates/rtd-indexer-alt-consistent-api/proto/sui:rtd"
    "crates/rtd-adapter-transactional-tests/tests/sui:rtd"
    "crates/rtd-package-resolver/tests/packages/sui:rtd"
    "crates/rtd-indexer-alt-e2e-tests/packages/suins:rtdns"
    "crates/rtd/tests/shell_tests/sui_move_new_tests:rtd_move_new_tests"
    "crates/rtd-types/src/sui_system_state:rtd_system_state"
    "docs/content/guides/suiplay0x1:rtdplay0x1"
)

for entry in "${DIRS_TO_RENAME[@]}"; do
    old_path="${entry%%:*}"
    new_name="${entry##*:}"

    if [ -d "$old_path" ]; then
        parent_dir=$(dirname "$old_path")
        echo "  $old_path -> $parent_dir/$new_name"
        mv "$old_path" "$parent_dir/$new_name"
    fi
done

# 3. 最终检查
echo ""
echo "最终检查..."

# 检查 sui.* 文件
remaining_files=$(find . -name "sui.*" ! -path "./.git/*" ! -path "./target/*" -type f 2>/dev/null | grep -v "sui.pdf" | wc -l)
if [ "$remaining_files" -gt 0 ]; then
    echo "还有 $remaining_files 个 sui.* 文件未处理（排除 pdf）:"
    find . -name "sui.*" ! -path "./.git/*" ! -path "./target/*" -type f 2>/dev/null | grep -v "sui.pdf"
else
    echo "✅ 所有 sui.* 文件已处理（pdf 文件保留）"
fi

# 检查 sui* 目录
remaining_dirs=$(find . -type d -name "sui*" ! -path "./.git/*" ! -path "./target/*" 2>/dev/null | wc -l)
if [ "$remaining_dirs" -gt 0 ]; then
    echo "还有 $remaining_dirs 个 sui* 目录未处理:"
    find . -type d -name "sui*" ! -path "./.git/*" ! -path "./target/*" 2>/dev/null
else
    echo "✅ 所有 sui* 目录已处理"
fi

echo "========================================"
echo "补丁脚本执行完成！"
echo "========================================"
