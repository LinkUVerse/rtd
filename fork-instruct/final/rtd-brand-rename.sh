#!/bin/bash
set -e

################################################################################
# RTD 品牌重命名一键式脚本（终极整合版）
#
# 整合了 v2 主脚本和所有补丁脚本的功能
#
# 用法：
#   ./rtd-brand-rename.sh all           # 完整执行所有阶段
#   ./rtd-brand-rename.sh phase01       # 单独执行某个阶段
#   ./rtd-brand-rename.sh --config xxx  # 使用自定义配置
#   ./rtd-brand-rename.sh --dry-run     # 预览模式（不执行）
#   ./rtd-brand-rename.sh --help        # 显示帮助
#
# 版本：v1.0
# 日期：2024-12-21
################################################################################

# 修复 macOS sed 的 "illegal byte sequence" 错误
export LC_ALL=C
export LANG=C

################################################################################
# 默认品牌配置（可通过 --config 参数覆盖）
################################################################################
OLD_ORG="MystenLabs"
NEW_ORG="LinkUVerse"

OLD_BRAND="Mysten"
NEW_BRAND="LinkU"

OLD_BRAND_LOWER="mysten"
NEW_BRAND_LOWER="linku"

OLD_UPPER="SUI"
NEW_UPPER="RTD"

OLD_MIXED="Sui"
NEW_MIXED="Rtd"

OLD_LOWER="sui"
NEW_LOWER="rtd"

# SDK 依赖配置
OLD_SDK_REV="339c2272fd5b8fb4e1fa6662cfa9acdbb0d05704"
NEW_SDK_REV="2fff36e9d4b7fbad1b7e44a1b9aefbd3f042d126"

################################################################################
# 全局变量
################################################################################
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
DRY_RUN=false
CONFIG_FILE=""

################################################################################
# 颜色输出函数
################################################################################
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
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

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_phase() {
    echo ""
    echo "========================================"
    echo -e "${GREEN}$1${NC}"
    echo "========================================"
}

log_step() {
    echo -e "${CYAN}  → $1${NC}"
}

################################################################################
# 工具函数
################################################################################

# 检查目录是否为空
is_dir_empty() {
    local dir="$1"
    if [ -d "$dir" ]; then
        local count=$(ls -A "$dir" 2>/dev/null | wc -l | tr -d ' ')
        [ "$count" -eq 0 ]
    else
        return 1
    fi
}

# 安全重命名目录（处理空目录冲突）
safe_rename_dir() {
    local src="$1"
    local dst="$2"

    if [ ! -d "$src" ]; then
        return 0
    fi

    if [ -d "$dst" ]; then
        if is_dir_empty "$dst"; then
            log_step "删除空目录: $dst"
            [ "$DRY_RUN" = false ] && rmdir "$dst"
        else
            log_warn "目标目录已存在且非空: $dst"
            return 1
        fi
    fi

    log_step "$src -> $dst"
    [ "$DRY_RUN" = false ] && mv "$src" "$dst"
}

# 通用的文件查找和替换函数
do_sed_replace() {
    local pattern="$1"
    local replacement="$2"

    log_step "替换 $pattern -> $replacement"

    if [ "$DRY_RUN" = true ]; then
        return 0
    fi

    # 处理各种文本文件（整合了所有补丁的文件类型）
    find . -type f \( \
        -name "*.rs" -o \
        -name "*.toml" -o \
        -name "*.md" -o \
        -name "*.mdx" -o \
        -name "*.yaml" -o \
        -name "*.yml" -o \
        -name "*.json" -o \
        -name "*.move" -o \
        -name "*.sh" -o \
        -name "*.txt" -o \
        -name "*.graphql" -o \
        -name "*.proto" -o \
        -name "*.nuspec" -o \
        -name "*.ros" -o \
        -name "*.keystore" -o \
        -name "Makefile" -o \
        -name "Dockerfile*" -o \
        -name ".env*" -o \
        -name "*.ts" -o \
        -name "*.js" -o \
        -name "*.tsx" -o \
        -name "*.jsx" -o \
        -name "*.html" -o \
        -name "*.css" -o \
        -name "*.scss" -o \
        -name "*.lock" -o \
        -name "*.service" -o \
        -name "*.conf" -o \
        -name "*.cfg" -o \
        -name "*.ini" -o \
        -name "*.env" -o \
        -name "*.exp" -o \
        -name "*.snap" -o \
        -name "*.py" -o \
        -name "*.ptb" -o \
        -name "*.sql" \
    \) \
    ! -path "./.git/*" \
    ! -path "./target/*" \
    ! -path "./node_modules/*" \
    ! -path "./fork-instruct/*" \
    -exec sed -i '' "s/${pattern}/${replacement}/g" {} + 2>/dev/null || true
}

