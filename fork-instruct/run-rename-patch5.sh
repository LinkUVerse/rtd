#!/bin/bash
set -e

# 修复 macOS sed 的 "illegal byte sequence" 错误
export LC_ALL=C
export LANG=C

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "========================================"
echo "RTD 品牌重命名补丁脚本 (Patch 5 - 更新版)"
echo "修复所有包含 sui 的 .rs 文件名"
echo "========================================"

# 1. 重命名所有包含 sui 的 .rs 文件
echo "Phase 1: 重命名包含 sui 的 .rs 文件..."

# 先处理 sui_ 开头的文件
find . -name "sui_*.rs" ! -path "./.git/*" ! -path "./target/*" 2>/dev/null | while read -r file; do
    dir=$(dirname "$file")
    base=$(basename "$file")
    new_base="${base/sui_/rtd_}"
    if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
        echo "  $file -> $dir/$new_base"
        mv "$file" "$dir/$new_base"
    fi
done

# 处理包含 _sui_ 的文件（如 epoch_start_sui_system_state.rs）
find . -name "*_sui_*.rs" ! -path "./.git/*" ! -path "./target/*" 2>/dev/null | while read -r file; do
    dir=$(dirname "$file")
    base=$(basename "$file")
    new_base="${base/_sui_/_rtd_}"
    if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
        echo "  $file -> $dir/$new_base"
        mv "$file" "$dir/$new_base"
    fi
done

# 处理 suins -> rtdns 的文件
find . -name "suins*.rs" ! -path "./.git/*" ! -path "./target/*" 2>/dev/null | while read -r file; do
    dir=$(dirname "$file")
    base=$(basename "$file")
    new_base="${base/suins/rtdns}"
    if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
        echo "  $file -> $dir/$new_base"
        mv "$file" "$dir/$new_base"
    fi
done

# 处理 pay_sui.rs 等文件
find . -name "*_sui.rs" ! -path "./.git/*" ! -path "./target/*" 2>/dev/null | while read -r file; do
    dir=$(dirname "$file")
    base=$(basename "$file")
    new_base="${base/_sui.rs/_rtd.rs}"
    if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
        echo "  $file -> $dir/$new_base"
        mv "$file" "$dir/$new_base"
    fi
done

# 2. 修复 move-compiler editions/mod.rs 中的常量冲突（如果尚未修复）
echo ""
echo "Phase 2: 检查并修复 move-compiler 常量冲突..."

EDITIONS_FILE="external-crates/move/crates/move-compiler/src/editions/mod.rs"
if [ -f "$EDITIONS_FILE" ]; then
    if grep -q 'pub const RTD: &'"'"'static str' "$EDITIONS_FILE"; then
        # 将常量 RTD 改为 RTD_FLAVOR 以避免与枚举变体冲突
        sed -i '' 's/pub const RTD: &'"'"'static str = "rtd";/pub const RTD_FLAVOR: \&'"'"'static str = "rtd";/g' "$EDITIONS_FILE"
        # 更新 match 语句中的引用
        sed -i '' 's/Self::RTD => Self::RTD/Self::RTD_FLAVOR => Self::RTD/g' "$EDITIONS_FILE"
        echo "  已修复: $EDITIONS_FILE"
    else
        echo "  已经修复过: $EDITIONS_FILE"
    fi
else
    echo "  警告: $EDITIONS_FILE 不存在"
fi

# 3. 检查是否还有遗漏
echo ""
echo "Phase 3: 检查遗漏..."

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
