#!/bin/bash
set -e

################################################################################
# RTD 品牌重命名脚本 v2
#
# 用于将 Sui 区块链 fork 为 RTD 区块链的品牌重命名脚本
#
# 替换规则（严格遵循）：
# | 原始        | 替换为       | 说明                     |
# |------------|-------------|--------------------------|
# | MystenLabs | LinkUVerse  | 组织名称                 |
# | Mysten     | LinkU       | 品牌名称（首字母大写）     |
# | mysten     | linku       | 品牌名称（纯小写）        |
# | SUI        | RTD         | 纯大写                   |
# | Sui        | Rtd         | 混合大小写               |
# | sui        | rtd         | 纯小写                   |
#
# sui-rust-sdk 依赖更新：
# - 旧 rev: 339c2272fd5b8fb4e1fa6662cfa9acdbb0d05704
# - 新 rev: 2fff36e9d4b7fbad1b7e44a1b9aefbd3f042d126
# - 仓库: sui-rust-sdk -> rtd-rust-sdk
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

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_phase() {
    echo ""
    echo "========================================"
    echo -e "${GREEN}$1${NC}"
    echo "========================================"
}

# 通用的文件查找和替换函数
do_sed_replace() {
    local pattern="$1"
    local replacement="$2"

    log_info "替换 $pattern -> $replacement..."

    # 处理各种文本文件
    find . -type f \( \
        -name "*.rs" -o \
        -name "*.toml" -o \
        -name "*.md" -o \
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
        -name "*.snap" \
    \) \
    ! -path "./.git/*" \
    ! -path "./target/*" \
    ! -path "./node_modules/*" \
    ! -path "./fork-instruct/*" \
    -exec sed -i '' "s/${pattern}/${replacement}/g" {} + 2>/dev/null || true
}

################################################################################
# Phase 0: 准备工作
################################################################################
phase_0_prepare() {
    log_phase "Phase 0: 准备工作"

    log_info "创建备份分支和标签..."
    git checkout -b feature/rtd-brand-rename-v2 2>/dev/null || {
        log_warn "分支已存在，切换到该分支"
        git checkout feature/rtd-brand-rename-v2 2>/dev/null || true
    }
    git tag -f pre-rename-backup-v2

    log_success "备份完成: 标签 pre-rename-backup-v2"
}

################################################################################
# Phase 1: 文本内容替换（按长度从长到短的顺序）
################################################################################
phase_1_text_replacement() {
    log_phase "Phase 1: 文本内容替换"

    # 1.1 替换 MystenLabs -> LinkUVerse (最长的先替换)
    do_sed_replace "MystenLabs" "LinkUVerse"

    # 1.2 替换 Mysten -> LinkU (首字母大写)
    do_sed_replace "Mysten" "LinkU"

    # 1.3 替换 mysten -> linku (小写)
    do_sed_replace "mysten" "linku"

    # 1.4 替换 SUI -> RTD (大写)
    do_sed_replace "SUI" "RTD"

    # 1.5 替换 Sui -> Rtd (混合大小写)
    do_sed_replace "Sui" "Rtd"

    # 1.6 替换 sui -> rtd (小写)
    do_sed_replace "sui" "rtd"

    log_success "文本内容替换完成"
}

################################################################################
# Phase 2: 更新 sui-rust-sdk 依赖
################################################################################
phase_2_update_sdk_deps() {
    log_phase "Phase 2: 更新 rtd-rust-sdk 依赖"

    OLD_REV="339c2272fd5b8fb4e1fa6662cfa9acdbb0d05704"
    NEW_REV="2fff36e9d4b7fbad1b7e44a1b9aefbd3f042d126"

    log_info "更新 Cargo.toml 中的 rev 版本号..."

    # 替换 rev 版本号 (注意: sui-rust-sdk 在 Phase 1 已被替换为 rtd-rust-sdk)
    find . -name "Cargo.toml" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        -exec sed -i '' "s|$OLD_REV|$NEW_REV|g" {} + 2>/dev/null || true

    log_success "SDK 依赖更新完成"
}