################################################################################
# Phase 00: 准备工作
################################################################################
phase_00_prepare() {
    log_phase "Phase 00: 准备工作"

    log_info "创建备份分支和标签..."

    if [ "$DRY_RUN" = false ]; then
        git checkout -b feature/${NEW_LOWER}-brand-rename-v2 2>/dev/null || {
            log_warn "分支已存在，切换到该分支"
            git checkout feature/${NEW_LOWER}-brand-rename-v2 2>/dev/null || true
        }
        git tag -f pre-rename-backup-v2
    fi

    log_success "备份完成: 标签 pre-rename-backup-v2"
}

################################################################################
# Phase 01: 文本内容替换
################################################################################
phase_01_text_replace() {
    log_phase "Phase 01: 文本内容替换"

    # 按长度从长到短的顺序替换，避免部分匹配问题
    do_sed_replace "$OLD_ORG" "$NEW_ORG"
    do_sed_replace "$OLD_BRAND" "$NEW_BRAND"
    do_sed_replace "$OLD_BRAND_LOWER" "$NEW_BRAND_LOWER"
    do_sed_replace "$OLD_UPPER" "$NEW_UPPER"
    do_sed_replace "$OLD_MIXED" "$NEW_MIXED"
    do_sed_replace "$OLD_LOWER" "$NEW_LOWER"

    log_success "文本内容替换完成"
}

################################################################################
# Phase 02: 更新 SDK 依赖
################################################################################
phase_02_update_sdk() {
    log_phase "Phase 02: 更新 SDK 依赖"

    log_step "更新 Cargo.toml 中的 rev 版本号..."

    if [ "$DRY_RUN" = false ]; then
        find . -name "Cargo.toml" \
            ! -path "./.git/*" \
            ! -path "./target/*" \
            ! -path "./fork-instruct/*" \
            -exec sed -i '' "s|$OLD_SDK_REV|$NEW_SDK_REV|g" {} + 2>/dev/null || true
    fi

    log_success "SDK 依赖更新完成"
}

################################################################################
# Phase 03: 重命名 crates 目录（整合 patch1/2）
################################################################################
phase_03_rename_crates() {
    log_phase "Phase 03: 重命名 crates 目录"

    # 3.1 首先处理嵌套的子目录（从深到浅）
    log_info "3.1 处理嵌套子目录..."

    # 处理 crates/${OLD_LOWER}-framework 内的子目录
    if [ -d "crates/${OLD_LOWER}-framework" ]; then
        for subdir in "crates/${OLD_LOWER}-framework/packages/${OLD_LOWER}-framework" \
                      "crates/${OLD_LOWER}-framework/packages/${OLD_LOWER}-system"; do
            if [ -d "$subdir" ]; then
                new_subdir="${subdir//${OLD_LOWER}-/${NEW_LOWER}-}"
                safe_rename_dir "$subdir" "$new_subdir"
            fi
        done
    fi

    # 3.2 重命名 sui-* -> rtd-*
    log_info "3.2 重命名 crates/${OLD_LOWER}-* -> crates/${NEW_LOWER}-*..."
    for dir in crates/${OLD_LOWER}-*; do
        if [ -d "$dir" ]; then
            new_dir="${dir/${OLD_LOWER}-/${NEW_LOWER}-}"
            safe_rename_dir "$dir" "$new_dir"
        fi
    done

    # 3.3 重命名 mysten-* -> linku-*
    log_info "3.3 重命名 crates/${OLD_BRAND_LOWER}-* -> crates/${NEW_BRAND_LOWER}-*..."
    for dir in crates/${OLD_BRAND_LOWER}-*; do
        if [ -d "$dir" ]; then
            new_dir="${dir/${OLD_BRAND_LOWER}-/${NEW_BRAND_LOWER}-}"
            safe_rename_dir "$dir" "$new_dir"
        fi
    done

    # 3.4 重命名 crates/sui -> crates/rtd (不带连字符)
    log_info "3.4 重命名 crates/${OLD_LOWER} -> crates/${NEW_LOWER}..."
    if [ -d "crates/${OLD_LOWER}" ]; then
        safe_rename_dir "crates/${OLD_LOWER}" "crates/${NEW_LOWER}"
    fi

    # 3.5 重命名 suins-indexer -> rtdns-indexer
    log_info "3.5 重命名 crates/${OLD_LOWER}ns-indexer -> crates/${NEW_LOWER}ns-indexer..."
    if [ -d "crates/${OLD_LOWER}ns-indexer" ]; then
        safe_rename_dir "crates/${OLD_LOWER}ns-indexer" "crates/${NEW_LOWER}ns-indexer"
    fi

    log_success "crates 目录重命名完成"
}

