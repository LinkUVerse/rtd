#!/bin/bash

# 单元测试专用脚本
# 运行单元测试并收集失败用例

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
REPORT_DIR="$SCRIPT_DIR/test-reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

cd "$PROJECT_ROOT"

# 创建报告目录
mkdir -p "$REPORT_DIR"

echo "========================================"
echo "RTD 单元测试脚本"
echo "========================================"

# 运行单元测试
echo "运行单元测试..."
TEST_LOG="$REPORT_DIR/unit_test_$TIMESTAMP.log"
RTD_SKIP_SIMTESTS=1 cargo test --workspace --lib 2>&1 | tee "$TEST_LOG"
TEST_STATUS=${PIPESTATUS[0]}

echo ""
echo "========================================"

if [ $TEST_STATUS -eq 0 ]; then
    echo "✅ 所有单元测试通过"
else
    echo "❌ 存在失败的单元测试"
    echo ""

    # 提取失败的测试用例
    FAILED_TESTS="$REPORT_DIR/failed_tests_$TIMESTAMP.txt"
    echo "# 失败的单元测试列表" > "$FAILED_TESTS"
    echo "# 生成时间: $(date)" >> "$FAILED_TESTS"
    echo "" >> "$FAILED_TESTS"

    # 提取 FAILED 的测试
    echo "## 失败的测试用例:" >> "$FAILED_TESTS"
    grep "^test .* FAILED" "$TEST_LOG" >> "$FAILED_TESTS" 2>/dev/null || echo "无法提取失败测试" >> "$FAILED_TESTS"

    echo ""
    echo "## 错误摘要:" >> "$FAILED_TESTS"
    grep -A 5 "^failures:" "$TEST_LOG" >> "$FAILED_TESTS" 2>/dev/null || true

    # 统计
    FAILED_COUNT=$(grep -c "^test .* FAILED" "$TEST_LOG" 2>/dev/null || echo "0")
    PASSED_COUNT=$(grep -c "^test .* ok" "$TEST_LOG" 2>/dev/null || echo "0")

    echo ""
    echo "测试统计:"
    echo "  通过: $PASSED_COUNT"
    echo "  失败: $FAILED_COUNT"
    echo ""
    echo "失败的测试用例:"
    grep "^test .* FAILED" "$TEST_LOG" 2>/dev/null || echo "  (无法解析)"
    echo ""
    echo "详细日志: $TEST_LOG"
    echo "失败列表: $FAILED_TESTS"
fi

echo "========================================"
