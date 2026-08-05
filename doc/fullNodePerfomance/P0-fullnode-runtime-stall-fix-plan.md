# Fullnode 单线程运行时停顿 P0 修复实施方案

> 状态：待审阅
> 优先级：P0
> 适用范围：本地持久化 `rtd start` 启动的 RPC fullnode
> 方案性质：运行时隔离、阻塞任务治理和可观测性增强；不修改共识、协议、Move 执行语义或数据库格式

## 1. 审阅结论

建议按本方案实施，不建议通过延长 SDK/RPC 确认超时、改用
`WaitForEffectsCert` 或为“首次向新地址转账”增加特殊逻辑来处理。

本轮现象的直接原因不是新收款地址需要初始化，也不是 validator 共识或
Move VM 执行慢，而是：

1. 本地 `rtd start` 创建的 RPC fullnode 使用
   `RuntimeType::SingleThreaded`；
2. fullnode checkpoint 后处理仍包含同步 RocksDB 写入、同步索引提交、
   running-root 更新和 checkpoint 清理；
3. 其中一次同步工作或锁等待占住 fullnode 唯一 Tokio worker 后，RPC、
   checkpoint 可见性和 thread-stall monitor 的 async update task 会一起停止；
4. validator 已在亚秒级完成交易，但 SDK 的
   `WaitForLocalExecution` 兼容逻辑仍需轮询当前 fullnode，因而用户看到约
   1～2 分钟“确认时间”；
5. 停顿解除后 fullnode 快速补处理 checkpoint，后续转账暂时恢复毫秒级可见。

因此 P0 必须同时包含：

- **止血：** 只把本地 RPC fullnode 切到多线程 Tokio runtime，避免单个同步
  调用冻结全部 RPC；
- **根治：** 保持 checkpoint 顺序和崩溃恢复语义不变，把已确认的同步持久化
  阶段迁到 Tokio blocking pool，并等待其完成，绝不后台脱离执行；
- **归因：** 为每个 pipeline stage 和 blocking task 增加单次延迟直方图及慢
  阶段日志，使下一次停顿能够定位到具体 checkpoint、stage 和数据库。

这三项应作为两个源码提交和一个部署脚本提交实施，便于单独回滚。

---

## 2. 现场证据与结论边界

### 2.1 已确认事实

| 证据 | 观察 | 结论 |
| --- | --- | --- |
| 首笔 600 RTD 转账 | validator 约在 `13:05:07.173` 执行，fullnode 约在 `13:07:07.509` 才可见，相差约 `120.34s` | 延迟发生在 validator finality 之后 |
| 后续 300 RTD 转账 | validator 与 fullnode 可见时间约相差 `11ms` | 不存在稳定的地址初始化成本 |
| 两笔交易结构 | 都是 `SplitCoins + TransferObjects`，gas 路径一致 | “第一次向新地址转账”只是碰巧撞上停顿 |
| orchestrator/finality 日志 | 对应交易均在约 `0.45s` 内完成 | 共识和 validator 执行不是分钟级延迟来源 |
| fullnode thread-stall 数据 | 当天记录 7 次停顿，总计约 215 秒，其中 3 次超过 10 秒 | fullnode runtime 确实发生进程内调度冻结 |
| fullnode 交易执行直方图 | 约 427 万次执行全部不超过 `0.5s` | Move VM/用户交易执行不是主要嫌疑 |
| RPC 指标 | 一次 `rtdx_getBalance` 落入 60～90 秒桶 | RPC 是同一 runtime 停顿的受害者 |
| RocksDB/APFS | 未出现 flow-control stall；磁盘空间充足；普通 flush 为毫秒级 | 不支持“磁盘满或 RocksDB 全局限流”解释 |
| 重启恢复逻辑 | `rebuild_running_root_to_checkpoint` 只在 `run_epoch` 启动路径执行，而本次长停顿出现在持续运行约 29 小时后 | 现有停机恢复修复与本次在线停顿不是同一问题 |

### 2.2 当前能够下的结论

- **高置信：** RPC fullnode 的 current-thread Tokio runtime 被同步工作或锁等待
  长时间占用。
- **高置信：** 用户感知的“交易确认慢”实质是当前 fullnode 的本地执行/查询可见
  性慢，不是 validator finality 慢。