################################################################################
# Phase 04: 重命名 sui-execution 目录
################################################################################
phase_04_rename_execution() {
    log_phase "Phase 04: 重命名 ${OLD_LOWER}-execution 目录"

    # 4.1 重命名顶级目录
    log_info "4.1 重命名 ${OLD_LOWER}-execution -> ${NEW_LOWER}-execution..."
    if [ -d "${OLD_LOWER}-execution" ]; then
        safe_rename_dir "${OLD_LOWER}-execution" "${NEW_LOWER}-execution"
    fi

    # 4.2 重命名子目录
    log_info "4.2 重命名 ${NEW_LOWER}-execution 子目录..."
    for version in v0 v1 v2 latest; do
        if [ -d "${NEW_LOWER}-execution/$version" ]; then
            log_step "处理 ${NEW_LOWER}-execution/$version..."

            for subdir in "${OLD_LOWER}-adapter" "${OLD_LOWER}-verifier" "${OLD_LOWER}-move-natives"; do
                if [ -d "${NEW_LOWER}-execution/$version/$subdir" ]; then
                    new_subdir="${subdir/${OLD_LOWER}-/${NEW_LOWER}-}"
                    safe_rename_dir "${NEW_LOWER}-execution/$version/$subdir" "${NEW_LOWER}-execution/$version/$new_subdir"
                fi
            done
        fi
    done

    log_success "${OLD_LOWER}-execution 目录重命名完成"
}

################################################################################
# Phase 05: 重命名框架目录
################################################################################
phase_05_rename_framework() {
    log_phase "Phase 05: 重命名框架目录"

    FRAMEWORK_DIR="crates/${NEW_LOWER}-framework"

    # 5.1 重命名 packages 子目录
    log_info "5.1 重命名框架 packages 子目录..."
    if [ -d "$FRAMEWORK_DIR/packages/${OLD_LOWER}-framework" ]; then
        safe_rename_dir "$FRAMEWORK_DIR/packages/${OLD_LOWER}-framework" "$FRAMEWORK_DIR/packages/${NEW_LOWER}-framework"
    fi
    if [ -d "$FRAMEWORK_DIR/packages/${OLD_LOWER}-system" ]; then
        safe_rename_dir "$FRAMEWORK_DIR/packages/${OLD_LOWER}-system" "$FRAMEWORK_DIR/packages/${NEW_LOWER}-system"
    fi

    # 5.2 重命名 packages_compiled 文件
    log_info "5.2 重命名 packages_compiled 文件..."
    COMPILED_DIR="$FRAMEWORK_DIR/packages_compiled"
    if [ -d "$COMPILED_DIR" ]; then
        if [ -f "$COMPILED_DIR/${OLD_LOWER}-framework" ]; then
            log_step "${OLD_LOWER}-framework -> ${NEW_LOWER}-framework"
            [ "$DRY_RUN" = false ] && mv "$COMPILED_DIR/${OLD_LOWER}-framework" "$COMPILED_DIR/${NEW_LOWER}-framework"
        fi
        if [ -f "$COMPILED_DIR/${OLD_LOWER}-system" ]; then
            log_step "${OLD_LOWER}-system -> ${NEW_LOWER}-system"
            [ "$DRY_RUN" = false ] && mv "$COMPILED_DIR/${OLD_LOWER}-system" "$COMPILED_DIR/${NEW_LOWER}-system"
        fi
    fi

    # 5.3 重命名 docs 子目录
    log_info "5.3 重命名框架 docs 子目录..."
    if [ -d "$FRAMEWORK_DIR/docs/${OLD_LOWER}" ]; then
        safe_rename_dir "$FRAMEWORK_DIR/docs/${OLD_LOWER}" "$FRAMEWORK_DIR/docs/${NEW_LOWER}"
    fi
    if [ -d "$FRAMEWORK_DIR/docs/${OLD_LOWER}_system" ]; then
        safe_rename_dir "$FRAMEWORK_DIR/docs/${OLD_LOWER}_system" "$FRAMEWORK_DIR/docs/${NEW_LOWER}_system"
    fi

    log_success "框架目录重命名完成"
}

