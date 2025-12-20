#!/bin/bash
# Sui 区块链完整 Fork 脚本
# 使用方法:
#   交互模式: ./fork_all.sh
#   个人账户: ./fork_all.sh --user
#   组织账户: ./fork_all.sh --org YOUR_ORG
#
# 功能特性:
#   - 支持个人账户和组织账户两种模式
#   - 自动跳过已存在的仓库，继续 fork 剩余仓库
#   - 显示详细的 fork 统计信息

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 解析参数
USE_ORG=false
MY_ORG=""
INTERACTIVE=true

while [[ $# -gt 0 ]]; do
    case $1 in
        --org)
            USE_ORG=true
            MY_ORG="$2"
            INTERACTIVE=false
            shift 2
            ;;
        --user)
            USE_ORG=false
            INTERACTIVE=false
            shift
            ;;
        -h|--help)
            echo "Sui 区块链完整 Fork 脚本"
            echo ""
            echo "使用方法:"
            echo "  ./fork_all.sh              交互模式，会询问账户类型"
            echo "  ./fork_all.sh --user       直接使用个人账户 fork"
            echo "  ./fork_all.sh --org NAME   直接使用组织账户 fork"
            echo ""
            echo "选项:"
            echo "  --user       使用个人账户 fork"
            echo "  --org NAME   使用指定的组织账户 fork"
            echo "  -h, --help   显示此帮助信息"
            exit 0
            ;;
        *)
            # 兼容旧的使用方式：直接传组织名
            if [ -z "$MY_ORG" ]; then
                MY_ORG="$1"
                USE_ORG=true
                INTERACTIVE=false
            fi
            shift
            ;;
    esac
done

echo -e "${BLUE}============================================${NC}"
echo -e "${BLUE}   Sui 区块链完整 Fork 脚本${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""

# 检查 gh CLI 是否安装
if ! command -v gh &> /dev/null; then
    echo -e "${RED}错误: GitHub CLI (gh) 未安装${NC}"
    echo "请先安装: brew install gh (macOS) 或访问 https://cli.github.com/"
    exit 1
fi

# 检查是否已登录
if ! gh auth status &> /dev/null; then
    echo -e "${YELLOW}请先登录 GitHub CLI...${NC}"
    gh auth login
fi

# 获取当前用户名
CURRENT_USER=$(gh api user -q .login)
echo -e "当前登录用户: ${GREEN}$CURRENT_USER${NC}"
echo ""

# 交互式选择账户类型
if [ "$INTERACTIVE" = true ]; then
    echo -e "${CYAN}请选择 Fork 目标账户类型:${NC}"
    echo ""
    echo "  1) 个人账户 - Fork 到你的个人账户 ($CURRENT_USER)"
    echo "  2) 组织账户 - Fork 到一个 GitHub 组织"
    echo ""
    read -p "请输入选项 (1 或 2): " -n 1 -r ACCOUNT_CHOICE
    echo ""
    echo ""

    case $ACCOUNT_CHOICE in
        1)
            USE_ORG=false
            ;;
        2)
            USE_ORG=true
            # 获取用户所属的组织列表
            echo -e "${CYAN}正在获取你所属的组织列表...${NC}"
            ORGS=$(gh api user/orgs -q '.[].login' 2>/dev/null || echo "")

            if [ -n "$ORGS" ]; then
                echo ""
                echo -e "${CYAN}你所属的组织:${NC}"
                i=1
                declare -a ORG_ARRAY
                while IFS= read -r org; do
                    echo "  $i) $org"
                    ORG_ARRAY[$i]="$org"
                    ((i++))
                done <<< "$ORGS"
                echo "  $i) 手动输入组织名称"
                echo ""
                read -p "请选择组织 (输入数字): " ORG_CHOICE

                if [ "$ORG_CHOICE" -eq "$i" ] 2>/dev/null; then
                    read -p "请输入组织名称: " MY_ORG
                elif [ "$ORG_CHOICE" -ge 1 ] && [ "$ORG_CHOICE" -lt "$i" ] 2>/dev/null; then
                    MY_ORG="${ORG_ARRAY[$ORG_CHOICE]}"
                else
                    echo -e "${RED}无效选择${NC}"
                    exit 1
                fi
            else
                echo -e "${YELLOW}未找到你所属的组织，请手动输入组织名称${NC}"
                read -p "请输入组织名称: " MY_ORG
            fi

            if [ -z "$MY_ORG" ]; then
                echo -e "${RED}错误: 组织名称不能为空${NC}"
                exit 1
            fi
            ;;
        *)
            echo -e "${RED}无效选择，退出${NC}"
            exit 1
            ;;
    esac
fi

# 设置目标
if [ "$USE_ORG" = true ]; then
    if [ -z "$MY_ORG" ]; then
        echo -e "${RED}错误: 使用组织账户时必须提供组织名称${NC}"
        echo "使用方法: ./fork_all.sh --org YOUR_ORG"
        exit 1
    fi
    TARGET="$MY_ORG"
    echo -e "${BLUE}模式: 组织账户${NC}"
else
    TARGET="$CURRENT_USER"
    echo -e "${BLUE}模式: 个人账户${NC}"
fi

echo -e "Fork 目标: ${GREEN}$TARGET${NC}"
echo ""

# 核心仓库（必须 fork）
CORE_REPOS=(
    "MystenLabs/sui"
    "MystenLabs/fastcrypto"
    "MystenLabs/mysten-sim"
    "MystenLabs/sui-rust-sdk"
    "mystenlabs/anemo"
    "amnn/async-graphql"
)