- **中等置信：** 阻塞发生在 checkpoint executor 的同步后处理或其持有的数据库
  锁附近。
- **尚未证明：** 历史那一次 120 秒停顿究竟是
  `handle_finalized_checkpoint`、RPC index commit、running-root、checkpoint
  pruning，还是另一个同步调用。现有日志过滤器屏蔽了
  `linku_metrics`/`typed_store` 的告警，且现场没有线程栈。

方案不能把“中等置信候选”写成已经证实的唯一根因。P0 的设计目标是即使候选
判断不完整，也先恢复 RPC 隔离能力，并在下次异常时得到确定归因。

---

## 3. 目标与非目标

### 3.1 P0 目标

- 一个 checkpoint 同步写入或锁等待，不得让 fullnode 的 RPC、health 和
  checkpoint 拉取全部冻结。
- 所有迁移到 blocking pool 的 checkpoint 工作仍按 sequence number 严格有序，
  调用方必须 `await` 完成后才能推进下一 stage/watermark。
- 不改变 `highest_executed_checkpoint`、RPC index watermark、running root 和
  transaction output 的提交顺序。
- 慢阶段必须能由 Prometheus 和一条结构化日志定位，不再依赖数 GB 的全量
  `info` 日志猜测。
- 保持 validator 和 msim 现有 runtime 语义，降低回归面。

### 3.2 非目标

- 不改变共识协议、checkpoint 内容、交易排序或对象版本规则。
- 不改变 Move 合约和新地址语义。
- 不把正常成功标准从 `WaitForLocalExecution` 降为
  `WaitForEffectsCert`。
- 不通过增加客户端 60 秒轮询次数来掩盖 fullnode 不可见。
- 不在没有新证据时调整 state-sync timeout、pruner 保留高度或 RocksDB 参数。
- 不在 P0 中重构 `linku_metrics` 的进程级全局单例；该改造范围过大。

---

## 4. 总体实施顺序

| 提交 | 内容 | 目的 | 是否必须 |
| --- | --- | --- | --- |
| A | fullnode 多线程 runtime + runtime 选择测试 | 立即隔离 RPC，缩小行为变更范围 | 是 |
| B | checkpoint stage 延迟指标 + 已知同步持久化阶段 `spawn_blocking` | 移除 async worker 上的阻塞工作并获得精确归因 | 是 |
| C | `toggle_local_rtd.sh` 日志过滤和轮转 | 保留慢调用证据，避免 3 GB 级日志持续增长 | 是，位于部署仓库 |

提交 A 通过针对性测试后即可先进入现场验证；提交 B 合入前后使用同一份压力
脚本做 A/B 对照。不要把三个提交压成一个无法判断效果来源的大改动。

---

## 5. 提交 A：只让 RPC fullnode 使用多线程 runtime

### 5.1 设计原则

当前 `rtd-swarm` 已经实现了两种 runtime：

- `RuntimeType::SingleThreaded` 对应 `tokio::runtime::Builder::new_current_thread()`；
- `RuntimeType::MultiThreaded` 对应 `tokio::runtime::Builder::new_multi_thread()`。

问题不在于缺少多线程实现，而在于所有 `Node::new` 都固定选择
`SingleThreaded`。不应直接把 `Node::new` 的全局默认值改为多线程，否则大量
单元测试、validator swarm 和非本场景调用方都会一起改变。

正确做法是保留单线程默认值，只让 `rtd start` 创建的 RPC fullnode 显式选择
`MultiThreaded`。

### 5.2 源码改动

#### `crates/rtd-swarm/src/memory/node.rs`

1. 为 `RuntimeType` 增加 `PartialEq, Eq`，便于测试。
2. 保留 `Node::new(config)`，继续委托到单线程 runtime。
3. 新增显式构造函数：

```rust
pub fn new_with_runtime(config: NodeConfig, runtime_type: RuntimeType) -> Self
```

4. 增加只读访问器：

```rust
pub fn runtime_type(&self) -> RuntimeType
```

不得提供节点启动后热切换 runtime 的接口；runtime 只能在 `Container::spawn`
前确定。

#### `crates/rtd-swarm/src/memory/swarm.rs`

1. 在 `SwarmBuilder` 增加：

```rust
fullnode_runtime_type: RuntimeType
```

