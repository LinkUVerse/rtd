#!/bin/bash
set -e

################################################################################
# RTD 品牌重命名补丁脚本 v2 - Patch 7
#
# 替换 *.sql 文件中的品牌标识
#
# 替换规则：
# | 原始        | 替换为       | 说明                     |
# |------------|-------------|--------------------------|
# | MystenLabs | LinkUVerse  | 组织名称                 |
# | Mysten     | LinkU       | 品牌名称（首字母大写）     |
# | mysten     | linku       | 品牌名称（纯小写）        |
# | SUI        | RTD         | 纯大写                   |
# | Sui        | Rtd         | 混合大小写               |
# | sui        | rtd         | 纯小写                   |
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
echo "  RTD 品牌重命名补丁脚本 v2 - Patch 7"
echo "  替换 *.sql 文件中的品牌标识"
echo "========================================"
echo ""

# 统计函数
count_files() {
    local pattern="$1"
    find . -type f -name "$pattern" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./node_modules/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null | wc -l | tr -d ' '
}

# 替换函数
do_replace() {
    local file_pattern="$1"
    local search="$2"
    local replace="$3"

    find . -type f -name "$file_pattern" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./node_modules/*" \
        ! -path "./fork-instruct/*" \
        -exec sed -i '' "s/${search}/${replace}/g" {} + 2>/dev/null || true
}

################################################################################
# Phase 1: 统计文件数量
################################################################################
log_info "Phase 1: 统计文件数量..."

SQL_COUNT=$(count_files "*.sql")

echo "  *.sql 文件数量: $SQL_COUNT"
echo ""

################################################################################
# Phase 2: 替换 *.sql 文件
################################################################################
log_info "Phase 2: 替换 *.sql 文件..."

if [ "$SQL_COUNT" -gt 0 ]; then
    # 按长度从长到短替换
    log_info "  替换 MystenLabs -> LinkUVerse..."
    do_replace "*.sql" "MystenLabs" "LinkUVerse"

    log_info "  替换 Mysten -> LinkU..."
    do_replace "*.sql" "Mysten" "LinkU"

    log_info "  替换 mysten -> linku..."
    do_replace "*.sql" "mysten" "linku"

    log_info "  替换 SUI -> RTD..."
    do_replace "*.sql" "SUI" "RTD"

    log_info "  替换 Sui -> Rtd..."
    do_replace "*.sql" "Sui" "Rtd"

    log_info "  替换 sui -> rtd..."
    do_replace "*.sql" "sui" "rtd"

    log_success "*.sql 文件替换完成"
else
    log_warn "没有找到 *.sql 文件"
fi

################################################################################
# Phase 3: 验证
################################################################################
log_info "Phase 3: 验证替换结果..."

echo ""
echo "=== *.sql 文件中的残留检查 ==="
sql_sui=$(grep -r "sui\|Sui\|SUI" . --include="*.sql" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | grep -v "rtd\|Rtd\|RTD" | head -5 || true)
if [ -n "$sql_sui" ]; then
    echo "$sql_sui"
    log_warn "*.sql 文件中可能有残留"
else
    log_success "*.sql 文件检查通过"
fi

echo ""
echo "========================================"
log_success "补丁脚本执行完成!"
echo "========================================"