# 外部依赖仓库（可选 fork）
EXTERNAL_REPOS=(
    "bmwill/axum-server"
    "nextest-rs/datatest-stable"
    "nextest-rs/nexlint"
    "zhiburt/tabled"
    "asonnino/prometheus-parser"
    "andll/tidehunter"
    "sui-foundation/awesome-sui"
)

# 统计变量
TOTAL_NEW=0
TOTAL_SKIPPED=0
TOTAL_FAILED=0

fork_repo() {
    local repo=$1
    local name=$(basename $repo)
    echo -ne "  ${YELLOW}[$name]${NC} "

    # 检查是否已存在
    if gh repo view "$TARGET/$name" &> /dev/null; then
        echo -e "${CYAN}[已存在 - 跳过]${NC}"
        TOTAL_SKIPPED=$((TOTAL_SKIPPED + 1))
        return 0
    fi

    # 执行 fork
    echo -ne "正在 fork..."
    if [ "$USE_ORG" = true ]; then
        if gh repo fork "$repo" --org "$MY_ORG" --clone=false 2>/dev/null; then
            echo -e " ${GREEN}[新建成功]${NC}"
            TOTAL_NEW=$((TOTAL_NEW + 1))
        else
            echo -e " ${RED}[失败]${NC}"
            TOTAL_FAILED=$((TOTAL_FAILED + 1))
            return 1
        fi
    else
        if gh repo fork "$repo" --clone=false 2>/dev/null; then
            echo -e " ${GREEN}[新建成功]${NC}"
            TOTAL_NEW=$((TOTAL_NEW + 1))
        else
            echo -e " ${RED}[失败]${NC}"
            TOTAL_FAILED=$((TOTAL_FAILED + 1))
            return 1
        fi
    fi
}

echo -e "${GREEN}=== 第一步: Fork 核心仓库 (必须) ===${NC}"
echo -e "${CYAN}共 ${#CORE_REPOS[@]} 个核心仓库${NC}"
echo ""

CORE_NEW=0
CORE_SKIPPED=0
CORE_FAILED=0

for repo in "${CORE_REPOS[@]}"; do
    BEFORE_NEW=$TOTAL_NEW
    BEFORE_SKIPPED=$TOTAL_SKIPPED
    BEFORE_FAILED=$TOTAL_FAILED

    fork_repo "$repo"

    if [ $TOTAL_NEW -gt $BEFORE_NEW ]; then
        CORE_NEW=$((CORE_NEW + 1))
    elif [ $TOTAL_SKIPPED -gt $BEFORE_SKIPPED ]; then
        CORE_SKIPPED=$((CORE_SKIPPED + 1))
    else
        CORE_FAILED=$((CORE_FAILED + 1))
    fi
done

echo ""
echo -e "核心仓库统计: ${GREEN}新建 $CORE_NEW${NC} | ${CYAN}跳过 $CORE_SKIPPED${NC} | ${RED}失败 $CORE_FAILED${NC}"
echo ""

# 询问是否 fork 外部依赖
echo -e "${YELLOW}是否要 fork 外部依赖仓库? (推荐用于完全控制)${NC}"
echo -e "${CYAN}外部依赖共 ${#EXTERNAL_REPOS[@]} 个:${NC}"
echo "  axum-server, datatest-stable, nexlint, tabled,"
echo "  prometheus-parser, tidehunter, awesome-sui"
echo ""
read -p "输入 y/n: " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo -e "${GREEN}=== 第二步: Fork 外部依赖仓库 ===${NC}"
    echo ""

    EXT_NEW=0
    EXT_SKIPPED=0
    EXT_FAILED=0

    for repo in "${EXTERNAL_REPOS[@]}"; do
        BEFORE_NEW=$TOTAL_NEW
        BEFORE_SKIPPED=$TOTAL_SKIPPED
        BEFORE_FAILED=$TOTAL_FAILED

        fork_repo "$repo"

        if [ $TOTAL_NEW -gt $BEFORE_NEW ]; then
            EXT_NEW=$((EXT_NEW + 1))
        elif [ $TOTAL_SKIPPED -gt $BEFORE_SKIPPED ]; then
            EXT_SKIPPED=$((EXT_SKIPPED + 1))
        else
            EXT_FAILED=$((EXT_FAILED + 1))
        fi
    done

    echo ""
    echo -e "外部依赖统计: ${GREEN}新建 $EXT_NEW${NC} | ${CYAN}跳过 $EXT_SKIPPED${NC} | ${RED}失败 $EXT_FAILED${NC}"
fi

echo ""
echo -e "${BLUE}============================================${NC}"
echo -e "${GREEN}Fork 完成!${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""
echo -e "总计统计:"
echo -e "  ${GREEN}新建: $TOTAL_NEW 个仓库${NC}"
echo -e "  ${CYAN}跳过: $TOTAL_SKIPPED 个仓库 (已存在)${NC}"
echo -e "  ${RED}失败: $TOTAL_FAILED 个仓库${NC}"
echo ""

if [ $TOTAL_FAILED -gt 0 ]; then
    echo -e "${YELLOW}提示: 有 $TOTAL_FAILED 个仓库 fork 失败，你可以重新运行此脚本继续 fork${NC}"
    echo ""
fi

echo -e "下一步操作:"
echo -e "  1. 克隆你的 sui fork:"
echo -e "     ${YELLOW}git clone https://github.com/$TARGET/sui.git${NC}"
echo ""
echo -e "  2. 进入目录并运行依赖更新脚本:"
echo -e "     ${YELLOW}cd sui${NC}"
echo -e "     ${YELLOW}./fork-instruct/update_deps.sh $TARGET${NC}"
echo ""
echo -e "  3. 验证编译:"
echo -e "     ${YELLOW}cargo check${NC}"
echo ""
