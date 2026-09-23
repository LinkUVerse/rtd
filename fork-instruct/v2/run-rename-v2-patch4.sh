#!/bin/bash
set -e

################################################################################
# RTD 品牌重命名补丁脚本 v2 - Patch 4
#
# 修复 rtd-bridge crate 中的 ABI 文件命名问题
#
# 问题：代码中引用 "abi/rtd_bridge.json"，但实际文件名仍为 "sui_bridge.json"
################################################################################

export LC_ALL=C
export LANG=C

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

cd "$PROJECT_ROOT"

# 颜色输出
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

echo "========================================"
echo "  RTD 品牌重命名补丁脚本 v2 - Patch 4"
echo "  修复 rtd-bridge ABI 文件命名问题"
echo "========================================"
echo ""

################################################################################
# Phase 1: 重命名 ABI 文件
################################################################################
log_info "Phase 1: 重命名 ABI 文件..."

ABI_DIR="crates/rtd-bridge/abi"

if [ -d "$ABI_DIR" ]; then
    # 重命名 sui_bridge.json -> rtd_bridge.json
    if [ -f "$ABI_DIR/sui_bridge.json" ]; then
        log_info "  重命名 sui_bridge.json -> rtd_bridge.json"
        mv "$ABI_DIR/sui_bridge.json" "$ABI_DIR/rtd_bridge.json"
    fi

    # 检查 tests 目录下是否有需要重命名的文件
    if [ -d "$ABI_DIR/tests" ]; then
        for f in "$ABI_DIR/tests"/sui_*; do
            if [ -f "$f" ]; then
                base=$(basename "$f")
                new_base="${base/sui_/rtd_}"
                log_info "  重命名 tests/$base -> tests/$new_base"
                mv "$f" "$ABI_DIR/tests/$new_base"
            fi
        done
    fi

    echo ""
    log_info "当前 ABI 目录内容:"
    ls -la "$ABI_DIR"
else
    log_warn "$ABI_DIR 目录不存在"
fi

################################################################################
# Phase 2: 查找并重命名其他 sui_* 文件
################################################################################
log_info "Phase 2: 查找并重命名其他 sui_* 文件..."

# 查找所有名为 sui_*.json 的文件并重命名
find . -type f -name "sui_*.json" \
    ! -path "./.git/*" \
    ! -path "./target/*" \
    ! -path "./fork-instruct/*" \
    2>/dev/null | while read -r file; do
    dir=$(dirname "$file")
    base=$(basename "$file")
    new_base="${base/sui_/rtd_}"
    if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
        log_info "  $file -> $dir/$new_base"
        mv "$file" "$dir/$new_base"
    fi
done

################################################################################
# Phase 3: 验证
################################################################################
log_info "Phase 3: 验证..."

echo ""
echo "=== 检查剩余的 sui_*.json 文件 ==="
remaining=$(find . -type f -name "sui_*.json" \
    ! -path "./.git/*" \
    ! -path "./target/*" \
    ! -path "./fork-instruct/*" \
    2>/dev/null)

if [ -n "$remaining" ]; then
    echo "$remaining"
    log_warn "还有文件未处理"
else
    log_success "所有 sui_*.json 文件已处理完成"
fi

echo ""
echo "========================================"
log_success "补丁脚本执行完成!"
echo "========================================"
echo ""
echo "请重新运行编译验证:"
echo "  cargo build -p rtd-bridge"
echo ""