################################################################################
# Phase 06: 重命名 docker 目录
################################################################################
phase_06_rename_docker() {
    log_phase "Phase 06: 重命名 docker 目录"

    log_info "6.1 重命名 docker/${OLD_LOWER}-* 目录..."
    for dir in docker/${OLD_LOWER}-*; do
        if [ -d "$dir" ]; then
            new_dir="${dir/${OLD_LOWER}-/${NEW_LOWER}-}"
            safe_rename_dir "$dir" "$new_dir"
        fi
    done

    log_info "6.2 重命名 crates/${NEW_LOWER}-rosetta/docker 子目录..."
    for dir in crates/${NEW_LOWER}-rosetta/docker/${OLD_LOWER}-*; do
        if [ -d "$dir" ]; then
            new_dir="${dir/${OLD_LOWER}-/${NEW_LOWER}-}"
            safe_rename_dir "$dir" "$new_dir"
        fi
    done

    log_success "docker 目录重命名完成"
}

################################################################################
# Phase 07: 重命名 nre/ansible 目录
################################################################################
phase_07_rename_ansible() {
    log_phase "Phase 07: 重命名 nre/ansible 目录"

    log_info "重命名 nre/ansible/roles/${OLD_LOWER}-node..."
    if [ -d "nre/ansible/roles/${OLD_LOWER}-node" ]; then
        safe_rename_dir "nre/ansible/roles/${OLD_LOWER}-node" "nre/ansible/roles/${NEW_LOWER}-node"
    fi

    log_success "nre/ansible 目录重命名完成"
}

################################################################################
# Phase 08: 重命名 docs/content 目录
################################################################################
phase_08_rename_docs() {
    log_phase "Phase 08: 重命名 docs/content 目录"

    declare -a DOCS_DIRS=(
        "docs/content/references/${OLD_LOWER}-api:${NEW_LOWER}-api"
        "docs/content/guides/developer/${OLD_LOWER}-101:${NEW_LOWER}-101"
        "docs/content/concepts/${OLD_LOWER}-architecture:${NEW_LOWER}-architecture"
        "docs/content/concepts/${OLD_LOWER}-move-concepts:${NEW_LOWER}-move-concepts"
        "docs/content/guides/${OLD_LOWER}play0x1:${NEW_LOWER}play0x1"
    )

    for entry in "${DOCS_DIRS[@]}"; do
        old_path="${entry%%:*}"
        new_name="${entry##*:}"

        if [ -d "$old_path" ]; then
            parent_dir=$(dirname "$old_path")
            safe_rename_dir "$old_path" "$parent_dir/$new_name"
        fi
    done

    log_success "docs/content 目录重命名完成"
}

################################################################################
# Phase 09: 重命名 external-crates/move 目录
################################################################################
phase_09_rename_move() {
    log_phase "Phase 09: 重命名 external-crates/move 目录"

    declare -a MOVE_DIRS=(
        "external-crates/move/crates/move-stackless-bytecode-2/tests/move/${OLD_LOWER}_move_2024:${NEW_LOWER}_move_2024"
        "external-crates/move/crates/move-compiler/tests/${OLD_LOWER}_mode:${NEW_LOWER}_mode"
        "external-crates/move/crates/move-compiler/tests/move_2024/${OLD_LOWER}_mode:${NEW_LOWER}_mode"
        "external-crates/move/crates/move-compiler/src/${OLD_LOWER}_mode:${NEW_LOWER}_mode"
    )

    for entry in "${MOVE_DIRS[@]}"; do
        old_path="${entry%%:*}"
        new_name="${entry##*:}"

        if [ -d "$old_path" ]; then
            parent_dir=$(dirname "$old_path")
            safe_rename_dir "$old_path" "$parent_dir/$new_name"
        fi
    done

    log_success "external-crates/move 目录重命名完成"
}

################################################################################
# Phase 10: 重命名其他 sui* 目录
################################################################################
phase_10_rename_other() {
    log_phase "Phase 10: 重命名其他 ${OLD_LOWER}* 目录"

    # 10.1 查找并重命名所有 sui_* 目录
    log_info "10.1 查找并重命名所有 ${OLD_LOWER}_* 目录..."
    find . -type d -name "${OLD_LOWER}_*" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null | sort -r | while read -r dir; do
        parent_dir=$(dirname "$dir")
        base=$(basename "$dir")
        new_base="${base/${OLD_LOWER}_/${NEW_LOWER}_}"
        if [ "$base" != "$new_base" ] && [ ! -d "$parent_dir/$new_base" ]; then
            safe_rename_dir "$dir" "$parent_dir/$new_base"
        fi
    done

    # 10.2 查找并重命名所有 suins* 目录
    log_info "10.2 查找并重命名所有 ${OLD_LOWER}ns* 目录..."
    find . -type d -name "${OLD_LOWER}ns*" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null | sort -r | while read -r dir; do
        parent_dir=$(dirname "$dir")
        base=$(basename "$dir")
        new_base="${base/${OLD_LOWER}ns/${NEW_LOWER}ns}"
        if [ "$base" != "$new_base" ] && [ ! -d "$parent_dir/$new_base" ]; then
            safe_rename_dir "$dir" "$parent_dir/$new_base"
        fi
    done

    # 10.3 特定目录重命名
    log_info "10.3 特定目录重命名..."
    declare -a OTHER_DIRS=(
        "crates/${NEW_LOWER}-rpc-api/proto/${OLD_LOWER}:${NEW_LOWER}"
        "crates/${NEW_LOWER}-indexer-alt-consistent-api/proto/${OLD_LOWER}:${NEW_LOWER}"
    )

    for entry in "${OTHER_DIRS[@]}"; do
        old_path="${entry%%:*}"
        new_name="${entry##*:}"

        if [ -d "$old_path" ]; then
            parent_dir=$(dirname "$old_path")
            safe_rename_dir "$old_path" "$parent_dir/$new_name"
        fi
    done

    log_success "其他目录重命名完成"
}