################################################################################
# Phase 3: 重命名 crates 目录
################################################################################
phase_3_rename_crates() {
    log_phase "Phase 3: 重命名 crates 目录"

    # 3.1 重命名 sui-* -> rtd-*
    log_info "3.1 重命名 crates/sui-* -> crates/rtd-*..."
    for dir in crates/sui-*; do
        if [ -d "$dir" ]; then
            new_dir="${dir/sui-/rtd-}"
            if [ ! -d "$new_dir" ]; then
                log_info "  $dir -> $new_dir"
                mv "$dir" "$new_dir"
            fi
        fi
    done

    # 3.2 重命名 mysten-* -> linku-*
    log_info "3.2 重命名 crates/mysten-* -> crates/linku-*..."
    for dir in crates/mysten-*; do
        if [ -d "$dir" ]; then
            new_dir="${dir/mysten-/linku-}"
            if [ ! -d "$new_dir" ]; then
                log_info "  $dir -> $new_dir"
                mv "$dir" "$new_dir"
            fi
        fi
    done

    # 3.3 重命名 crates/sui -> crates/rtd (不带连字符)
    log_info "3.3 重命名 crates/sui -> crates/rtd..."
    if [ -d "crates/sui" ]; then
        if [ ! -d "crates/rtd" ]; then
            mv "crates/sui" "crates/rtd"
            log_info "  crates/sui -> crates/rtd"
        fi
    fi

    # 3.4 重命名 crates/suins-indexer -> crates/rtdns-indexer
    log_info "3.4 重命名 crates/suins-indexer -> crates/rtdns-indexer..."
    if [ -d "crates/suins-indexer" ]; then
        if [ ! -d "crates/rtdns-indexer" ]; then
            mv "crates/suins-indexer" "crates/rtdns-indexer"
            log_info "  crates/suins-indexer -> crates/rtdns-indexer"
        fi
    fi

    log_success "crates 目录重命名完成"
}

################################################################################
# Phase 4: 重命名 sui-execution 目录
################################################################################
phase_4_rename_execution() {
    log_phase "Phase 4: 重命名 sui-execution 目录"

    # 4.1 重命名顶级目录
    log_info "4.1 重命名 sui-execution -> rtd-execution..."
    if [ -d "sui-execution" ]; then
        if [ ! -d "rtd-execution" ]; then
            mv "sui-execution" "rtd-execution"
            log_info "  sui-execution -> rtd-execution"
        fi
    fi

    # 4.2 重命名子目录
    log_info "4.2 重命名 rtd-execution 子目录..."
    for version in v0 v1 v2 latest; do
        if [ -d "rtd-execution/$version" ]; then
            log_info "  处理 rtd-execution/$version..."

            if [ -d "rtd-execution/$version/sui-adapter" ]; then
                mv "rtd-execution/$version/sui-adapter" "rtd-execution/$version/rtd-adapter"
            fi

            if [ -d "rtd-execution/$version/sui-verifier" ]; then
                mv "rtd-execution/$version/sui-verifier" "rtd-execution/$version/rtd-verifier"
            fi

            if [ -d "rtd-execution/$version/sui-move-natives" ]; then
                mv "rtd-execution/$version/sui-move-natives" "rtd-execution/$version/rtd-move-natives"
            fi
        fi
    done

    log_success "sui-execution 目录重命名完成"
}

################################################################################
# Phase 5: 重命名框架目录
################################################################################
phase_5_rename_framework() {
    log_phase "Phase 5: 重命名框架目录"

    # 5.1 重命名 packages 子目录
    log_info "5.1 重命名框架 packages 子目录..."

    if [ -d "crates/rtd-framework/packages/sui-framework" ]; then
        mv "crates/rtd-framework/packages/sui-framework" "crates/rtd-framework/packages/rtd-framework"
        log_info "  packages/sui-framework -> packages/rtd-framework"
    fi

    if [ -d "crates/rtd-framework/packages/sui-system" ]; then
        mv "crates/rtd-framework/packages/sui-system" "crates/rtd-framework/packages/rtd-system"
        log_info "  packages/sui-system -> packages/rtd-system"
    fi

    # 5.2 重命名 packages_compiled 文件
    log_info "5.2 重命名 packages_compiled 文件..."
    COMPILED_DIR="crates/rtd-framework/packages_compiled"
    if [ -d "$COMPILED_DIR" ]; then
        if [ -f "$COMPILED_DIR/sui-framework" ]; then
            mv "$COMPILED_DIR/sui-framework" "$COMPILED_DIR/rtd-framework"
            log_info "  sui-framework -> rtd-framework"
        fi

        if [ -f "$COMPILED_DIR/sui-system" ]; then
            mv "$COMPILED_DIR/sui-system" "$COMPILED_DIR/rtd-system"
            log_info "  sui-system -> rtd-system"
        fi
    fi

    # 5.3 重命名 docs 子目录
    log_info "5.3 重命名框架 docs 子目录..."
    if [ -d "crates/rtd-framework/docs/sui" ]; then
        mv "crates/rtd-framework/docs/sui" "crates/rtd-framework/docs/rtd"
        log_info "  docs/sui -> docs/rtd"
    fi

    if [ -d "crates/rtd-framework/docs/sui_system" ]; then
        mv "crates/rtd-framework/docs/sui_system" "crates/rtd-framework/docs/rtd_system"
        log_info "  docs/sui_system -> docs/rtd_system"
    fi

    log_success "框架目录重命名完成"
}