2. 默认值保持 `RuntimeType::SingleThreaded`。
3. `SwarmBuilder::rng()` 必须复制该字段，避免更换 RNG 后静默丢配置。
4. 增加：

```rust
pub fn with_fullnode_runtime_type(mut self, runtime_type: RuntimeType) -> Self
```

5. 构建 validator 时继续调用 `Node::new(config)`。
6. 构建 fullnode 时调用
   `Node::new_with_runtime(config, self.fullnode_runtime_type)`。
7. `Swarm` 自身保存 `fullnode_runtime_type`。
8. `spawn_new_node` 根据 `config.consensus_config.is_none()` 判断角色：
   fullnode 继承 swarm 的 fullnode runtime，validator 仍使用单线程。

这一设计保证持久化配置创建的第一个 fullnode和后续动态创建的 fullnode语义
一致。

#### `crates/rtd/src/rtd_commands.rs`

将导入改为：

```rust
use rtd_swarm::memory::{RuntimeType, Swarm};
```

仅在 `no_full_node == false` 的 fullnode builder 链中加入：

```rust
.with_fullnode_runtime_type(RuntimeType::MultiThreaded)
```

不要给 validator 增加相同设置，也不要把该选择写进 `fullnode.yaml`。它是当前
进程如何承载嵌入式 fullnode 的运行时策略，不是链或节点数据库身份配置。

#### `crates/rtd-swarm/src/memory/container.rs`

现有 `MultiThreaded` 分支可以直接复用，P0 不改变 Tokio 默认 worker 数量。
先使用 Tokio 根据可用 CPU 计算的默认值，避免同时引入新的线程数调参问题。

#### `crates/rtd-swarm/src/memory/container-sim.rs`

保持忽略 `_runtime` 的现状。msim 继续由 simulator runtime 调度，不能为了本地
真实进程性能修改模拟器确定性。

### 5.3 提交 A 测试

在 `rtd-swarm` 增加不启动网络的构建测试：

- 默认 `Swarm::builder()` 的 validator 为 `SingleThreaded`；
- 默认 builder 创建的 fullnode 仍为 `SingleThreaded`；
- 配置 `with_fullnode_runtime_type(MultiThreaded)` 后，fullnode 为
  `MultiThreaded`，validator 仍为 `SingleThreaded`；
- `spawn_new_node` 创建的 fullnode 继承选择；
- 节点 stop/start 后 runtime 类型不变。

另在 `rtd` 命令层增加最小测试或抽取一个无副作用 builder helper，断言
`rtd start` 的非 `--no-full-node` 路径确实选择 `MultiThreaded`。不能只测试
builder API 存在而遗漏实际命令调用。

---

## 6. 提交 B：checkpoint 阻塞阶段治理

### 6.1 先补足单次 stage 指标

当前 `PipelineHandle` 只把各 stage 的总纳秒数累加到 counter。总量无法回答
“哪一个 checkpoint 卡了 120 秒”，必须保留原 counter 兼容已有 dashboard，
同时新增 `HistogramVec`：

```text
checkpoint_executor_pipeline_stage_wait_latency_seconds{stage}
checkpoint_executor_pipeline_stage_active_latency_seconds{stage}
checkpoint_executor_blocking_queue_latency_seconds{stage}
checkpoint_executor_blocking_execution_latency_seconds{stage}
checkpoint_executor_slow_blocking_stage_total{stage}
```

实现文件：

- `crates/rtd-core/src/checkpoints/checkpoint_executor/metrics.rs`
- `crates/rtd-core/src/checkpoints/checkpoint_executor/utils.rs`

要求：

- 使用 `linku_metrics::LATENCY_SEC_BUCKETS`，该 buckets 已覆盖到 90 秒；
  另补 `180s` bucket，确保本次 120 秒停顿不会全部落入 `+Inf`。
- `stage` 是固定枚举字符串，不把 checkpoint sequence、digest 或 DB path
  放入 Prometheus label，避免高基数。
- checkpoint sequence、epoch、stage、queue time 和 execution time 只写入慢
  调用结构化日志。
- 慢调用阈值先固定为 `500ms`，与 thread-stall monitor 的告警阈值一致；P0
  不为此增加持久化配置字段。

在 `PipelineHandle::finish_stage` 中同时记录：

