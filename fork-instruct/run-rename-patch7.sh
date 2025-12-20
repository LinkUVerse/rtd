#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "========================================"
echo "RTD 品牌重命名补丁脚本 (Patch 7)"
echo "修复 packages_compiled 目录下的文件"
echo "========================================"

# 1. 重命名 packages_compiled 目录下的文件
echo "重命名 packages_compiled 目录下的文件..."

COMPILED_DIR="crates/rtd-framework/packages_compiled"

if [ -d "$COMPILED_DIR" ]; then
    # 重命名 sui-framework -> rtd-framework
    if [ -f "$COMPILED_DIR/sui-framework" ]; then
        echo "  $COMPILED_DIR/sui-framework -> $COMPILED_DIR/rtd-framework"
        mv "$COMPILED_DIR/sui-framework" "$COMPILED_DIR/rtd-framework"
    fi

    # 重命名 sui-system -> rtd-system
    if [ -f "$COMPILED_DIR/sui-system" ]; then
        echo "  $COMPILED_DIR/sui-system -> $COMPILED_DIR/rtd-system"
        mv "$COMPILED_DIR/sui-system" "$COMPILED_DIR/rtd-system"
    fi

    echo ""
    echo "当前 packages_compiled 目录内容:"
    ls -la "$COMPILED_DIR"
else
    echo "警告: $COMPILED_DIR 目录不存在"
fi

echo "========================================"
echo "补丁脚本执行完成！"
echo "========================================"
