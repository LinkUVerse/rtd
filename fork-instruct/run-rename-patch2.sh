#!/bin/bash
set -e

# 修复 macOS sed 的 "illegal byte sequence" 错误
export LC_ALL=C
export LANG=C

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "========================================"
echo "RTD 品牌重命名补丁脚本 (Patch 2)"
echo "修复不带连字符的目录重命名"
echo "========================================"

# 1. 重命名 crates/sui -> crates/rtd
echo "重命名 crates/sui -> crates/rtd..."
if [ -d "crates/sui" ]; then
    mv "crates/sui" "crates/rtd"
    echo "  完成: crates/sui -> crates/rtd"
else
    echo "  跳过: crates/sui 不存在"
fi

# 2. 重命名 crates/suins-indexer -> crates/rtdns-indexer
echo "重命名 crates/suins-indexer -> crates/rtdns-indexer..."
if [ -d "crates/suins-indexer" ]; then
    mv "crates/suins-indexer" "crates/rtdns-indexer"
    echo "  完成: crates/suins-indexer -> crates/rtdns-indexer"
else
    echo "  跳过: crates/suins-indexer 不存在"
fi

# 3. 检查是否还有其他不带连字符的 sui 目录
echo ""
echo "检查是否还有遗漏的目录..."
remaining=$(find crates -maxdepth 1 -type d -name "sui*" 2>/dev/null | wc -l)

if [ "$remaining" -gt 0 ]; then
    echo "警告: 还有 $remaining 个 crates 目录未处理:"
    find crates -maxdepth 1 -type d -name "sui*" 2>/dev/null
else
    echo "crates 目录检查完成!"
fi

echo "========================================"
echo "补丁脚本执行完成！"
echo "========================================"
