#!/bin/bash
# Sui Fork 仓库同步脚本
# 将所有 fork 的仓库与上游仓库同步
#
# 使用方法:
#   交互模式: ./sync_forks.sh
#   个人账户: ./sync_forks.sh --user
#   组织账户: ./sync_forks.sh --org YOUR_ORG
#   强制同步: ./sync_forks.sh --force
#
# 功能特性:
#   - 批量同步所有 fork 仓库与上游最新代码
#   - 支持个人账户和组织账户
#   - 可选强制同步（hard reset）
#   - 显示详细的同步统计信息

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
FORCE_SYNC=false

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
        --force|-f)
            FORCE_SYNC=true
            shift
            ;;
        -h|--help)
            echo "Sui Fork 仓库同步脚本"
            echo ""
            echo "使用方法:"
            echo "  ./sync_forks.sh              交互模式，会询问账户类型"
            echo "  ./sync_forks.sh --user       同步个人账户的 fork"
            echo "  ./sync_forks.sh --org NAME   同步组织账户的 fork"
            echo ""
            echo "选项:"
            echo "  --user       使用个人账户"
            echo "  --org NAME   使用指定的组织账户"
            echo "  --force, -f  强制同步（hard reset，会覆盖本地修改）"
            echo "  -h, --help   显示此帮助信息"
            echo ""
            echo "注意:"
            echo "  - 默认使用 fast-forward 方式同步"
            echo "  - 使用 --force 会强制覆盖，请谨慎使用"
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
echo -e "${BLUE}   Sui Fork 仓库同步脚本${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""

if [ "$FORCE_SYNC" = true ]; then
    echo -e "${RED}警告: 强制同步模式已启用！${NC}"
    echo -e "${RED}这将使用 hard reset 覆盖你 fork 仓库中的所有修改！${NC}"
    echo ""
    read -p "确定要继续吗? (输入 yes 确认): " CONFIRM
    if [ "$CONFIRM" != "yes" ]; then
        echo "已取消"
        exit 0
    fi
    echo ""
fi

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
    echo -e "${CYAN}请选择要同步的 Fork 所在账户:${NC}"
    echo ""
    echo "  1) 个人账户 - 同步你个人账户的 fork ($CURRENT_USER)"
    echo "  2) 组织账户 - 同步某个组织的 fork"
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
        echo "使用方法: ./sync_forks.sh --org YOUR_ORG"
        exit 1
    fi
    TARGET="$MY_ORG"
    echo -e "${BLUE}模式: 组织账户${NC}"
else
    TARGET="$CURRENT_USER"
    echo -e "${BLUE}模式: 个人账户${NC}"
fi

echo -e "同步目标: ${GREEN}$TARGET${NC}"
if [ "$FORCE_SYNC" = true ]; then
    echo -e "同步方式: ${RED}强制同步 (hard reset)${NC}"
else
    echo -e "同步方式: ${GREEN}Fast-forward${NC}"
fi
echo ""

# 定义仓库列表（仓库名 -> 上游仓库）
declare -A REPO_MAP=(
    ["sui"]="MystenLabs/sui"
    ["fastcrypto"]="MystenLabs/fastcrypto"
    ["mysten-sim"]="MystenLabs/mysten-sim"
    ["sui-rust-sdk"]="MystenLabs/sui-rust-sdk"
    ["anemo"]="mystenlabs/anemo"
    ["async-graphql"]="amnn/async-graphql"
    ["axum-server"]="bmwill/axum-server"
    ["datatest-stable"]="nextest-rs/datatest-stable"
    ["nexlint"]="nextest-rs/nexlint"
    ["tabled"]="zhiburt/tabled"
    ["prometheus-parser"]="asonnino/prometheus-parser"
    ["tidehunter"]="andll/tidehunter"
    ["awesome-sui"]="sui-foundation/awesome-sui"
)

# 核心仓库列表
CORE_REPOS=("sui" "fastcrypto" "mysten-sim" "sui-rust-sdk" "anemo" "async-graphql")

# 外部依赖仓库列表
EXTERNAL_REPOS=("axum-server" "datatest-stable" "nexlint" "tabled" "prometheus-parser" "tidehunter" "awesome-sui")

# 统计变量
TOTAL_SYNCED=0
TOTAL_SKIPPED=0
TOTAL_FAILED=0
TOTAL_NOT_FOUND=0

sync_repo() {
    local repo_name=$1
    local upstream=${REPO_MAP[$repo_name]}
    local fork_repo="$TARGET/$repo_name"

    echo -ne "  ${YELLOW}[$repo_name]${NC} "

    # 检查 fork 是否存在
    if ! gh repo view "$fork_repo" &> /dev/null; then
        echo -e "${CYAN}[未找到 - 跳过]${NC}"
        TOTAL_NOT_FOUND=$((TOTAL_NOT_FOUND + 1))
        return 0
    fi

    # 执行同步
    echo -ne "正在同步..."

    local SYNC_CMD="gh repo sync $fork_repo"
    if [ "$FORCE_SYNC" = true ]; then
        SYNC_CMD="$SYNC_CMD --force"
    fi

    if $SYNC_CMD 2>/dev/null; then
        echo -e " ${GREEN}[同步成功]${NC}"
        TOTAL_SYNCED=$((TOTAL_SYNCED + 1))
    else
        # 检查是否是因为已经是最新
        local ERROR_OUTPUT=$($SYNC_CMD 2>&1 || true)
        if echo "$ERROR_OUTPUT" | grep -q "already up to date"; then
            echo -e " ${CYAN}[已是最新]${NC}"
            TOTAL_SKIPPED=$((TOTAL_SKIPPED + 1))
        else
            echo -e " ${RED}[同步失败]${NC}"
            echo -e "    ${RED}错误: $ERROR_OUTPUT${NC}"
            TOTAL_FAILED=$((TOTAL_FAILED + 1))
            return 1
        fi
    fi
}