################################################################################
# Phase 11: 重命名文件（整合 patch4 的 ABI 文件）
################################################################################
phase_11_rename_files() {
    log_phase "Phase 11: 重命名文件"

    # 11.1 重命名 sui.* 文件
    log_info "11.1 重命名 ${OLD_LOWER}.* 文件..."
    find . -type f -name "${OLD_LOWER}.*" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        ! -name "*.pdf" \
        2>/dev/null | while read -r file; do
        dir=$(dirname "$file")
        base=$(basename "$file")
        new_base="${base/${OLD_LOWER}./${NEW_LOWER}.}"
        if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
            log_step "$file -> $dir/$new_base"
            [ "$DRY_RUN" = false ] && mv "$file" "$dir/$new_base"
        fi
    done

    # 11.2 重命名 sui_*.rs 文件
    log_info "11.2 重命名 ${OLD_LOWER}_*.rs 文件..."
    find . -type f -name "${OLD_LOWER}_*.rs" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null | while read -r file; do
        dir=$(dirname "$file")
        base=$(basename "$file")
        new_base="${base/${OLD_LOWER}_/${NEW_LOWER}_}"
        if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
            log_step "$file -> $dir/$new_base"
            [ "$DRY_RUN" = false ] && mv "$file" "$dir/$new_base"
        fi
    done

    # 11.3 重命名 *_sui_*.rs 文件
    log_info "11.3 重命名 *_${OLD_LOWER}_*.rs 文件..."
    find . -type f -name "*_${OLD_LOWER}_*.rs" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null | while read -r file; do
        dir=$(dirname "$file")
        base=$(basename "$file")
        new_base="${base/_${OLD_LOWER}_/_${NEW_LOWER}_}"
        if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
            log_step "$file -> $dir/$new_base"
            [ "$DRY_RUN" = false ] && mv "$file" "$dir/$new_base"
        fi
    done

    # 11.4 重命名 *_sui.rs 文件
    log_info "11.4 重命名 *_${OLD_LOWER}.rs 文件..."
    find . -type f -name "*_${OLD_LOWER}.rs" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null | while read -r file; do
        dir=$(dirname "$file")
        base=$(basename "$file")
        new_base="${base/_${OLD_LOWER}.rs/_${NEW_LOWER}.rs}"
        if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
            log_step "$file -> $dir/$new_base"
            [ "$DRY_RUN" = false ] && mv "$file" "$dir/$new_base"
        fi
    done

    # 11.5 重命名 suins*.rs 文件
    log_info "11.5 重命名 ${OLD_LOWER}ns*.rs 文件..."
    find . -type f -name "${OLD_LOWER}ns*.rs" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null | while read -r file; do
        dir=$(dirname "$file")
        base=$(basename "$file")
        new_base="${base/${OLD_LOWER}ns/${NEW_LOWER}ns}"
        if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
            log_step "$file -> $dir/$new_base"
            [ "$DRY_RUN" = false ] && mv "$file" "$dir/$new_base"
        fi
    done

    # 11.6 重命名 sui.move 文件
    log_info "11.6 重命名 ${OLD_LOWER}.move 文件..."
    find . -type f -name "${OLD_LOWER}.move" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null | while read -r file; do
        dir=$(dirname "$file")
        if [ ! -f "$dir/${NEW_LOWER}.move" ]; then
            log_step "$file -> $dir/${NEW_LOWER}.move"
            [ "$DRY_RUN" = false ] && mv "$file" "$dir/${NEW_LOWER}.move"
        fi
    done

    # 11.7 重命名 sui-* 文件
    log_info "11.7 重命名 ${OLD_LOWER}-* 文件..."
    find . -type f -name "${OLD_LOWER}-*" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null | while read -r file; do
        dir=$(dirname "$file")
        base=$(basename "$file")
        new_base="${base/${OLD_LOWER}-/${NEW_LOWER}-}"
        if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
            log_step "$file -> $dir/$new_base"
            [ "$DRY_RUN" = false ] && mv "$file" "$dir/$new_base"
        fi
    done

    # 11.8 重命名 ABI 文件（patch4）
    log_info "11.8 重命名 ABI 文件..."
    ABI_DIR="crates/${NEW_LOWER}-bridge/abi"
    if [ -d "$ABI_DIR" ]; then
        for file in "$ABI_DIR"/${OLD_LOWER}_*.json; do
            if [ -f "$file" ]; then
                base=$(basename "$file")
                new_base="${base/${OLD_LOWER}_/${NEW_LOWER}_}"
                if [ "$base" != "$new_base" ] && [ ! -f "$ABI_DIR/$new_base" ]; then
                    log_step "$file -> $ABI_DIR/$new_base"
                    [ "$DRY_RUN" = false ] && mv "$file" "$ABI_DIR/$new_base"
                fi
            fi
        done
    fi

    log_success "文件重命名完成"
}

