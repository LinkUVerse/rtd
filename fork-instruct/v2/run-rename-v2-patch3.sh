#!/bin/bash
set -e

################################################################################
# RTD 品牌重命名补丁脚本 v2 - Patch 3
#
# 修复 move-compiler 中的常量命名问题
#
# 问题：常量被错误命名为 RTD_FLAVOR，但应该是 RTD
# 原因：原版代码中常量名是 SUI，按照 SUI -> RTD 规则应该是 RTD
#
# 原版代码结构：
#   pub const SUI: &'static str = "sui";   // 常量
#   Self::SUI => Self::Sui,                // 常量 -> 枚举变体
#
# 正确的替换后：
#   pub const RTD: &'static str = "rtd";   // 常量
#   Self::RTD => Self::Rtd,                // 常量 -> 枚举变体
################################################################################

export LC_ALL=C
export LANG=C

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

cd "$PROJECT_ROOT"

# 颜色输出
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

echo "========================================"
echo "  RTD 品牌重命名补丁脚本 v2 - Patch 3"
echo "  修复 move-compiler 常量命名问题"
echo "========================================"
echo ""

################################################################################
# 修复 editions/mod.rs 中的常量问题
################################################################################
EDITIONS_FILE="external-crates/move/crates/move-compiler/src/editions/mod.rs"

if [ -f "$EDITIONS_FILE" ]; then
    log_info "修复 $EDITIONS_FILE..."

    # 将错误的 RTD_FLAVOR 改回正确的 RTD
    if grep -q 'pub const RTD_FLAVOR:' "$EDITIONS_FILE"; then
        log_info "  将 RTD_FLAVOR 改为 RTD..."
        sed -i '' 's/pub const RTD_FLAVOR:/pub const RTD:/g' "$EDITIONS_FILE"
    fi

    # 确保所有引用都使用 RTD（而不是 RTD_FLAVOR）
    if grep -q 'Self::RTD_FLAVOR' "$EDITIONS_FILE"; then
        log_info "  更新 Self::RTD_FLAVOR 引用为 Self::RTD..."
        sed -i '' 's/Self::RTD_FLAVOR/Self::RTD/g' "$EDITIONS_FILE"
    fi

    echo ""
    log_info "验证修复后的代码:"
    echo ""
    echo "--- 常量定义 ---"
    grep -n 'pub const' "$EDITIONS_FILE" | grep -E 'CORE|RTD'
    echo ""
    echo "--- FromStr 实现 ---"
    grep -n 'Self::CORE\|Self::RTD' "$EDITIONS_FILE" | head -5
    echo ""
    echo "--- Display 实现 ---"
    grep -n 'Flavor::Core\|Flavor::Rtd' "$EDITIONS_FILE" | grep 'write!'

    log_success "$EDITIONS_FILE 修复完成"
else
    log_info "警告: $EDITIONS_FILE 不存在"
fi

echo ""
echo "========================================"
log_success "补丁脚本执行完成!"
echo "========================================"
echo ""
echo "请重新运行编译验证:"
echo "  cargo build -p move-compiler"
echo ""
