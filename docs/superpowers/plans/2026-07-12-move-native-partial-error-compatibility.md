# Move Native Partial Error 兼容修复实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 修复安全加固补丁对未合入的新 Move VM 辅助宏的引用，并保持 RTD debug 构建通过。

**架构：** 保留 native 安全加固逻辑，仅将两处 `partial_vm_error!` 调用替换为 RTD 当前 Move 版本已经广泛使用的 `PartialVMError` 构造方式。不移植新 Move VM 提交中的宏或其他功能，从而避免扩大上游 feature 的合入范围。

**技术栈：** Rust、Cargo、Move VM native runtime

---

### 任务 1：记录编译回归

**文件：**
- 测试：`rtd-execution/latest/rtd-move-natives/src/dynamic_field.rs`

- [x] **步骤 1：运行针对性编译并确认失败**

运行：`cargo check -p rtd-move-natives-latest`

预期：FAIL，报告 `move_binary_format::partial_vm_error` 未定义。

### 任务 2：使用现有错误 API 完成兼容修复

**文件：**
- 修改：`rtd-execution/latest/rtd-move-natives/src/dynamic_field.rs`

- [ ] **步骤 1：移除不存在的宏导入**

保留已有安全宏导入：

```rust
use move_binary_format::{safe_assert, safe_assert_eq, safe_unwrap, safe_unwrap_err};
```

- [ ] **步骤 2：替换两处不兼容调用**

在 `borrow_global` 和 `move_from` 的 `MISSING_DATA` 分支使用现有 API：

```rust
PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
    .with_message("operation returned MISSING_DATA after exists() was true".to_owned())
```

- [ ] **步骤 3：运行针对性编译验证通过**

运行：`cargo check -p rtd-move-natives-latest`

预期：PASS，不再报告 `partial_vm_error` 导入错误。

### 任务 3：补全 accumulator 合并补丁的既有类型导入

**文件：**
- 修改：`rtd-execution/latest/rtd-adapter/src/programmable_transactions/context.rs`

- [x] **步骤 1：运行完整构建并确认失败**

运行：`cargo build`

预期：FAIL，报告 `AccumulatorWriteV1` 和 `AccumulatorObjId` 未导入。

- [ ] **步骤 2：恢复补丁仍在使用的类型导入**

```rust
use rtd_types::{
    accumulator_root::AccumulatorObjId,
    effects::{AccumulatorAddress, AccumulatorValue, AccumulatorWriteV1},
};
```

- [ ] **步骤 3：运行针对性编译验证通过**

运行：`cargo check -p rtd-adapter-latest`

预期：PASS，不再报告 accumulator 类型未定义。

### 任务 4：修正共识重复提交补丁的语法残留

**文件：**
- 修改：`crates/rtd-core/src/authority_server.rs`

- [x] **步骤 1：运行完整构建并确认失败**

运行：`cargo build`

预期：FAIL，报告 soft bundle 的 `vec!` 调用在 `;]` 处意外结束。

- [ ] **步骤 2：恢复合法的单元素 vector 构造**

```rust
vec![
    transaction_indexes
        .into_iter()
        .zip(tx_digests)
        .collect::<Vec<_>>(),
]
```

- [ ] **步骤 3：运行针对性编译验证通过**

运行：`cargo check -p rtd-core`

预期：PASS，不再报告 macro invocation 语法错误。

### 任务 5：修正共识补丁的错误转换和异步借用

**文件：**
- 修改：`crates/rtd-core/src/authority_server.rs`
- 修改：`crates/rtd-core/src/consensus_adapter.rs`

- [x] **步骤 1：运行针对性编译并确认失败**

运行：`cargo check -p rtd-core`

预期：FAIL，报告 `RtdErrorKind::into()` 类型多义以及 `tx_consensus_positions` 重复可变借用。

- [ ] **步骤 2：显式构造 RTD 错误类型**

```rust
RtdError::from(RtdErrorKind::FailedToSubmitToConsensus(format!(
    "Failed to get consensus position: {e}"
)))
```

- [ ] **步骤 3：在 select 结果析构后发送 processing error**

记录 `processed_via_notify`，先完成 `select(...).await` 的 match，再对 `tx_consensus_positions.take()` 执行错误发送，缩短 submit future 对 sender 的可变借用生命周期。

- [ ] **步骤 4：重新运行针对性编译验证通过**

运行：`cargo check -p rtd-core`

预期：PASS，不再报告类型推导或可变借用错误。

### 任务 6：修正 bridge 审计测试的跨版本状态码比较

**文件：**
- 修改：`crates/rtd-bridge/src/server/mod.rs`

- [x] **步骤 1：运行 clippy 并确认测试目标失败**

运行：`cargo xclippy`

预期：FAIL，报告 `reqwest::StatusCode` 无法与 `axum::http::StatusCode` 比较。

- [ ] **步骤 2：按 HTTP 数值比较状态码**

```rust
assert_eq!(
    response.status().as_u16(),
    StatusCode::URI_TOO_LONG.as_u16()
);
```

- [ ] **步骤 3：运行 bridge 全目标 clippy**

运行：`cargo clippy -p rtd-bridge --all-targets --features test-utils`

预期：PASS；启用该 crate 既有的测试工具依赖后，oversized URI 审计回归测试可正常编译。

### 任务 7：验证并提交全部工作区改动

**文件：**
- 验证：Cargo workspace 全部 debug targets
- 提交：工作区当前全部已跟踪和未跟踪文件

- [ ] **步骤 1：重新运行完整 debug 构建**

运行：`cargo build`

预期：PASS，生成 debug 产物且无编译错误。

- [ ] **步骤 2：执行仓库要求的静态检查**

运行：`cargo xclippy`

预期：PASS；若发现与本次改动无关的既有问题，记录完整结果后评估是否影响提交。

- [ ] **步骤 3：检查变更完整性**

运行：`git diff --check` 和 `git status --short`

预期：无空白错误，变更范围符合当前工作区内容。

- [ ] **步骤 4：提交全部改动**

```bash
git add -A
git commit -m "fix: complete upstream patch compatibility"
```