echo -e "${GREEN}=== 第一步: 同步核心仓库 ===${NC}"
echo -e "${CYAN}共 ${#CORE_REPOS[@]} 个核心仓库${NC}"
echo ""

CORE_SYNCED=0
CORE_SKIPPED=0
CORE_FAILED=0
CORE_NOT_FOUND=0

for repo in "${CORE_REPOS[@]}"; do
    BEFORE_SYNCED=$TOTAL_SYNCED
    BEFORE_SKIPPED=$TOTAL_SKIPPED
    BEFORE_FAILED=$TOTAL_FAILED
    BEFORE_NOT_FOUND=$TOTAL_NOT_FOUND

    sync_repo "$repo"

    if [ $TOTAL_SYNCED -gt $BEFORE_SYNCED ]; then
        CORE_SYNCED=$((CORE_SYNCED + 1))
    elif [ $TOTAL_SKIPPED -gt $BEFORE_SKIPPED ]; then
        CORE_SKIPPED=$((CORE_SKIPPED + 1))
    elif [ $TOTAL_NOT_FOUND -gt $BEFORE_NOT_FOUND ]; then
        CORE_NOT_FOUND=$((CORE_NOT_FOUND + 1))
    else
        CORE_FAILED=$((CORE_FAILED + 1))
    fi
done

echo ""
echo -e "核心仓库统计: ${GREEN}同步 $CORE_SYNCED${NC} | ${CYAN}最新 $CORE_SKIPPED${NC} | ${YELLOW}未找到 $CORE_NOT_FOUND${NC} | ${RED}失败 $CORE_FAILED${NC}"
echo ""

# 询问是否同步外部依赖
echo -e "${YELLOW}是否要同步外部依赖仓库?${NC}"
echo -e "${CYAN}外部依赖共 ${#EXTERNAL_REPOS[@]} 个:${NC}"
echo "  axum-server, datatest-stable, nexlint, tabled,"
echo "  prometheus-parser, tidehunter, awesome-sui"
echo ""
read -p "输入 y/n: " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo -e "${GREEN}=== 第二步: 同步外部依赖仓库 ===${NC}"
    echo ""

    EXT_SYNCED=0
    EXT_SKIPPED=0
    EXT_FAILED=0
    EXT_NOT_FOUND=0

    for repo in "${EXTERNAL_REPOS[@]}"; do
        BEFORE_SYNCED=$TOTAL_SYNCED
        BEFORE_SKIPPED=$TOTAL_SKIPPED
        BEFORE_FAILED=$TOTAL_FAILED
        BEFORE_NOT_FOUND=$TOTAL_NOT_FOUND

        sync_repo "$repo"

        if [ $TOTAL_SYNCED -gt $BEFORE_SYNCED ]; then
            EXT_SYNCED=$((EXT_SYNCED + 1))
        elif [ $TOTAL_SKIPPED -gt $BEFORE_SKIPPED ]; then
            EXT_SKIPPED=$((EXT_SKIPPED + 1))
        elif [ $TOTAL_NOT_FOUND -gt $BEFORE_NOT_FOUND ]; then
            EXT_NOT_FOUND=$((EXT_NOT_FOUND + 1))
        else
            EXT_FAILED=$((EXT_FAILED + 1))
        fi
    done

    echo ""
    echo -e "外部依赖统计: ${GREEN}同步 $EXT_SYNCED${NC} | ${CYAN}最新 $EXT_SKIPPED${NC} | ${YELLOW}未找到 $EXT_NOT_FOUND${NC} | ${RED}失败 $EXT_FAILED${NC}"
fi

echo ""
echo -e "${BLUE}============================================${NC}"
echo -e "${GREEN}同步完成!${NC}"
echo -e "${BLUE}============================================${NC}"
echo ""
echo -e "总计统计:"
echo -e "  ${GREEN}已同步: $TOTAL_SYNCED 个仓库${NC}"
echo -e "  ${CYAN}已是最新: $TOTAL_SKIPPED 个仓库${NC}"
echo -e "  ${YELLOW}未找到: $TOTAL_NOT_FOUND 个仓库 (未 fork)${NC}"
echo -e "  ${RED}失败: $TOTAL_FAILED 个仓库${NC}"
echo ""

if [ $TOTAL_FAILED -gt 0 ]; then
    echo -e "${YELLOW}提示: 有 $TOTAL_FAILED 个仓库同步失败${NC}"
    echo -e "${YELLOW}可能原因:${NC}"
    echo -e "  - fork 仓库有本地修改与上游冲突"
    echo -e "  - 使用 --force 参数可强制同步（会覆盖本地修改）"
    echo ""
fi

if [ $TOTAL_NOT_FOUND -gt 0 ]; then
    echo -e "${YELLOW}提示: 有 $TOTAL_NOT_FOUND 个仓库未找到${NC}"
    echo -e "${YELLOW}请先运行 fork_all.sh 脚本 fork 这些仓库${NC}"
    echo ""
fi

echo -e "下一步操作:"
echo -e "  1. 如果你已克隆本地仓库，拉取最新代码:"
echo -e "     ${YELLOW}cd sui && git pull${NC}"
echo ""
echo -e "  2. 更新 Cargo 依赖缓存:"
echo -e "     ${YELLOW}rm -rf ~/.cargo/git/checkouts/ ~/.cargo/git/db/${NC}"
echo -e "     ${YELLOW}cargo update${NC}"
echo ""
