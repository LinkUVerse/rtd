#!/bin/bash
set -e

################################################################################
# RTD 品牌重命名补丁脚本 v2 - Patch 2
#
# 修复重复目录问题：同时存在 sui-* 和空的 rtd-* 目录
# 策略：删除空的 rtd-* 目录，然后重命名 sui-* 为 rtd-*
################################################################################

export LC_ALL=C
export LANG=C

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

cd "$PROJECT_ROOT"

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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
echo "  RTD 品牌重命名补丁脚本 v2 - Patch 2"
echo "  修复重复目录问题"
echo "========================================"
echo ""

# 检查目录是否为空（只包含 . 和 ..）
is_dir_empty() {
    local dir="$1"
    if [ -d "$dir" ]; then
        local count=$(ls -A "$dir" 2>/dev/null | wc -l | tr -d ' ')
        [ "$count" -eq 0 ]
    else
        return 1
    fi
}

################################################################################
# Phase 1: 处理 crates 目录下的重复
################################################################################
log_info "Phase 1: 处理 crates 目录下的重复..."

# 定义需要处理的目录对
declare -a DUPLICATE_DIRS=(
    "sui-framework:rtd-framework"
    "sui-light-client:rtd-light-client"
    "sui-source-validation:rtd-source-validation"
    "sui-core:rtd-core"
    "sui-types:rtd-types"
    "sui-single-node-benchmark:rtd-single-node-benchmark"
)