################################################################################
# Phase 6: 重命名 docker 目录
################################################################################
phase_6_rename_docker() {
    log_phase "Phase 6: 重命名 docker 目录"

    log_info "6.1 重命名 docker/sui-* 目录..."
    for dir in docker/sui-*; do
        if [ -d "$dir" ]; then
            new_dir="${dir/sui-/rtd-}"
            if [ ! -d "$new_dir" ]; then
                log_info "  $dir -> $new_dir"
                mv "$dir" "$new_dir"
            fi
        fi
    done

    log_info "6.2 重命名 crates/rtd-rosetta/docker 子目录..."
    for dir in crates/rtd-rosetta/docker/sui-*; do
        if [ -d "$dir" ]; then
            new_dir="${dir/sui-/rtd-}"
            if [ ! -d "$new_dir" ]; then
                log_info "  $dir -> $new_dir"
                mv "$dir" "$new_dir"
            fi
        fi
    done

    log_success "docker 目录重命名完成"
}

################################################################################
# Phase 7: 重命名 nre/ansible 目录
################################################################################
phase_7_rename_ansible() {
    log_phase "Phase 7: 重命名 nre/ansible 目录"

    log_info "重命名 nre/ansible/roles/sui-node..."
    if [ -d "nre/ansible/roles/sui-node" ]; then
        mv "nre/ansible/roles/sui-node" "nre/ansible/roles/rtd-node"
        log_info "  sui-node -> rtd-node"
    fi

    log_success "nre/ansible 目录重命名完成"
}

################################################################################
# Phase 8: 重命名 docs/content 目录
################################################################################
phase_8_rename_docs() {
    log_phase "Phase 8: 重命名 docs/content 目录"

    declare -a DOCS_DIRS=(
        "docs/content/references/sui-api:rtd-api"
        "docs/content/guides/developer/sui-101:rtd-101"
        "docs/content/concepts/sui-architecture:rtd-architecture"
        "docs/content/concepts/sui-move-concepts:rtd-move-concepts"
        "docs/content/guides/suiplay0x1:rtdplay0x1"
    )

    for entry in "${DOCS_DIRS[@]}"; do
        old_path="${entry%%:*}"
        new_name="${entry##*:}"

        if [ -d "$old_path" ]; then
            parent_dir=$(dirname "$old_path")
            log_info "  $old_path -> $parent_dir/$new_name"
            mv "$old_path" "$parent_dir/$new_name"
        fi
    done

    log_success "docs/content 目录重命名完成"
}

################################################################################
# Phase 9: 重命名 external-crates/move 目录
################################################################################
phase_9_rename_move_compiler() {
    log_phase "Phase 9: 重命名 external-crates/move 目录"

    declare -a MOVE_DIRS=(
        "external-crates/move/crates/move-stackless-bytecode-2/tests/move/sui_move_2024:rtd_move_2024"
        "external-crates/move/crates/move-compiler/tests/sui_mode:rtd_mode"
        "external-crates/move/crates/move-compiler/tests/move_2024/sui_mode:rtd_mode"
        "external-crates/move/crates/move-compiler/src/sui_mode:rtd_mode"
    )

    for entry in "${MOVE_DIRS[@]}"; do
        old_path="${entry%%:*}"
        new_name="${entry##*:}"

        if [ -d "$old_path" ]; then
            parent_dir=$(dirname "$old_path")
            log_info "  $old_path -> $parent_dir/$new_name"
            mv "$old_path" "$parent_dir/$new_name"
        fi
    done

    log_success "external-crates/move 目录重命名完成"
}

