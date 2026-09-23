#!/bin/bash
set -e

################################################################################
# RTD 品牌重命名补丁脚本 v2 - Patch 1
#
# 修复遗漏的目录重命名
################################################################################

# 修复 macOS sed 的 "illegal byte sequence" 错误
export LC_ALL=C
export LANG=C

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

cd "$PROJECT_ROOT"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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
echo "  RTD 品牌重命名补丁脚本 v2 - Patch 1"
echo "  修复遗漏的目录重命名"
echo "========================================"
echo ""

################################################################################
# Phase 1: 重命名嵌套的子目录（从深到浅）
################################################################################
log_info "Phase 1: 重命名嵌套的子目录..."

# 1.1 crates/sui-framework 内的子目录（先处理最深的）
log_info "1.1 处理 crates/sui-framework 内的子目录..."

# packages/sui-framework
if [ -d "crates/sui-framework/packages/sui-framework" ]; then
    log_info "  重命名 packages/sui-framework -> packages/rtd-framework"
    mv "crates/sui-framework/packages/sui-framework" "crates/sui-framework/packages/rtd-framework"
fi

# packages/sui-system
if [ -d "crates/sui-framework/packages/sui-system" ]; then
    log_info "  重命名 packages/sui-system -> packages/rtd-system"
    mv "crates/sui-framework/packages/sui-system" "crates/sui-framework/packages/rtd-system"
fi

# docs/sui
if [ -d "crates/sui-framework/docs/sui" ]; then
    log_info "  重命名 docs/sui -> docs/rtd"
    mv "crates/sui-framework/docs/sui" "crates/sui-framework/docs/rtd"
fi

# docs/sui_system
if [ -d "crates/sui-framework/docs/sui_system" ]; then
    log_info "  重命名 docs/sui_system -> docs/rtd_system"
    mv "crates/sui-framework/docs/sui_system" "crates/sui-framework/docs/rtd_system"
fi

# 1.2 其他 crate 内的子目录
log_info "1.2 处理其他 crate 内的子目录..."

# rtd-adapter-transactional-tests/tests/sui
if [ -d "crates/rtd-adapter-transactional-tests/tests/sui" ]; then
    log_info "  重命名 rtd-adapter-transactional-tests/tests/sui -> tests/rtd"
    mv "crates/rtd-adapter-transactional-tests/tests/sui" "crates/rtd-adapter-transactional-tests/tests/rtd"
fi

# rtd-package-resolver/tests/packages/sui
if [ -d "crates/rtd-package-resolver/tests/packages/sui" ]; then
    log_info "  重命名 rtd-package-resolver/tests/packages/sui -> packages/rtd"
    mv "crates/rtd-package-resolver/tests/packages/sui" "crates/rtd-package-resolver/tests/packages/rtd"
fi

################################################################################
# Phase 2: 重命名顶级 crates 目录
################################################################################
log_info "Phase 2: 重命名顶级 crates 目录..."

# 定义需要重命名的 crate 目录
declare -a CRATES_TO_RENAME=(
    "sui-framework:rtd-framework"
    "sui-light-client:rtd-light-client"
    "sui-source-validation:rtd-source-validation"
    "sui-core:rtd-core"
    "sui-types:rtd-types"
    "sui-single-node-benchmark:rtd-single-node-benchmark"
)

for entry in "${CRATES_TO_RENAME[@]}"; do
    old_name="${entry%%:*}"
    new_name="${entry##*:}"

    if [ -d "crates/$old_name" ]; then
        if [ ! -d "crates/$new_name" ]; then
            log_info "  crates/$old_name -> crates/$new_name"
            mv "crates/$old_name" "crates/$new_name"
        else
            log_warn "  跳过: crates/$new_name 已存在"
        fi
    fi
done

################################################################################
# Phase 3: 查找并重命名所有剩余的 sui-* 目录
################################################################################
log_info "Phase 3: 查找并重命名所有剩余的 sui-* 目录..."

# 使用 find 查找所有 sui-* 目录（从深到浅排序）
find ./crates -type d -name "sui-*" 2>/dev/null | sort -r | while read -r dir; do
    parent_dir=$(dirname "$dir")
    base=$(basename "$dir")
    new_base="${base/sui-/rtd-}"

    if [ "$base" != "$new_base" ] && [ ! -d "$parent_dir/$new_base" ]; then
        log_info "  $dir -> $parent_dir/$new_base"
        mv "$dir" "$parent_dir/$new_base"
    fi
done

################################################################################
# Phase 4: 查找并重命名所有剩余的 sui_* 目录
################################################################################
log_info "Phase 4: 查找并重命名所有剩余的 sui_* 目录..."

find ./crates -type d -name "sui_*" 2>/dev/null | sort -r | while read -r dir; do
    parent_dir=$(dirname "$dir")
    base=$(basename "$dir")
    new_base="${base/sui_/rtd_}"

    if [ "$base" != "$new_base" ] && [ ! -d "$parent_dir/$new_base" ]; then
        log_info "  $dir -> $parent_dir/$new_base"
        mv "$dir" "$parent_dir/$new_base"
    fi
done

################################################################################
# Phase 5: 查找并重命名所有名为 "sui" 的目录
################################################################################
log_info "Phase 5: 查找并重命名所有名为 'sui' 的目录..."

find ./crates -type d -name "sui" 2>/dev/null | sort -r | while read -r dir; do
    parent_dir=$(dirname "$dir")

    if [ ! -d "$parent_dir/rtd" ]; then
        log_info "  $dir -> $parent_dir/rtd"
        mv "$dir" "$parent_dir/rtd"
    fi
done

################################################################################
# Phase 6: 验证
################################################################################
log_info "Phase 6: 验证..."

echo ""
echo "=== 检查剩余的 sui 相关目录 ==="
remaining_dirs=$(find . -type d \( -name "sui-*" -o -name "sui_*" -o -name "sui" \) \
    ! -path "./.git/*" \
    ! -path "./target/*" \
    ! -path "./node_modules/*" \
    ! -path "./fork-instruct/*" \
    2>/dev/null | wc -l | tr -d ' ')

if [ "$remaining_dirs" -gt 0 ]; then
    log_warn "还有 $remaining_dirs 个目录未处理:"
    find . -type d \( -name "sui-*" -o -name "sui_*" -o -name "sui" \) \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./node_modules/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null
else
    log_success "所有 sui 相关目录已处理完成"
fi

echo ""
echo "========================================"
log_success "补丁脚本执行完成!"
echo "========================================"
