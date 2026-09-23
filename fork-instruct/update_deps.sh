#!/bin/bash
# Sui 依赖 URL 替换脚本
# 使用方法: ./update_deps.sh YOUR_ORG [SUI_DIR]

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
MY_ORG="${1:-}"
SUI_DIR="${2:-$(dirname "$(dirname "$(realpath "$0")")")}"

if [ -z "$MY_ORG" ]; then
    echo -e "${RED}错误: 请提供你的 GitHub 组织名称${NC}"
    echo "使用方法: ./update_deps.sh YOUR_ORG [SUI_DIR]"
    exit 1
fi

echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}   Sui 依赖 URL 替换脚本${NC}"
echo -e "${BLUE}   目标组织: $MY_ORG${NC}"
echo -e "${BLUE}   Sui 目录: $SUI_DIR${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""

cd "$SUI_DIR"

# 检查是否在正确的目录
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}错误: 未找到 Cargo.toml，请确保在 Sui 项目根目录运行${NC}"
    exit 1
fi

# 定义替换规则（只包含核心仓库，不包含外部依赖）
# 核心仓库：必须 fork 才能编译
declare -a CORE_REPLACEMENTS=(
    "MystenLabs/fastcrypto:$MY_ORG/fastcrypto"
    "MystenLabs/mysten-sim:$MY_ORG/mysten-sim"
    "MystenLabs/sui-rust-sdk:$MY_ORG/sui-rust-sdk"
    "mystenlabs/anemo:$MY_ORG/anemo"
    "amnn/async-graphql:$MY_ORG/async-graphql"
)

# 外部依赖仓库（可选，默认不替换）
# 如果你也 fork 了这些仓库，可以取消注释下面的数组并添加到替换列表
# declare -a EXTERNAL_REPLACEMENTS=(
#     "bmwill/axum-server:$MY_ORG/axum-server"
#     "nextest-rs/datatest-stable:$MY_ORG/datatest-stable"
#     "nextest-rs/nexlint:$MY_ORG/nexlint"
#     "zhiburt/tabled:$MY_ORG/tabled"
#     "asonnino/prometheus-parser:$MY_ORG/prometheus-parser"
#     "andll/tidehunter:$MY_ORG/tidehunter"
#     "sui-foundation/awesome-sui:$MY_ORG/awesome-sui"
# )

# 检测操作系统以使用正确的 sed 语法
if [[ "$OSTYPE" == "darwin"* ]]; then
    SED_INPLACE="sed -i ''"
else
    SED_INPLACE="sed -i"
fi

update_file() {
    local file=$1
    local old=$2
    local new=$3

    if [ -f "$file" ]; then
        if grep -q "github.com/$old" "$file" 2>/dev/null; then
            if [[ "$OSTYPE" == "darwin"* ]]; then
                sed -i '' "s|github.com/$old|github.com/$new|g" "$file"
            else
                sed -i "s|github.com/$old|github.com/$new|g" "$file"
            fi
            return 0
        fi
    fi
    return 1
}

echo -e "${GREEN}=== 更新 Cargo.toml ===${NC}"

for item in "${CORE_REPLACEMENTS[@]}"; do
    old="${item%%:*}"
    new="${item##*:}"
    if update_file "Cargo.toml" "$old" "$new"; then
        echo -e "  ${GREEN}[替换]${NC} $old -> $new"
    fi
done

echo ""
echo -e "${GREEN}=== 更新 crates/typed-store/Cargo.toml ===${NC}"

for item in "${CORE_REPLACEMENTS[@]}"; do
    old="${item%%:*}"
    new="${item##*:}"
    if update_file "crates/typed-store/Cargo.toml" "$old" "$new"; then
        echo -e "  ${GREEN}[替换]${NC} $old -> $new"
    fi
done

echo ""
echo -e "${GREEN}=== 更新 .gitmodules ===${NC}"

for item in "${CORE_REPLACEMENTS[@]}"; do
    old="${item%%:*}"
    new="${item##*:}"
    if update_file ".gitmodules" "$old" "$new"; then
        echo -e "  ${GREEN}[替换]${NC} $old -> $new"
    fi
done

echo ""
echo -e "${YELLOW}=== 同步 Git Submodule ===${NC}"
if [ -f ".gitmodules" ]; then
    git submodule sync 2>/dev/null || echo -e "${YELLOW}警告: submodule sync 失败（可能尚未初始化）${NC}"
fi

echo ""
echo -e "${BLUE}============================================${NC}"
echo -e "${GREEN}依赖更新完成!${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""
echo -e "下一步操作:"
echo ""
echo -e "  1. 清理 Cargo 缓存（推荐）:"
echo -e "     ${YELLOW}rm -rf ~/.cargo/git/checkouts/ ~/.cargo/git/db/${NC}"
echo -e "     ${YELLOW}cargo clean${NC}"
echo ""
echo -e "  2. 更新依赖锁文件:"
echo -e "     ${YELLOW}cargo update${NC}"
echo ""
echo -e "  3. 验证依赖解析:"
echo -e "     ${YELLOW}cargo fetch${NC}"
echo ""
echo -e "  4. 编译项目:"
echo -e "     ${YELLOW}cargo check${NC}"
echo -e "     或: ${YELLOW}cargo build -p sui-node${NC}"
echo ""
echo -e "  5. 提交更改:"
echo -e "     ${YELLOW}git add .${NC}"
echo -e "     ${YELLOW}git commit -m \"chore: update dependencies to use forked repositories\"${NC}"
echo ""