################################################################################
# Phase 10: 重命名其他 sui* 目录
################################################################################
phase_10_rename_other_dirs() {
    log_phase "Phase 10: 重命名其他 sui* 目录"

    # 10.1 使用通配符查找并重命名所有 sui_* 目录
    log_info "10.1 查找并重命名所有 sui_* 目录..."
    find . -type d -name "sui_*" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null | sort -r | while read -r dir; do
        parent_dir=$(dirname "$dir")
        base=$(basename "$dir")
        new_base="${base/sui_/rtd_}"
        if [ "$base" != "$new_base" ] && [ ! -d "$parent_dir/$new_base" ]; then
            log_info "  $dir -> $parent_dir/$new_base"
            mv "$dir" "$parent_dir/$new_base"
        fi
    done

    # 10.2 查找并重命名所有 suins* 目录
    log_info "10.2 查找并重命名所有 suins* 目录..."
    find . -type d -name "suins*" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null | sort -r | while read -r dir; do
        parent_dir=$(dirname "$dir")
        base=$(basename "$dir")
        new_base="${base/suins/rtdns}"
        if [ "$base" != "$new_base" ] && [ ! -d "$parent_dir/$new_base" ]; then
            log_info "  $dir -> $parent_dir/$new_base"
            mv "$dir" "$parent_dir/$new_base"
        fi
    done

    # 10.3 特定目录重命名
    log_info "10.3 特定目录重命名..."
    declare -a OTHER_DIRS=(
        "crates/rtd-rpc-api/proto/sui:rtd"
        "crates/rtd-indexer-alt-consistent-api/proto/sui:rtd"
    )

    for entry in "${OTHER_DIRS[@]}"; do
        old_path="${entry%%:*}"
        new_name="${entry##*:}"

        if [ -d "$old_path" ]; then
            parent_dir=$(dirname "$old_path")
            log_info "  $old_path -> $parent_dir/$new_name"
            mv "$old_path" "$parent_dir/$new_name"
        fi
    done

    log_success "其他目录重命名完成"
}

################################################################################
# Phase 11: 重命名文件
################################################################################
phase_11_rename_files() {
    log_phase "Phase 11: 重命名文件"

    # 11.1 重命名 sui.* 文件
    log_info "11.1 重命名 sui.* 文件..."
    find . -type f -name "sui.*" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        ! -name "*.pdf" \
        2>/dev/null | while read -r file; do
        dir=$(dirname "$file")
        base=$(basename "$file")
        new_base="${base/sui./rtd.}"
        if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
            log_info "  $file -> $dir/$new_base"
            mv "$file" "$dir/$new_base"
        fi
    done

    # 11.2 重命名 sui_*.rs 文件
    log_info "11.2 重命名 sui_*.rs 文件..."
    find . -type f -name "sui_*.rs" \
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

    # 11.3 重命名 *_sui_*.rs 文件
    log_info "11.3 重命名 *_sui_*.rs 文件..."
    find . -type f -name "*_sui_*.rs" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null | while read -r file; do
        dir=$(dirname "$file")
        base=$(basename "$file")
        new_base="${base/_sui_/_rtd_}"
        if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
            log_info "  $file -> $dir/$new_base"
            mv "$file" "$dir/$new_base"
        fi
    done

    # 11.4 重命名 *_sui.rs 文件
    log_info "11.4 重命名 *_sui.rs 文件..."
    find . -type f -name "*_sui.rs" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null | while read -r file; do
        dir=$(dirname "$file")
        base=$(basename "$file")
        new_base="${base/_sui.rs/_rtd.rs}"
        if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
            log_info "  $file -> $dir/$new_base"
            mv "$file" "$dir/$new_base"
        fi
    done

    # 11.5 重命名 suins*.rs 文件
    log_info "11.5 重命名 suins*.rs 文件..."
    find . -type f -name "suins*.rs" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null | while read -r file; do
        dir=$(dirname "$file")
        base=$(basename "$file")
        new_base="${base/suins/rtdns}"
        if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
            log_info "  $file -> $dir/$new_base"
            mv "$file" "$dir/$new_base"
        fi
    done

    # 11.6 重命名 sui.move 文件
    log_info "11.6 重命名 sui.move 文件..."
    find . -type f -name "sui.move" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null | while read -r file; do
        dir=$(dirname "$file")
        if [ ! -f "$dir/rtd.move" ]; then
            log_info "  $file -> $dir/rtd.move"
            mv "$file" "$dir/rtd.move"
        fi
    done

    # 11.7 重命名其他 sui-* 文件
    log_info "11.7 重命名 sui-* 文件..."
    find . -type f -name "sui-*" \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null | while read -r file; do
        dir=$(dirname "$file")
        base=$(basename "$file")
        new_base="${base/sui-/rtd-}"
        if [ "$base" != "$new_base" ] && [ ! -f "$dir/$new_base" ]; then
            log_info "  $file -> $dir/$new_base"
            mv "$file" "$dir/$new_base"
        fi
    done

    log_success "文件重命名完成"
}

