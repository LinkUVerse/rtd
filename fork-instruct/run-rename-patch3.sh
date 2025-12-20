#!/bin/bash
set -e

# 修复 macOS sed 的 "illegal byte sequence" 错误
export LC_ALL=C
export LANG=C

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "========================================"
echo "RTD 品牌重命名补丁脚本 (Patch 3)"
echo "修复 generated 目录下的文件重命名"
echo "========================================"

# 1. 重命名 generated 目录下的 sui.* 文件
echo "重命名 generated 目录下的 sui.* 文件..."

find . -path "*/generated/*" -name "sui.*" ! -path "./.git/*" ! -path "./target/*" 2>/dev/null | while read -r file; do
    dir=$(dirname "$file")
    base=$(basename "$file")
    new_base="${base/sui./rtd.}"
    if [ "$base" != "$new_base" ]; then
        echo "  $file -> $dir/$new_base"
        mv "$file" "$dir/$new_base"
    fi
done

# 2. 检查是否还有遗漏的 sui.* 文件
echo ""
echo "检查是否还有遗漏的 sui.* 文件..."
remaining=$(find . -name "sui.*" ! -path "./.git/*" ! -path "./target/*" -type f 2>/dev/null | wc -l)

if [ "$remaining" -gt 0 ]; then
    echo "警告: 还有 $remaining 个文件未处理:"
    find . -name "sui.*" ! -path "./.git/*" ! -path "./target/*" -type f 2>/dev/null
else
    echo "所有 sui.* 文件已处理完成!"
fi

# 3. 检查是否有其他 sui 前缀的目录
echo ""
echo "检查是否有遗漏的 sui* 目录..."
remaining_dirs=$(find . -type d -name "sui*" ! -path "./.git/*" ! -path "./target/*" 2>/dev/null | wc -l)

if [ "$remaining_dirs" -gt 0 ]; then
    echo "警告: 还有 $remaining_dirs 个目录未处理:"
    find . -type d -name "sui*" ! -path "./.git/*" ! -path "./target/*" 2>/dev/null
else
    echo "所有 sui* 目录已处理完成!"
fi

echo "========================================"
echo "补丁脚本执行完成！"
echo "========================================"