################################################################################
# Phase 12: 修复特殊情况（整合 patch3）
################################################################################
phase_12_fix_special() {
    log_phase "Phase 12: 修复特殊情况"

    # 12.1 修复 move-compiler editions/mod.rs 中的常量
    log_info "12.1 检查并修复 move-compiler 常量..."
    EDITIONS_FILE="external-crates/move/crates/move-compiler/src/editions/mod.rs"

    if [ -f "$EDITIONS_FILE" ] && [ "$DRY_RUN" = false ]; then
        # 检查是否有 RTD_FLAVOR 需要改回 RTD
        if grep -q "RTD_FLAVOR" "$EDITIONS_FILE" 2>/dev/null; then
            log_step "修复常量: RTD_FLAVOR -> RTD"
            sed -i '' 's/RTD_FLAVOR/RTD/g' "$EDITIONS_FILE"
        fi

        # 确保常量和枚举变体正确
        # 常量应该是: pub const RTD: &'static str = "rtd";
        # 枚举变体应该是: Rtd
    fi

    log_success "特殊情况修复完成"
}

################################################################################
# Phase 13: 重编译 Move 字节码（新增）
################################################################################
phase_13_recompile_move() {
    log_phase "Phase 13: 重编译 Move 字节码"

    if [ "$DRY_RUN" = true ]; then
        log_info "[DRY-RUN] 将执行: UPDATE=1 cargo test -p ${NEW_LOWER}-framework --test build-system-packages"
        log_info "[DRY-RUN] 将执行: rm -rf crates/${NEW_LOWER}-framework-snapshot/bytecode_snapshot/*"
        return 0
    fi

    # 13.1 重新编译 packages_compiled
    log_info "13.1 重新编译 packages_compiled..."
    log_warn "这可能需要几分钟时间..."

    if UPDATE=1 cargo test -p ${NEW_LOWER}-framework --test build-system-packages 2>&1; then
        log_success "packages_compiled 重新编译成功"
    else
        log_error "packages_compiled 重新编译失败"
        log_warn "请手动运行: UPDATE=1 cargo test -p ${NEW_LOWER}-framework --test build-system-packages"
    fi

    # 13.2 删除旧的 bytecode_snapshot
    log_info "13.2 删除旧的 bytecode_snapshot..."
    SNAPSHOT_DIR="crates/${NEW_LOWER}-framework-snapshot/bytecode_snapshot"
    if [ -d "$SNAPSHOT_DIR" ]; then
        rm -rf "$SNAPSHOT_DIR"/*
        log_success "bytecode_snapshot 已清空"
    fi

    # 13.3 验证
    log_info "13.3 验证重编译结果..."
    COMPILED_FILE="crates/${NEW_LOWER}-framework/packages_compiled/${NEW_LOWER}-framework"
    if [ -f "$COMPILED_FILE" ]; then
        OLD_REFS=$(strings "$COMPILED_FILE" 2>/dev/null | grep -E "::${OLD_LOWER}::" | wc -l | tr -d ' ')
        NEW_REFS=$(strings "$COMPILED_FILE" 2>/dev/null | grep -E "::${NEW_LOWER}::" | wc -l | tr -d ' ')

        if [ "$OLD_REFS" -gt 0 ]; then
            log_error "仍存在 ${OLD_REFS} 处 ::${OLD_LOWER}:: 引用"
        else
            log_success "无 ::${OLD_LOWER}:: 残留引用"
        fi

        if [ "$NEW_REFS" -gt 0 ]; then
            log_success "包含 ${NEW_REFS} 处 ::${NEW_LOWER}:: 引用"
        fi
    fi

    log_success "Move 字节码重编译完成"
}

################################################################################
# Phase 14: 清理和验证
################################################################################
phase_14_cleanup_verify() {
    log_phase "Phase 14: 清理和验证"

    # 14.1 清理构建缓存
    log_info "14.1 清理构建缓存..."
    if [ "$DRY_RUN" = false ]; then
        find ./target -type d -name "cynic-schemas" 2>/dev/null | while read -r dir; do
            log_step "删除缓存: $dir"
            rm -rf "$dir"
        done
    fi

    # 14.2 检查遗漏的目录
    log_info "14.2 检查遗漏的 ${OLD_LOWER}*/${OLD_BRAND_LOWER}* 目录..."
    echo ""
    echo "=== 遗漏的目录 ==="
    find . -type d \( -name "${OLD_LOWER}-*" -o -name "${OLD_LOWER}_*" -o -name "${OLD_BRAND_LOWER}-*" -o -name "${OLD_BRAND_LOWER}_*" -o -name "${OLD_LOWER}" \) \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./node_modules/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null || echo "  (无遗漏)"

    # 14.3 检查遗漏的文件
    log_info "14.3 检查遗漏的 ${OLD_LOWER}* 文件..."
    echo ""
    echo "=== 遗漏的文件 ==="
    find . -type f \( -name "${OLD_LOWER}_*.rs" -o -name "*_${OLD_LOWER}.rs" -o -name "*_${OLD_LOWER}_*.rs" -o -name "${OLD_LOWER}.*" -o -name "${OLD_LOWER}-*" \) \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./node_modules/*" \
        ! -path "./fork-instruct/*" \
        ! -name "*.pdf" \
        2>/dev/null || echo "  (无遗漏)"

    # 14.4 统计替换结果
    log_info "14.4 统计文本替换结果..."
    echo ""
    echo "=== 新品牌统计 ==="
    echo "  - $NEW_ORG 出现次数: $(grep -r "$NEW_ORG" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | wc -l | tr -d ' ')"
    echo "  - $NEW_BRAND 出现次数: $(grep -r "$NEW_BRAND" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | grep -v "$NEW_ORG" | wc -l | tr -d ' ')"
    echo "  - $NEW_UPPER 出现次数: $(grep -r "$NEW_UPPER" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | wc -l | tr -d ' ')"
    echo "  - $NEW_MIXED 出现次数: $(grep -r "$NEW_MIXED" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | wc -l | tr -d ' ')"
    echo "  - $NEW_LOWER 出现次数: $(grep -r "$NEW_LOWER" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | wc -l | tr -d ' ')"
    echo ""
    echo "=== 残留检查 ==="
    echo "  - $OLD_ORG 残留: $(grep -r "$OLD_ORG" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | wc -l | tr -d ' ')"
    echo "  - $OLD_BRAND 残留: $(grep -r "$OLD_BRAND" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | grep -v "$NEW_BRAND" | wc -l | tr -d ' ')"
    echo "  - $OLD_BRAND_LOWER 残留: $(grep -r "$OLD_BRAND_LOWER" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | grep -v "$NEW_BRAND_LOWER" | wc -l | tr -d ' ')"
    echo "  - $OLD_UPPER 残留: $(grep -rw "$OLD_UPPER" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | grep -v "$NEW_UPPER" | wc -l | tr -d ' ')"
    echo "  - $OLD_MIXED 残留: $(grep -r "$OLD_MIXED" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | grep -v "$NEW_MIXED" | wc -l | tr -d ' ')"
    echo "  - $OLD_LOWER 残留: $(grep -r "$OLD_LOWER" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | grep -v "$NEW_LOWER" | grep -v "suite" | wc -l | tr -d ' ')"

    log_success "清理和验证完成"
}

################################################################################
# 显示帮助
################################################################################
show_help() {
    echo "RTD 品牌重命名一键式脚本"
    echo ""
    echo "用法: $0 [选项] [阶段]"
    echo ""
    echo "选项:"
    echo "  --help, -h        显示帮助信息"
    echo "  --dry-run         预览模式，不执行实际操作"
    echo "  --config FILE     使用自定义配置文件"
    echo ""
    echo "阶段:"
    echo "  all               执行所有阶段（默认）"
    echo "  phase00, prepare  准备工作（创建备份）"
    echo "  phase01, text     文本内容替换"
    echo "  phase02, sdk      更新 SDK 依赖"
    echo "  phase03, crates   重命名 crates 目录"
    echo "  phase04, exec     重命名 execution 目录"
    echo "  phase05, frame    重命名框架目录"
    echo "  phase06, docker   重命名 docker 目录"
    echo "  phase07, ansible  重命名 ansible 目录"
    echo "  phase08, docs     重命名 docs 目录"
    echo "  phase09, move     重命名 move compiler 目录"
    echo "  phase10, other    重命名其他目录"
    echo "  phase11, files    重命名文件"
    echo "  phase12, fix      修复特殊情况"
    echo "  phase13, compile  重编译 Move 字节码"
    echo "  phase14, verify   清理和验证"
    echo ""
    echo "示例:"
    echo "  $0 all                    # 执行所有阶段"
    echo "  $0 phase01                # 只执行文本替换"
    echo "  $0 --dry-run all          # 预览模式"
    echo "  $0 --config my.sh all     # 使用自定义配置"
}

################################################################################
# 主函数
################################################################################
main() {
    echo "========================================"
    echo "  品牌重命名一键式脚本（终极整合版）"
    echo "========================================"
    echo ""
    echo "品牌替换规则:"
    echo "  $OLD_ORG -> $NEW_ORG"
    echo "  $OLD_BRAND -> $NEW_BRAND"
    echo "  $OLD_BRAND_LOWER -> $NEW_BRAND_LOWER"
    echo "  $OLD_UPPER -> $NEW_UPPER"
    echo "  $OLD_MIXED -> $NEW_MIXED"
    echo "  $OLD_LOWER -> $NEW_LOWER"
    echo ""
    echo "项目根目录: $PROJECT_ROOT"

    if [ "$DRY_RUN" = true ]; then
        echo ""
        echo -e "${YELLOW}[DRY-RUN 模式] 不会执行实际操作${NC}"
    fi

    echo ""

    # 询问是否继续
    if [ "$DRY_RUN" = false ]; then
        read -p "是否继续执行? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "用户取消执行"
            exit 0
        fi
    fi

    # 执行各阶段
    phase_00_prepare
    phase_01_text_replace
    phase_02_update_sdk
    phase_03_rename_crates
    phase_04_rename_execution
    phase_05_rename_framework
    phase_06_rename_docker
    phase_07_rename_ansible
    phase_08_rename_docs
    phase_09_rename_move
    phase_10_rename_other
    phase_11_rename_files
    phase_12_fix_special
    phase_13_recompile_move
    phase_14_cleanup_verify

    echo ""
    echo "========================================"
    log_success "品牌重命名脚本执行完成!"
    echo "========================================"
    echo ""
    echo "后续步骤:"
    echo "  1. 运行 'cargo build --workspace' 验证编译"
    echo "  2. 运行 'cargo test' 验证测试"
    echo "  3. 检查并修复任何编译错误"
    echo ""
    echo "回滚方法:"
    echo "  git checkout pre-rename-backup-v2"
    echo "  git reset --hard pre-rename-backup-v2"
    echo ""
}

################################################################################
# 参数解析和入口
################################################################################

# 解析参数
while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            show_help
            exit 0
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --config)
            CONFIG_FILE="$2"
            shift 2
            ;;
        *)
            break
            ;;
    esac
done

# 加载配置文件
if [ -n "$CONFIG_FILE" ] && [ -f "$CONFIG_FILE" ]; then
    log_info "加载配置文件: $CONFIG_FILE"
    source "$CONFIG_FILE"
fi

# 切换到项目根目录
cd "$PROJECT_ROOT"

# 执行指定阶段
case "${1:-all}" in
    phase00|prepare)
        phase_00_prepare
        ;;
    phase01|text)
        phase_01_text_replace
        ;;
    phase02|sdk)
        phase_02_update_sdk
        ;;
    phase03|crates)
        phase_03_rename_crates
        ;;
    phase04|exec|execution)
        phase_04_rename_execution
        ;;
    phase05|frame|framework)
        phase_05_rename_framework
        ;;
    phase06|docker)
        phase_06_rename_docker
        ;;
    phase07|ansible)
        phase_07_rename_ansible
        ;;
    phase08|docs)
        phase_08_rename_docs
        ;;
    phase09|move)
        phase_09_rename_move
        ;;
    phase10|other)
        phase_10_rename_other
        ;;
    phase11|files)
        phase_11_rename_files
        ;;
    phase12|fix)
        phase_12_fix_special
        ;;
    phase13|compile|recompile)
        phase_13_recompile_move
        ;;
    phase14|verify|cleanup)
        phase_14_cleanup_verify
        ;;
    all)
        main
        ;;
    *)
        echo "未知阶段: $1"
        echo ""
        show_help
        exit 1
        ;;
esac