################################################################################
# Phase 12: 修复特殊情况
################################################################################
phase_12_fix_special_cases() {
    log_phase "Phase 12: 修复特殊情况"

    # 12.1 修复 move-compiler editions/mod.rs 中的常量冲突
    log_info "12.1 检查并修复 move-compiler 常量冲突..."
    EDITIONS_FILE="external-crates/move/crates/move-compiler/src/editions/mod.rs"
    if [ -f "$EDITIONS_FILE" ]; then
        # 检查是否有常量与枚举变体冲突
        if grep -q 'pub const RTD: &'"'"'static str' "$EDITIONS_FILE" 2>/dev/null; then
            log_info "  修复常量冲突: RTD -> RTD_FLAVOR"
            sed -i '' 's/pub const RTD: &'"'"'static str = "rtd";/pub const RTD_FLAVOR: \&'"'"'static str = "rtd";/g' "$EDITIONS_FILE"
            sed -i '' 's/Self::RTD => Self::RTD/Self::RTD_FLAVOR => Self::RTD/g' "$EDITIONS_FILE"
        fi
    fi

    log_success "特殊情况修复完成"
}

################################################################################
# Phase 13: 清理和验证
################################################################################
phase_13_cleanup_verify() {
    log_phase "Phase 13: 清理和验证"

    # 13.1 清理 cynic 缓存
    log_info "13.1 清理构建缓存..."
    find ./target -type d -name "cynic-schemas" 2>/dev/null | while read -r dir; do
        log_info "  删除缓存: $dir"
        rm -rf "$dir"
    done

    # 13.2 检查遗漏的目录
    log_info "13.2 检查遗漏的 sui*/mysten* 目录..."
    echo ""
    echo "=== 遗漏的目录 ==="
    find . -type d \( -name "sui-*" -o -name "sui_*" -o -name "mysten-*" -o -name "mysten_*" -o -name "sui" \) \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./node_modules/*" \
        ! -path "./fork-instruct/*" \
        2>/dev/null || true

    # 13.3 检查遗漏的文件
    log_info "13.3 检查遗漏的 sui* 文件..."
    echo ""
    echo "=== 遗漏的文件 ==="
    find . -type f \( -name "sui_*.rs" -o -name "*_sui.rs" -o -name "*_sui_*.rs" -o -name "sui.*" -o -name "sui-*" \) \
        ! -path "./.git/*" \
        ! -path "./target/*" \
        ! -path "./node_modules/*" \
        ! -path "./fork-instruct/*" \
        ! -name "*.pdf" \
        2>/dev/null || true

    # 13.4 统计替换结果
    log_info "13.4 统计文本替换结果..."
    echo ""
    echo "=== 文本内容统计 ==="
    echo "  - LinkUVerse 出现次数: $(grep -r "LinkUVerse" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | wc -l | tr -d ' ')"
    echo "  - LinkU 出现次数: $(grep -r "LinkU" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | grep -v "LinkUVerse" | wc -l | tr -d ' ')"
    echo "  - RTD 出现次数: $(grep -r "RTD" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | wc -l | tr -d ' ')"
    echo "  - Rtd 出现次数: $(grep -r "Rtd" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | wc -l | tr -d ' ')"
    echo "  - rtd 出现次数: $(grep -r "rtd" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | wc -l | tr -d ' ')"
    echo ""
    echo "=== 残留检查 ==="
    echo "  - MystenLabs 残留: $(grep -r "MystenLabs" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | wc -l | tr -d ' ')"
    echo "  - Mysten 残留: $(grep -r "Mysten" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | grep -v "LinkU" | wc -l | tr -d ' ')"
    echo "  - mysten 残留: $(grep -r "mysten" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | grep -v "linku" | wc -l | tr -d ' ')"
    echo "  - SUI 残留: $(grep -rw "SUI" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | grep -v "RTD" | wc -l | tr -d ' ')"
    echo "  - Sui 残留: $(grep -r "Sui" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | grep -v "Rtd" | wc -l | tr -d ' ')"
    echo "  - sui 残留: $(grep -r "sui" . --include="*.rs" --include="*.toml" --include="*.md" 2>/dev/null | grep -v ".git" | grep -v "target" | grep -v "fork-instruct" | grep -v "rtd" | grep -v "suite" | wc -l | tr -d ' ')"
}

################################################################################
# 主函数
################################################################################
main() {
    echo "========================================"
    echo "  RTD 品牌重命名自动化脚本 v2"
    echo "========================================"
    echo ""
    echo "替换规则:"
    echo "  MystenLabs -> LinkUVerse"
    echo "  Mysten     -> LinkU"
    echo "  mysten     -> linku"
    echo "  SUI        -> RTD"
    echo "  Sui        -> Rtd"
    echo "  sui        -> rtd"
    echo ""
    echo "SDK 依赖更新:"
    echo "  sui-rust-sdk -> rtd-rust-sdk"
    echo "  rev: 339c2272... -> 2fff36e9..."
    echo ""
    echo "项目根目录: $PROJECT_ROOT"
    echo ""

    # 询问是否继续
    read -p "是否继续执行? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "用户取消执行"
        exit 0
    fi

    # 执行各阶段
    phase_0_prepare
    phase_1_text_replacement
    phase_2_update_sdk_deps
    phase_3_rename_crates
    phase_4_rename_execution
    phase_5_rename_framework
    phase_6_rename_docker
    phase_7_rename_ansible
    phase_8_rename_docs
    phase_9_rename_move_compiler
    phase_10_rename_other_dirs
    phase_11_rename_files
    phase_12_fix_special_cases
    phase_13_cleanup_verify

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

# 支持单独执行某个阶段
case "${1:-all}" in
    phase0|prepare)
        phase_0_prepare
        ;;
    phase1|text)
        phase_1_text_replacement
        ;;
    phase2|sdk)
        phase_2_update_sdk_deps
        ;;
    phase3|crates)
        phase_3_rename_crates
        ;;
    phase4|execution)
        phase_4_rename_execution
        ;;
    phase5|framework)
        phase_5_rename_framework
        ;;
    phase6|docker)
        phase_6_rename_docker
        ;;
    phase7|ansible)
        phase_7_rename_ansible
        ;;
    phase8|docs)
        phase_8_rename_docs
        ;;
    phase9|move)
        phase_9_rename_move_compiler
        ;;
    phase10|other)
        phase_10_rename_other_dirs
        ;;
    phase11|files)
        phase_11_rename_files
        ;;
    phase12|fix)
        phase_12_fix_special_cases
        ;;
    phase13|verify)
        phase_13_cleanup_verify
        ;;
    all)
        main
        ;;
    *)
        echo "用法: $0 [phase0|phase1|...|phase13|all]"
        echo ""
        echo "阶段说明:"
        echo "  phase0/prepare    - 准备工作（创建备份）"
        echo "  phase1/text       - 文本内容替换"
        echo "  phase2/sdk        - 更新 SDK 依赖"
        echo "  phase3/crates     - 重命名 crates 目录"
        echo "  phase4/execution  - 重命名 sui-execution 目录"
        echo "  phase5/framework  - 重命名框架目录"
        echo "  phase6/docker     - 重命名 docker 目录"
        echo "  phase7/ansible    - 重命名 ansible 目录"
        echo "  phase8/docs       - 重命名 docs 目录"
        echo "  phase9/move       - 重命名 move compiler 目录"
        echo "  phase10/other     - 重命名其他目录"
        echo "  phase11/files     - 重命名文件"
        echo "  phase12/fix       - 修复特殊情况"
        echo "  phase13/verify    - 清理和验证"
        echo "  all               - 执行所有阶段（默认）"
        exit 1
        ;;
esac
