#!/bin/bash
set -e

# 修复 macOS sed 的 "illegal byte sequence" 错误
export LC_ALL=C
export LANG=C

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "========================================"
echo "RTD 品牌重命名补丁脚本 (Patch 1)"
echo "修复遗漏的子目录重命名"
echo "========================================"

# 1. 重命名 rtd-execution 下的子目录
echo "重命名 rtd-execution 子目录..."

for version in v0 v1 v2 latest; do
    if [ -d "rtd-execution/$version" ]; then
        echo "  处理 rtd-execution/$version ..."

        [ -d "rtd-execution/$version/sui-adapter" ] && \
            mv "rtd-execution/$version/sui-adapter" "rtd-execution/$version/rtd-adapter"

        [ -d "rtd-execution/$version/sui-verifier" ] && \
            mv "rtd-execution/$version/sui-verifier" "rtd-execution/$version/rtd-verifier"

        [ -d "rtd-execution/$version/sui-move-natives" ] && \
            mv "rtd-execution/$version/sui-move-natives" "rtd-execution/$version/rtd-move-natives"
    fi
done

# 2. 重命名 docker 目录下的子目录
echo "重命名 docker 子目录..."

for dir in docker/sui-*; do
    if [ -d "$dir" ]; then
        new_dir="${dir/sui-/rtd-}"
        echo "  $dir -> $new_dir"
        mv "$dir" "$new_dir"
    fi
done

# 3. 重命名 nre/ansible/roles 下的目录
echo "重命名 nre/ansible/roles 子目录..."

[ -d "nre/ansible/roles/sui-node" ] && \
    mv "nre/ansible/roles/sui-node" "nre/ansible/roles/rtd-node"

# 4. 重命名 crates/rtd-rosetta/docker 下的目录
echo "重命名 crates/rtd-rosetta/docker 子目录..."

for dir in crates/rtd-rosetta/docker/sui-*; do
    if [ -d "$dir" ]; then
        new_dir="${dir/sui-/rtd-}"
        echo "  $dir -> $new_dir"
        mv "$dir" "$new_dir"
    fi
done

# 5. 重命名 docs/content 下的目录
echo "重命名 docs/content 子目录..."

[ -d "docs/content/references/sui-api" ] && \
    mv "docs/content/references/sui-api" "docs/content/references/rtd-api"

[ -d "docs/content/guides/developer/sui-101" ] && \
    mv "docs/content/guides/developer/sui-101" "docs/content/guides/developer/rtd-101"

[ -d "docs/content/concepts/sui-architecture" ] && \
    mv "docs/content/concepts/sui-architecture" "docs/content/concepts/rtd-architecture"

[ -d "docs/content/concepts/sui-move-concepts" ] && \
    mv "docs/content/concepts/sui-move-concepts" "docs/content/concepts/rtd-move-concepts"

# 6. 检查是否还有遗漏的 sui-* 或 mysten-* 目录
echo ""
echo "检查是否还有遗漏的目录..."
remaining=$(find . -type d \( -name "sui-*" -o -name "mysten-*" \) ! -path "./.git/*" ! -path "./target/*" 2>/dev/null | wc -l)

if [ "$remaining" -gt 0 ]; then
    echo "警告: 还有 $remaining 个目录未处理:"
    find . -type d \( -name "sui-*" -o -name "mysten-*" \) ! -path "./.git/*" ! -path "./target/*" 2>/dev/null
else
    echo "所有目录已处理完成!"
fi

echo "========================================"
echo "补丁脚本执行完成！"
echo "========================================"
