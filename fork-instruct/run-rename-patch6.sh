#!/bin/bash
set -e

# 修复 macOS sed 的 "illegal byte sequence" 错误
export LC_ALL=C
export LANG=C

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "========================================"
echo "RTD 品牌重命名补丁脚本 (Patch 6)"
echo "修复遗漏的包含 sui 的 .rs 文件名"
echo "========================================"

# 1. 处理包含 _sui_ 的文件（如 epoch_start_sui_system_state.rs）
echo "Phase 1: 重命名 *_sui_*.rs 文件..."

find . -name "*_sui_*.rs" ! -path "./.git/*" ! -path "./target/*" 2>/dev/null | while read -r file; do
    dir=$(dirname "$file")
    base=$(basename "$file")
    new_base="${base/_sui_/_rtd_}"
    if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
        echo "  $file -> $dir/$new_base"
        mv "$file" "$dir/$new_base"
    fi
done

# 2. 处理 suins -> rtdns 的文件
echo ""
echo "Phase 2: 重命名 suins*.rs 文件..."

find . -name "suins*.rs" ! -path "./.git/*" ! -path "./target/*" 2>/dev/null | while read -r file; do
    dir=$(dirname "$file")
    base=$(basename "$file")
    new_base="${base/suins/rtdns}"
    if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
        echo "  $file -> $dir/$new_base"
        mv "$file" "$dir/$new_base"
    fi
done

# 3. 处理 *_sui.rs 文件（如 pay_sui.rs）
echo ""
echo "Phase 3: 重命名 *_sui.rs 文件..."

find . -name "*_sui.rs" ! -path "./.git/*" ! -path "./target/*" 2>/dev/null | while read -r file; do
    dir=$(dirname "$file")
    base=$(basename "$file")
    new_base="${base/_sui.rs/_rtd.rs}"
    if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
        echo "  $file -> $dir/$new_base"
        mv "$file" "$dir/$new_base"
    fi
done

# 4. 检查是否还有遗漏
echo ""
echo "Phase 4: 检查遗漏..."

# 检查包含 sui 的 .rs 文件（排除 testsuite 等通用文件）
remaining=$(find . -name "*sui*.rs" ! -path "./.git/*" ! -path "./target/*" ! -name "*testsuite*" 2>/dev/null | wc -l)
if [ "$remaining" -gt 0 ]; then
    echo "还有 $remaining 个包含 sui 的 .rs 文件（排除 testsuite）:"
    find . -name "*sui*.rs" ! -path "./.git/*" ! -path "./target/*" ! -name "*testsuite*" 2>/dev/null
else
    echo "✅ 所有包含 sui 的 .rs 文件已处理（排除 testsuite）"
fi

echo "========================================"
echo "补丁脚本执行完成！"
echo "========================================"