for entry in "${DUPLICATE_DIRS[@]}"; do
    sui_name="${entry%%:*}"
    rtd_name="${entry##*:}"

    sui_path="crates/$sui_name"
    rtd_path="crates/$rtd_name"

    if [ -d "$sui_path" ]; then
        if [ -d "$rtd_path" ]; then
            # 两个都存在，检查 rtd 是否为空
            if is_dir_empty "$rtd_path"; then
                log_info "  删除空目录: $rtd_path"
                rm -rf "$rtd_path"
                log_info "  重命名: $sui_path -> $rtd_path"
                mv "$sui_path" "$rtd_path"
            else
                # rtd 不为空，需要合并
                log_warn "  $rtd_path 不为空，尝试合并..."
                # 将 sui 的内容移动到 rtd
                cp -R "$sui_path"/* "$rtd_path"/ 2>/dev/null || true
                rm -rf "$sui_path"
                log_info "  已合并并删除: $sui_path"
            fi
        else
            # 只有 sui 存在，直接重命名
            log_info "  重命名: $sui_path -> $rtd_path"
            mv "$sui_path" "$rtd_path"
        fi
    fi
done

################################################################################
# Phase 2: 递归处理所有嵌套的重复目录
################################################################################
log_info "Phase 2: 递归处理所有嵌套目录..."

# 从深到浅处理所有 sui-* 目录
find ./crates -type d -name "sui-*" 2>/dev/null | sort -r | while read -r sui_dir; do
    parent_dir=$(dirname "$sui_dir")
    sui_base=$(basename "$sui_dir")
    rtd_base="${sui_base/sui-/rtd-}"
    rtd_dir="$parent_dir/$rtd_base"

    if [ -d "$rtd_dir" ]; then
        if is_dir_empty "$rtd_dir"; then
            log_info "  删除空目录: $rtd_dir"
            rm -rf "$rtd_dir"
        else
            log_warn "  合并: $sui_dir -> $rtd_dir"
            cp -R "$sui_dir"/* "$rtd_dir"/ 2>/dev/null || true
            rm -rf "$sui_dir"
            continue
        fi
    fi

    log_info "  重命名: $sui_dir -> $rtd_dir"
    mv "$sui_dir" "$rtd_dir"
done

# 处理 sui_* 目录
find ./crates -type d -name "sui_*" 2>/dev/null | sort -r | while read -r sui_dir; do
    parent_dir=$(dirname "$sui_dir")
    sui_base=$(basename "$sui_dir")
    rtd_base="${sui_base/sui_/rtd_}"
    rtd_dir="$parent_dir/$rtd_base"

    if [ -d "$rtd_dir" ]; then
        if is_dir_empty "$rtd_dir"; then
            rm -rf "$rtd_dir"
        else
            cp -R "$sui_dir"/* "$rtd_dir"/ 2>/dev/null || true
            rm -rf "$sui_dir"
            continue
        fi
    fi

    log_info "  重命名: $sui_dir -> $rtd_dir"
    mv "$sui_dir" "$rtd_dir"
done

# 处理名为 "sui" 的目录
find ./crates -type d -name "sui" 2>/dev/null | sort -r | while read -r sui_dir; do
    parent_dir=$(dirname "$sui_dir")
    rtd_dir="$parent_dir/rtd"

    if [ -d "$rtd_dir" ]; then
        if is_dir_empty "$rtd_dir"; then
            rm -rf "$rtd_dir"
        else
            cp -R "$sui_dir"/* "$rtd_dir"/ 2>/dev/null || true
            rm -rf "$sui_dir"
            continue
        fi
    fi

    log_info "  重命名: $sui_dir -> $rtd_dir"
    mv "$sui_dir" "$rtd_dir"
done

################################################################################
# Phase 3: 处理其他位置的目录
################################################################################
log_info "Phase 3: 处理其他位置的目录..."

# 处理整个项目中的 sui 相关目录
find . -type d \( -name "sui-*" -o -name "sui_*" -o -name "sui" \) \
    ! -path "./.git/*" \
    ! -path "./target/*" \
    ! -path "./node_modules/*" \
    ! -path "./fork-instruct/*" \
    2>/dev/null | sort -r | while read -r sui_dir; do

    parent_dir=$(dirname "$sui_dir")
    sui_base=$(basename "$sui_dir")

    # 确定新名称
    if [ "$sui_base" = "sui" ]; then
        rtd_base="rtd"
    elif [[ "$sui_base" == sui-* ]]; then
        rtd_base="${sui_base/sui-/rtd-}"
    elif [[ "$sui_base" == sui_* ]]; then
        rtd_base="${sui_base/sui_/rtd_}"
    else
        continue
    fi

    rtd_dir="$parent_dir/$rtd_base"

    if [ -d "$rtd_dir" ]; then
        if is_dir_empty "$rtd_dir"; then
            rm -rf "$rtd_dir"
        else
            cp -R "$sui_dir"/* "$rtd_dir"/ 2>/dev/null || true
            rm -rf "$sui_dir"
            continue
        fi
    fi

    if [ -d "$sui_dir" ]; then
        log_info "  重命名: $sui_dir -> $rtd_dir"
        mv "$sui_dir" "$rtd_dir"
    fi
done

################################################################################
# Phase 4: 验证
################################################################################
log_info "Phase 4: 验证..."

echo ""
echo "=== 检查剩余的 sui 相关目录 ==="
remaining=$(find . -type d \( -name "sui-*" -o -name "sui_*" -o -name "sui" \) \
    ! -path "./.git/*" \
    ! -path "./target/*" \
    ! -path "./node_modules/*" \
    ! -path "./fork-instruct/*" \
    2>/dev/null)

if [ -n "$remaining" ]; then
    echo "$remaining" | while read -r dir; do
        log_warn "未处理: $dir"
    done
    echo ""
    remaining_count=$(echo "$remaining" | wc -l | tr -d ' ')
    log_warn "还有 $remaining_count 个目录未处理"
else
    log_success "所有 sui 相关目录已处理完成"
fi

echo ""
echo "=== 当前 crates 目录状态 ==="
ls -la crates/ | grep -E "^d" | grep -E "(sui|rtd)" || echo "无相关目录"

echo ""
echo "========================================"
log_success "补丁脚本执行完成!"
echo "========================================"
