#!/bin/bash

# 详细的编译和单元测试验证脚本
# 收集所有编译错误和测试失败信息

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
REPORT_DIR="$SCRIPT_DIR/test-reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

cd "$PROJECT_ROOT"

# 创建报告目录
mkdir -p "$REPORT_DIR"

echo "========================================"
echo "RTD 详细编译和测试验证脚本"
echo "报告目录: $REPORT_DIR"
echo "========================================"

# 1. 编译检查
echo ""
echo "[1/4] 运行 cargo check..."
CARGO_CHECK_LOG="$REPORT_DIR/cargo_check_$TIMESTAMP.log"
cargo check --workspace 2>&1 | tee "$CARGO_CHECK_LOG"
CARGO_CHECK_STATUS=${PIPESTATUS[0]}

if [ $CARGO_CHECK_STATUS -eq 0 ]; then
    echo "✅ cargo check 通过"
else
    echo "❌ cargo check 失败"
    echo ""
    echo "编译错误摘要:"
    grep -E "^error\[|^error:" "$CARGO_CHECK_LOG" | head -50
    echo ""
    echo "详细日志: $CARGO_CHECK_LOG"
fi

# 2. 编译构建
echo ""
echo "[2/4] 运行 cargo build..."
CARGO_BUILD_LOG="$REPORT_DIR/cargo_build_$TIMESTAMP.log"
cargo build --workspace 2>&1 | tee "$CARGO_BUILD_LOG"
CARGO_BUILD_STATUS=${PIPESTATUS[0]}

if [ $CARGO_BUILD_STATUS -eq 0 ]; then
    echo "✅ cargo build 通过"
else
    echo "❌ cargo build 失败"
    echo ""
    echo "编译错误摘要:"
    grep -E "^error\[|^error:" "$CARGO_BUILD_LOG" | head -50
    echo ""
    echo "详细日志: $CARGO_BUILD_LOG"
fi

# 如果编译失败，跳过测试
if [ $CARGO_BUILD_STATUS -ne 0 ]; then
    echo ""
    echo "========================================"
    echo "由于编译失败，跳过单元测试"
    echo "请先修复编译错误"
    echo "========================================"

    # 生成错误报告
    ERROR_REPORT="$REPORT_DIR/error_report_$TIMESTAMP.md"
    echo "# 编译错误报告" > "$ERROR_REPORT"
    echo "" >> "$ERROR_REPORT"
    echo "生成时间: $(date)" >> "$ERROR_REPORT"
    echo "" >> "$ERROR_REPORT"
    echo "## 错误列表" >> "$ERROR_REPORT"
    echo "" >> "$ERROR_REPORT"
    echo '```' >> "$ERROR_REPORT"
    grep -E "^error\[|^error:" "$CARGO_BUILD_LOG" >> "$ERROR_REPORT"
    echo '```' >> "$ERROR_REPORT"
    echo "" >> "$ERROR_REPORT"
    echo "## 受影响的 crate" >> "$ERROR_REPORT"
    echo "" >> "$ERROR_REPORT"
    grep "could not compile" "$CARGO_BUILD_LOG" | sort | uniq >> "$ERROR_REPORT"

    echo "错误报告已生成: $ERROR_REPORT"
    exit 1
fi

# 3. Clippy 检查
echo ""
echo "[3/4] 运行 cargo clippy..."
CARGO_CLIPPY_LOG="$REPORT_DIR/cargo_clippy_$TIMESTAMP.log"
cargo xclippy 2>&1 | tee "$CARGO_CLIPPY_LOG"
CARGO_CLIPPY_STATUS=${PIPESTATUS[0]}

if [ $CARGO_CLIPPY_STATUS -eq 0 ]; then
    echo "✅ cargo clippy 通过"
else
    echo "⚠️ cargo clippy 有警告或错误"
    echo "详细日志: $CARGO_CLIPPY_LOG"
fi

# 4. 单元测试
echo ""
echo "[4/4] 运行单元测试..."
CARGO_TEST_LOG="$REPORT_DIR/cargo_test_$TIMESTAMP.log"
RTD_SKIP_SIMTESTS=1 cargo test --workspace --lib 2>&1 | tee "$CARGO_TEST_LOG"
CARGO_TEST_STATUS=${PIPESTATUS[0]}

if [ $CARGO_TEST_STATUS -eq 0 ]; then
    echo "✅ 单元测试通过"
else
    echo "❌ 单元测试失败"
    echo ""
    echo "失败的测试:"
    grep -E "^test .* FAILED|failures:" "$CARGO_TEST_LOG" | head -50
    echo ""
    echo "详细日志: $CARGO_TEST_LOG"
fi

# 生成最终报告
echo ""
echo "========================================"
echo "测试报告摘要"
echo "========================================"

FINAL_REPORT="$REPORT_DIR/final_report_$TIMESTAMP.md"
echo "# RTD 品牌重命名验证报告" > "$FINAL_REPORT"
echo "" >> "$FINAL_REPORT"
echo "生成时间: $(date)" >> "$FINAL_REPORT"
echo "" >> "$FINAL_REPORT"
echo "## 测试结果" >> "$FINAL_REPORT"
echo "" >> "$FINAL_REPORT"
echo "| 检查项 | 状态 |" >> "$FINAL_REPORT"
echo "|--------|------|" >> "$FINAL_REPORT"

if [ $CARGO_CHECK_STATUS -eq 0 ]; then
    echo "| cargo check | ✅ 通过 |" >> "$FINAL_REPORT"
    echo "  cargo check:  ✅ 通过"
else
    echo "| cargo check | ❌ 失败 |" >> "$FINAL_REPORT"
    echo "  cargo check:  ❌ 失败"
fi

if [ $CARGO_BUILD_STATUS -eq 0 ]; then
    echo "| cargo build | ✅ 通过 |" >> "$FINAL_REPORT"
    echo "  cargo build:  ✅ 通过"
else
    echo "| cargo build | ❌ 失败 |" >> "$FINAL_REPORT"
    echo "  cargo build:  ❌ 失败"
fi

if [ $CARGO_CLIPPY_STATUS -eq 0 ]; then
    echo "| cargo clippy | ✅ 通过 |" >> "$FINAL_REPORT"
    echo "  cargo clippy: ✅ 通过"
else
    echo "| cargo clippy | ⚠️ 警告 |" >> "$FINAL_REPORT"
    echo "  cargo clippy: ⚠️ 警告"
fi

if [ $CARGO_TEST_STATUS -eq 0 ]; then
    echo "| 单元测试 | ✅ 通过 |" >> "$FINAL_REPORT"
    echo "  单元测试:     ✅ 通过"
else
    echo "| 单元测试 | ❌ 失败 |" >> "$FINAL_REPORT"
    echo "  单元测试:     ❌ 失败"
fi

echo "" >> "$FINAL_REPORT"
echo "## 日志文件" >> "$FINAL_REPORT"
echo "" >> "$FINAL_REPORT"
echo "- cargo check: $CARGO_CHECK_LOG" >> "$FINAL_REPORT"
echo "- cargo build: $CARGO_BUILD_LOG" >> "$FINAL_REPORT"
echo "- cargo clippy: $CARGO_CLIPPY_LOG" >> "$FINAL_REPORT"
echo "- cargo test: $CARGO_TEST_LOG" >> "$FINAL_REPORT"

echo ""
echo "最终报告: $FINAL_REPORT"
echo "========================================"