- 原有 `stage_active_duration_ns` counter；
- 新增 `stage_active_latency_seconds` histogram。

在 `PipelineStages::begin` 中同时记录：

- 原有 `stage_wait_duration_ns` counter；
- 新增 `stage_wait_latency_seconds` histogram。

### 6.2 增加有序 blocking helper

在 checkpoint executor 内增加一个统一 helper，所有同步阶段通过该入口运行：

```rust
async fn run_blocking_checkpoint_stage<T, F>(
    &self,
    checkpoint: CheckpointSequenceNumber,
    stage: &'static str,
    operation: F,
) -> T
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
```

helper 必须分别测量：

- 从提交 `spawn_blocking` 到 blocking worker 真正开始的 queue latency；
- closure 自身的 execution latency。

必须满足以下语义：

1. 调用方始终 `.await` `JoinHandle`，禁止 fire-and-forget。
2. closure panic 时恢复原 panic payload，不能转为日志后继续推进 watermark。
3. runtime shutdown/cancel 导致的 `JoinError` 必须终止当前 checkpoint，不得当作
   成功。
4. 超过 `500ms` 时用固定 target 输出 `warn`，字段至少包括：
   `epoch`、`checkpoint`、`stage`、`queue_ms`、`execution_ms`。
5. helper 不设置业务 timeout。DB 写入超时后强行继续会破坏一致性；健康状态应
   失败或进程失败，而不是跳过持久化。

### 6.3 第一批必须迁移的同步阶段

以下操作具有同步 DB 写入或同步清理行为，且输入可通过 `Arc`/owned clone
安全移入 blocking closure，应作为第一批迁移：

| Pipeline stage | 当前同步操作 | 文件 |
| --- | --- | --- |
| `FinalizeCheckpoint` | `AuthorityPerEpochStore::handle_finalized_checkpoint`，包含 quarantine write lock 和同步 `batch.write()` | `checkpoint_executor/mod.rs`、`authority_per_epoch_store.rs` |
| `UpdateRpcIndex` | `RpcIndexStore::commit_update_for_checkpoint`，包含 pending update mutex 和同步 `batch.write()` | `checkpoint_executor/mod.rs`、`rpc_index.rs` |
| 新增 `AccumulateRunningRoot` | `GlobalStateHasher::accumulate_running_root`，包含同步读取/写入 epoch tables | `checkpoint_executor/mod.rs`、`global_state_hasher.rs` |
| `BumpHighestExecutedCheckpoint` | 旧 full-checkpoint contents 删除、digest mapping 删除、highest-executed watermark 写入 | `checkpoint_executor/mod.rs` |

具体要求：

- 每个 closure 只 clone 必需的 `Arc` 和 owned 数据，不在闭包中借用栈上引用。
- 每个 closure 返回后才调用 `finish_stage!`。
- `commit_update_for_checkpoint` 完成后，才向 subscription service 发送
  checkpoint。
- `bump_highest_executed_checkpoint` 完成前，绝不能报告该 checkpoint 已执行。
- 保留现有 `expect`、assert 和 failpoint 语义；不要把一致性错误降级成可忽略
  warning。

### 6.4 给 running root 单独增加 pipeline stage

当前 `accumulate_running_root` 被计入
`BumpHighestExecutedCheckpoint` 的 active time，无法区分两者。修改
`PipelineStage` 为：

```text
...
UpdateRpcIndex
AccumulateRunningRoot
BumpHighestExecutedCheckpoint
End
```

并同步更新：

- 枚举序号和 `End`；
- `PipelineStages` 数组长度；
- pipeline 顺序测试；
- skip 路径测试。

该 stage 自身仍由 `SequenceWatch` 约束，所以 checkpoint N 的 running root
只有在 N-1 完成后才计算。不得为了并行度绕过这条顺序约束。

### 6.5 暂不盲目迁移的阶段

以下阶段先增加 histogram，不在第一批中机械迁移：

- `ExecuteTransactions`；
- `WaitForTransactions`；
- `BuildDbBatch`；
- `ProcessCheckpointData` 中的 `load_checkpoint` /
  `rpc_index.index_checkpoint`；
- `FinalizeTransactions` 中的 checkpoint accumulator。

原因：

