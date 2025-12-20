#!/bin/bash
set -e

# 修复 macOS sed 的 "illegal byte sequence" 错误
export LC_ALL=C
export LANG=C

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "========================================"
echo "RTD 品牌重命名补丁脚本 (Patch 8)"
echo "修复 GraphQL schema 中的 SuiAddress 类型"
echo "========================================"

# 1. 更新 rtd-graphql-rpc/schema.graphql
echo "Phase 1: 更新 rtd-graphql-rpc/schema.graphql..."

GRAPHQL_SCHEMA1="crates/rtd-graphql-rpc/schema.graphql"
if [ -f "$GRAPHQL_SCHEMA1" ]; then
    # 替换所有 SuiAddress 为 RTDAddress
    sed -i '' 's/SuiAddress/RTDAddress/g' "$GRAPHQL_SCHEMA1"
    echo "  已更新: $GRAPHQL_SCHEMA1"

    # 统计替换数量
    count=$(grep -c "RTDAddress" "$GRAPHQL_SCHEMA1" 2>/dev/null || echo "0")
    echo "  RTDAddress 出现次数: $count"
else
    echo "  警告: $GRAPHQL_SCHEMA1 不存在"
fi

# 2. 更新 rtd-indexer-alt-graphql/schema.graphql
echo ""
echo "Phase 2: 更新 rtd-indexer-alt-graphql/schema.graphql..."

GRAPHQL_SCHEMA2="crates/rtd-indexer-alt-graphql/schema.graphql"
if [ -f "$GRAPHQL_SCHEMA2" ]; then
    # 替换所有 SuiAddress 为 RTDAddress
    sed -i '' 's/SuiAddress/RTDAddress/g' "$GRAPHQL_SCHEMA2"
    echo "  已更新: $GRAPHQL_SCHEMA2"

    # 统计替换数量
    count=$(grep -c "RTDAddress" "$GRAPHQL_SCHEMA2" 2>/dev/null || echo "0")
    echo "  RTDAddress 出现次数: $count"
else
    echo "  警告: $GRAPHQL_SCHEMA2 不存在"
fi

# 3. 检查其他可能的 GraphQL schema 文件
echo ""
echo "Phase 3: 检查其他 GraphQL schema 文件..."

find . -name "*.graphql" ! -path "./.git/*" ! -path "./target/*" 2>/dev/null | while read -r file; do
    if grep -q "SuiAddress" "$file" 2>/dev/null; then
        echo "  发现 SuiAddress: $file"
        sed -i '' 's/SuiAddress/RTDAddress/g' "$file"
        echo "    已更新"
    fi
done

# 4. 清理 target 目录中的生成文件，强制重新生成
echo ""
echo "Phase 4: 清理 cynic 生成的缓存文件..."

# 删除 cynic-schemas 目录以强制重新生成
find ./target -type d -name "cynic-schemas" 2>/dev/null | while read -r dir; do
    echo "  删除缓存: $dir"
    rm -rf "$dir"
done

# 5. 最终检查
echo ""
echo "Phase 5: 最终检查..."

remaining=$(find . -name "*.graphql" ! -path "./.git/*" ! -path "./target/*" -exec grep -l "SuiAddress" {} \; 2>/dev/null | wc -l)
if [ "$remaining" -gt 0 ]; then
    echo "还有 $remaining 个 GraphQL 文件包含 SuiAddress:"
    find . -name "*.graphql" ! -path "./.git/*" ! -path "./target/*" -exec grep -l "SuiAddress" {} \; 2>/dev/null
else
    echo "✅ 所有 GraphQL schema 文件已更新"
fi

echo "========================================"
echo "补丁脚本执行完成！"
echo "请运行 'cargo clean -p rtd-package-dump -p rtd-data-store' 然后重新编译"
echo "========================================"