- 交易执行现有直方图已排除分钟级 Move 执行；
- 一次现场日志没有线程栈，尚不能证明这些阶段是 120 秒调用；
- `ProcessCheckpointData` 和 `FinalizeTransactions` 持有较多带生命周期的数据，
  迁移需要额外 `Send + 'static` 审计，容易把 P0 扩成大重构。

如果新增指标显示任一阶段 active latency 超过 500ms，下一提交按完全相同的
blocking helper 迁移该阶段。若 `blocking_queue_latency` 本身持续升高，说明
Tokio blocking pool 饱和，届时再评审 checkpoint 专用有界线程池；P0 不预先
创建第二套线程池。

---

## 7. 一致性与崩溃恢复约束

迁移工作线程不能改变下列顺序：

```text
交易 effects 持久化
  -> finalized checkpoint/quarantine 持久化
  -> RPC index batch 按序 commit
  -> running root 按序累积
  -> epoch-final 特殊写入（如适用）
  -> highest_executed_checkpoint watermark 推进
```

每个箭头都代表前一步 `spawn_blocking(...).await` 已成功返回，而不是任务已经
提交给线程池。

必须保持：

- RPC index `pending_updates.pop_first()` 的 checkpoint 连续性 assert；
- running root 的 N 依赖 N-1；
- `highest_executed_checkpoint` 只在此前所有持久化工作成功后推进；
- 崩溃后允许重复执行尚未推进 watermark 的幂等步骤；
- 当前工作区中的 `rebuild_running_root_to_checkpoint` 启动恢复逻辑继续作为
  崩溃窗口防线，不把它当作在线性能修复。

禁止：

- 用 `tokio::spawn` 后不等待；
- 为了避免阻塞而提前推进 watermark；
- 捕获 DB 错误后返回成功；
- 取消 checkpoint 顺序 assert；
- 在 blocking closure 和 async caller 两边重复写同一个 watermark。

---

## 8. 提交 C：部署脚本日志与轮转

该提交位于：

```text
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/
all-in-one-deploy/localDeploy/toggle_local_rtd.sh
```

当前默认过滤器以 `off` 开始，却没有开启 `linku_metrics` 和 `typed_store`，会
屏蔽已有的：

- `Thread stalled for ...`
- `Thread stall cleared after ...`
- `very slow batch write`

建议默认增加：

```text
linku_metrics::thread_stall_monitor=warn
typed_store=warn
rtd_core::checkpoints::checkpoint_executor=warn
```

如果 slow-stage 使用独立 tracing target，则只开启该 target，避免把整个
checkpoint executor 的高频 `info` 日志重新打开。

现有 `rtd-local-node.log` 已约 3.1 GB，启动前增加日志轮转：

- 默认上限 256 MiB；
- 保留最近 3 份；
- 轮转后再记录本次启动 offset；
- PID 仍在运行时不轮转，防止移动正在写入的文件；
- 允许通过环境变量覆盖上限和保留数量。

日志轮转是运维保护，不替代 Prometheus histogram，也不能作为本次根因修复。

---

## 9. 测试计划

### 9.1 单元与 crate 测试

#### `rtd-swarm`

- [ ] 默认 validator runtime 不变。
- [ ] 默认 generic swarm fullnode runtime 不变。
- [ ] 显式 fullnode 多线程选择生效。
- [ ] validator/fullnode runtime 选择互不污染。
- [ ] dynamic fullnode 继承 swarm runtime。
- [ ] stop/start 不改变 runtime 类型。

运行：

```bash
RTD_SKIP_SIMTESTS=1 cargo nextest run -p rtd-swarm
```

#### `rtd-core`

- [ ] 每个 pipeline stage 对 checkpoint 保持严格递增。
- [ ] 新增 `AccumulateRunningRoot` 后，多个并发 checkpoint 不会越序。
- [ ] blocking helper 正确记录 queue/active latency。
- [ ] blocking closure panic 会传播，watermark 不推进。
- [ ] RPC index commit 失败时 subscription 不发送。
- [ ] 在四个迁移阶段分别注入崩溃，重启后 checkpoint 可重放且 root/index/watermark
      一致。

运行：

```bash
RTD_SKIP_SIMTESTS=1 cargo nextest run -p rtd-core checkpoint_executor
```

#### `rtd`

- [ ] `rtd start` 的 RPC fullnode 使用多线程 runtime。
- [ ] `--no-full-node` 不受影响。
- [ ] 持久化 `fullnode.yaml`、DB path 和 identity 逻辑不受影响。

运行：

```bash
RTD_SKIP_SIMTESTS=1 cargo nextest run -p rtd
```

### 9.2 人工阻塞回归测试

增加仅测试可用的 failpoint/注入点，让一个 blocking checkpoint stage 停顿
2 秒，同时在 fullnode runtime 上运行 50ms heartbeat，并请求：

- health/readiness；
- `rtd_getLatestCheckpointSequenceNumber`；
- 一个不依赖该 checkpoint 新状态的只读 RPC。

断言：

- fullnode 为多线程 runtime 时，heartbeat 最大调度间隔小于 500ms；
- RPC 不出现 2 秒级全冻结；
- 受依赖的 checkpoint 查询允许等待，但不能提前看到未提交状态；
- 解除阻塞后 watermark 只推进一次。

这个测试验证的是“同步 stage 慢时 RPC 仍有调度能力”，不能用简单
`spawn_blocking(sleep)` 代替真实 pipeline 顺序测试。

### 9.3 本地链端到端回归

使用现有持久化 localnet，不清数据库，执行：

1. 记录 chain identifier、fullnode identity、DB path、validator/fullnode
   checkpoint。
2. 正常停止，等待数小时后重新启动。
3. 等待内部 readiness 完成。
4. 连续向至少 50 个从未收款的地址各发送一笔 RTD。
5. 对其中 10 个地址立即发送第二笔，形成“首次/后续”对照。
6. 每笔均使用 `WaitForLocalExecution`，记录：
   - orchestrator/finality latency；
   - fullnode local visibility latency；
   - 所属 checkpoint；
   - pipeline stage histogram；
   - thread-stall histogram。
7. 运行至少 24 小时，期间持续产 checkpoint 并每分钟执行 RPC heartbeat。
8. 执行一次 graceful restart 和一次 SIGKILL restart，复核
   running-root/index/watermark。

### 9.4 全仓门禁

```bash
cargo fmt --all -- --check
cargo xclippy
```

涉及的长时间 nextest/simtest 按仓库约定执行；任何环境失败必须记录实际输出，
不能以“未发现失败”代替“测试通过”。

---

## 10. 验收标准

### 10.1 功能正确性

- [ ] 旧链停机数小时后启动，不删除或重建数据库即可继续转账。
- [ ] `WaitForLocalExecution` 成功后，同一 fullnode 可立即查询交易和 effects。
- [ ] 首次收款地址和已有收款地址的确认延迟分布无系统性差异。
- [ ] chain identifier、fullnode identity 和 DB path 在重启前后不变。
- [ ] 空闲时 RPC index watermark 与 highest executed checkpoint 一致。
- [ ] running root 连续，epoch 切换和崩溃恢复测试通过。

### 10.2 性能

以 24 小时 localnet soak 为准：

- [ ] `WaitForLocalExecution`：p95 `< 5s`，最大值 `< 15s`。
- [ ] validator finality：p95 `< 1s`，不得因 fullnode 改动显著回退。
- [ ] fullnode checkpoint lag：p99 `< 3s`，最大值 `< 10s`。
- [ ] thread-stall `> 500ms` 次数为 0；不得再出现 `> 10s` 停顿。
- [ ] 任一 blocking stage `execution_latency > 1s` 时都有带 checkpoint/stage
      的 warning。
- [ ] blocking queue p99 `< 100ms`；若超过则停止扩大 `spawn_blocking` 范围并
      评审专用有界线程池。

### 10.3 资源

- [ ] fullnode 多线程 runtime 不导致 validator checkpoint/finality 明显抖动。
- [ ] 24 小时内 RSS 无持续单调增长。
- [ ] 日志不会超过轮转上限乘以保留份数。
- [ ] CPU 增长能由 checkpoint/RPC 吞吐解释，不出现空闲状态持续满核。

---

## 11. 回滚策略

### 提交 A 回滚

移除 `rtd start` 上的
`with_fullnode_runtime_type(RuntimeType::MultiThreaded)` 即恢复旧行为；generic
builder 默认始终保留单线程，因此不影响其他调用方。

### 提交 B 回滚

四个 stage 可逐个恢复为同步调用。每次回滚必须同时保留原顺序，不能只删
`.await` 或留下后台 blocking task。新增 histogram 可以保留，不影响状态语义。

### 提交 C 回滚

恢复旧 `RUST_LOG` 默认值和轮转设置即可，不涉及链数据。

任何回滚均不得删除 fullnode DB、回退
`highest_executed_checkpoint` 或重新 genesis。

---

## 12. 明确拒绝的替代方案

| 替代方案 | 拒绝原因 |
| --- | --- |
| 把 SDK 查询确认从 60 秒延到 180 秒 | 只让用户等待更久，runtime 仍会冻结 |
| 默认改为 `WaitForEffectsCert` | 隐藏 stale fullnode，本地 RPC 仍查不到刚成功的交易 |
| 为第一次收款建立账户/对象预热 | Sui/RTD 地址不需要预注册，现场两笔交易结构一致 |
| 降低 state-sync retry 间隔 | 恢复后的重试 burst 是放大器，不是在线停顿起点 |
| 直接关闭 pruner | 日志显示 pruning 后 checkpoint 仍持续推进，缺少因果证据 |
| 放宽对象版本或执行校验 | 会破坏链状态安全，不是性能修复 |
| 全部 Node 默认切成多线程 | 回归面覆盖 validator、测试和 msim 调用方，超出 P0 |
| 先调 RocksDB 参数 | 当前无 flow-control stall、磁盘不足或固定写放大证据 |

---

## 13. 实施检查清单

### 提交 A

- [ ] `Node::new_with_runtime` 和只读 runtime accessor。
- [ ] `SwarmBuilder::with_fullnode_runtime_type`，默认仍为单线程。
- [ ] `rng()`、`build()`、`Swarm`、`spawn_new_node()` 完整传播字段。
- [ ] `rtd start` 仅为 RPC fullnode 选择多线程。
- [ ] runtime 角色测试通过。

### 提交 B

- [ ] 保留原 counter并新增 per-stage histogram。
- [ ] blocking queue/execution 指标与慢日志。
- [ ] `FinalizeCheckpoint` 同步持久化迁移。
- [ ] `UpdateRpcIndex` 同步 commit 迁移。
- [ ] 新增并迁移 `AccumulateRunningRoot` stage。
- [ ] `BumpHighestExecutedCheckpoint` 同步清理/水位写入迁移。
- [ ] panic、DB error、顺序和 failpoint 测试通过。

### 提交 C

- [ ] 默认开启 thread-stall、slow batch、slow stage warning。
- [ ] 256 MiB × 3 的可配置日志轮转。
- [ ] 轮转不会操作仍由活进程持有的日志。

### 最终放行

- [ ] crate 测试、fmt、clippy 通过。
- [ ] graceful/SIGKILL 两类重启通过。
- [ ] 首次/后续地址转账对照通过。
- [ ] 24 小时 soak 达到第 10 节门槛。
- [ ] 文档补充实际提交、测试命令、Prometheus 快照和交易 digest。

---

## 14. 受影响文件清单

RTD 源码：

```text
crates/rtd-swarm/src/memory/node.rs
crates/rtd-swarm/src/memory/swarm.rs
crates/rtd-swarm/src/memory/container.rs        # 预计无需逻辑修改
crates/rtd-swarm/src/memory/container-sim.rs    # 明确保留现状
crates/rtd/src/rtd_commands.rs
crates/rtd-core/src/checkpoints/checkpoint_executor/mod.rs
crates/rtd-core/src/checkpoints/checkpoint_executor/utils.rs
crates/rtd-core/src/checkpoints/checkpoint_executor/metrics.rs
```

相关实现仅作为 blocking closure 调用目标，第一版通常不需要改其内部语义：

```text
crates/rtd-core/src/authority/authority_per_epoch_store.rs
crates/rtd-core/src/rpc_index.rs
crates/rtd-core/src/global_state_hasher.rs
crates/typed-store/src/rocks/mod.rs
```

部署仓库：

```text
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/
all-in-one-deploy/localDeploy/toggle_local_rtd.sh
```

本方案与当前尚未提交的停机恢复改动存在文件交集，实施时必须在其测试通过并固定
基线后再开始，禁止覆盖或回退现有工作区修改。
