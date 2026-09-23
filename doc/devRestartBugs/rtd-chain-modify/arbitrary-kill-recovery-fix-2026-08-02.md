# RTD 任意强杀后的快速恢复修复说明

> 日期：2026-08-02
> 范围：validator consensus、checkpoint builder、内存 swarm 启动协调
> 核心约束：不依赖 drain 或安全停机；`SIGKILL`、断电和系统崩溃都走同一套恢复路径；不删除链上数据

## 1. 结论

本次故障不是简单地把启动等待时间从 300 秒改长即可解决。历史数据库中出现了：

- consensus RocksDB 已持久化到 commit `1,730,063`；
- `AuthorityPerEpochStore::last_consensus_stats` 只安全持久化到 commit `1,386,243`；
- 重启必须重新处理两者之间约 `343,820` 个 commit；
- 原 RTD 启动协调在 300 秒后杀掉仍在健康回放的节点，下一次启动又从同一安全游标开始。

差距是在停机前形成的。2026-08-01 的历史日志显示 checkpoint builder 等待
`AccumulatorSettlement(5,2772494)` effects 超过 5 秒后进入 `debug_fatal!`。本地使用
debug binary，`debug_fatal!` 会 panic；builder 又是脱离监督的后台任务，因此只有
checkpoint 流水线退出，validator consensus 仍继续运行数小时，最终形成上述巨大尾部。

修复必须同时做到：

1. checkpoint 临时缺 effects 时持续重试，不能因 debug 构建直接退出；
2. 任一关键 checkpoint 任务异常退出时，由节点监督器让整个节点 fail-stop，不能留下
   “共识仍运行、checkpoint 已死亡”的半活状态；
3. 正常运行时直接按 crash-safe 游标限制 consensus 领先量，作为监督器之外的第二道保险；
4. 重启只信任原子写入后的安全游标，不能把 consensus DB 头伪装成已处理位置；
5. 启动恢复按进度运行，不使用会反复打断健康恢复的固定 300 秒内部 deadline；
6. 已经存在的大尾部使用持久化 drain anchor 和绝对 drain ceiling；连续强杀不能在每次
   重启后重新计算或扩展额度；
7. 历史大尾部的 replay 不能随着内存 quarantine 变长而退化为二次复杂度，也不能为
   没有任何内容的 RocksDB batch 执行同步 WAL 写入。

## 2. 与 Sui 源码的关系

对照源码为：

```text
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/temp/sui
HEAD 94c67f729f870593244b368203f8456ae8246e7a
```

Sui 对应版本仍存在以下同源行为：

| 行为 | Sui 对应路径 | 归属判断 |
| --- | --- | --- |
| effects 每次只等 5 秒，失败调用 `debug_fatal!` | `crates/sui-core/src/checkpoints/mod.rs` | Sui 同源问题 |
| checkpoint builder 由 detached task 启动，没有向节点传播 panic | `crates/sui-core/src/checkpoints/mod.rs` | Sui 同源问题 |
| commit output channel 无 flow control，consensus 先持久化再交给 Sui handler | `consensus/core/src/commit_observer.rs` | Sui 同源设计缺口 |
| `pending_checkpoint_exists` 每次线性扫描全部 quarantine output | `crates/sui-core/src/authority/consensus_quarantine.rs` | Sui 同源恢复性能问题 |
| 每个 consensus output 都调用同步 `DBBatch::write()`，即使 batch 为空 | `crates/sui-core/src/authority/consensus_quarantine.rs` | Sui 同源恢复性能问题 |
| quarantine 只在 checkpoint 已执行后把 `last_consensus_stats` 写入 DB | `crates/sui-core/src/authority/consensus_quarantine.rs` | 正确的 crash-safety 约束，不能绕过 |

RTD 自身引入并放大问题的部分是 2026-07-14 增加的本地 swarm 恢复协调：它给 validator
和 fullnode 的整体恢复设置了固定 300 秒 deadline。Sui 没有这段 RTD 本地部署逻辑。

因此准确归属是：checkpoint panic、任务脱离监督、缺少 durable backpressure 以及两处
replay 热点都是继承自 Sui、且对照版本仍未完整修复的缺陷；“每 300 秒杀掉一次恢复进程”
是 RTD 本地启动协调引入的放大器。

同时核对了两个容易被误认为已修复本问题的 Sui 提交：

- `77bebabad1 Fix rare edge case in crash recovery (#24998)` 修复的是 builder summary 已落盘、
  highest executed checkpoint 尚未推进时错误清理 state hash 的边界，不处理 checkpoint task
  panic、共识 durable lag 或启动回放尾部；
- `77e74b01af Do not panic when missing effects` 调整的是 execution cache 的可失败查询接口，
  对照 HEAD 中 `wait_for_effects_with_retry` 仍然在 5 秒超时后调用 `debug_fatal!`。

所以不能只把某个已有 Sui crash-recovery patch 再移植一次；本次故障链需要单独封闭。

## 3. 必须保持的安全不变量

定义：

```text
D = 已随 AuthorityPerEpochStore 原子 batch 成功写入的最高 consensus commit
H = 当前进程内 consensus handler 已处理的最高 commit
C = consensus RocksDB 已持久化的最高 commit
```

始终满足：

```text
D <= H <= C
```

正常运行时额外限制：

```text
C - D <= 1,000 commits
```

`D` 只能在以下顺序完整成功后前移：

```text
checkpoint 已认证并执行
  -> quarantine 确定哪些 commit 已被该 checkpoint 覆盖
  -> builder summary、共享对象版本、processed messages、last_consensus_stats
     写入同一个 DB batch
  -> batch.write() 成功
  -> 通知 CommitConsumerMonitor 更新 D
```

不能直接令 `D = C`。那会跳过共享对象版本、deferred transaction、JWK/randomness
状态和 checkpoint 内容恢复，属于链状态破坏，不是性能优化。

## 4. 任意强杀恢复设计

### 4.1 运行期阻止尾部再次失控

`CommitConsumerMonitor` 同时跟踪 handler 游标和 crash-safe durable 游标。

- 本地 block proposal 在容量为 0 时暂停；
- 本地 commit 数量被裁剪到剩余容量；
- commit sync 在把 certified commit range 交给 Core 前等待 durable 容量；
- checkpoint executor/quarantine 每次 DB batch 成功后立即推进 durable 游标。

即使 checkpoint builder 没有 panic、只是长时间卡住，consensus 也最多再前进 1,000 个
commit，不会再次形成几十万 commit 的恢复尾部。

仅 RTD 的生产构造路径启用显式 durable 游标。独立使用 `consensus-core` 的测试、模拟器和
其他通用调用方没有 checkpoint quarantine，继续把其既有的 handled 回报视为 durable，
避免这项 RTD 持久化约束让通用 consensus 实例在 1,000 commit 后意外停住。

### 4.2 关键 checkpoint 任务监督

builder、aggregator 和 state hasher 都在 panic boundary 内运行。任一任务 panic 时：

1. 写入唯一的 checkpoint service failure；
2. `RtdNode` 监督器发送节点 shutdown 信号；
3. 内存 swarm container 同时监听 owner cancel 和节点内部 shutdown；
4. validator 不会在 checkpoint 已死亡后继续孤立运行 consensus。

这一机制是 fail-stop，不是“安全停机流程”。它不要求提前 drain，突然强杀时也不执行
额外动作；恢复正确性完全来自已落盘的原子状态。

### 4.3 effects 等待改为真正可重试

checkpoint builder 每 5 秒输出一次简明告警并继续等待 effects，不再在 debug binary 中
panic。builder 的普通 `Retry` 错误同样等待 1 秒后重试。

### 4.4 启动回放与 checkpoint 恢复并行

consensus monitor 在打开并扫描完整 consensus commit DB 之前发布。checkpoint builder
只等待 replay 到达上一次 crash-safe 游标，然后与剩余历史 replay 并行处理；readiness
仍要等完整 replay 和最后一次 builder drain 完成，不能提前向 RPC 报告可用。

### 4.5 旧数据库的持久化固定 drain ceiling

已有数据库可能在本修复部署前就满足 `C - D > 1,000`。如果直接套用正常运行限额，
consensus 无法提交 checkpoint signature，而 `D` 又必须等 checkpoint 认证后才能前移，
会形成循环等待。

最初曾尝试仅把历史 consensus head 持久化为 `A`，并允许 `A + 1,000`。真实旧库实测证明
这个额度不充分：新增 1,000 个 commit 虽然把 `D` 从 `1,386,243` 推进到了约
`1,392,465`，但历史 output 产生的 checkpoint signature 以及这些 signature commit 自身
进入安全 checkpoint 还需要更多共识轮次；额度耗尽后仍会形成循环等待。

修正版首次检测到超大历史尾部时，在同一个 RocksDB 原子 batch 中持久化：

```text
A = 首次迁移时观察到的 consensus head
L = A - D                         # 修复前遗留尾部
B = max(2 × L, 1,000)            # 一次性迁移预算
E = A + B                         # 绝对 drain ceiling
```

完整有序回放入队后，允许的最高新 consensus head 为：

```text
max(D + 1,000, E)
```

其中 `A` 和 `E` 分别保存在当前 epoch 本地数据库的
`consensus_recovery_drain_anchor`、`consensus_recovery_drain_ceiling` 表中。`2 × L` 是有限的
一次性兼容预算：第一段覆盖已有 output，第二段覆盖由其产生的 signature commit；正常
运行期仍只允许 `D + 1,000`，并不会长期放宽安全窗口。

`A` 和 `E` 都不会在重启时改成新的 `C`，因此：

- drain 提交可承载 checkpoint signature，解除已有尾部的恢复依赖；
- 在 drain 完成前反复 `SIGKILL`，绝对上限仍是同一个 `E`，不会每次重启续杯；
- `E` 只在迁移期有效：完整 replay target 已 durable，且 Core 当前头与已经获准进入 Core
  的 commit-sync range 都满足 `head - D <= 1,000` 后，在 Core 串行边界一次性停用；
- 停用迁移额度与 commit-sync range 的准入由同一把门锁串行化，不存在“range 已按 `E`
  获准、但到达 Core 前 `E` 被关闭”的竞态；停用后本进程不会重新开启；
- 下一次启动发现 `D >= A` 且现有差距已回到正常窗口后，会原子删除 anchor 和 ceiling。

兼容已经运行过早期 anchor-only 版本的数据库：保留原 `A`，只计算并持久化一次缺失的
`E`；后续启动直接复用，不能再次计算。

真实末段恢复还发现了一个容易遗漏的退出条件：不能用“本次启动扫描到的旧 consensus
target”判断何时停用 `E`。扫描完成后，为认证历史 checkpoint 而产生的新 signature commit
会使 live consensus head 高于旧 target。如果在 `D + 1,000` 追到旧 target 时提前停用 `E`，
新 head 仍在正常窗口之外，签名提交会再次循环等待。最终退出条件因此同时要求：启动
target 已 durable，以及 Core 可见的实际头（含 commit-sync 已预留 range）已回到正常窗口。
这时在进程内关闭 `E`，之后即使 checkpoint 再次卡住，也只能新增 1,000 个 commit；持久化
的迁移记录则在下一次启动读取真实 `C` 后原子清理。单元测试同时覆盖旧 target 到达但
live head 仍过高时不得退出、回到正常窗口后必须退出、退出后不得重开三种场景。

### 4.6 历史 replay 热点改为有界操作

真实旧库恢复期间的 macOS sampling 找到两个不改变共识结果、但会显著拖慢 replay 的路径：

1. `pending_checkpoint_exists` 原来遍历 `output_queue`。历史尾部达到 17 万条时，每生成一个
   稀疏 checkpoint root 都会扫描整条队列，整体趋近 `O(number_of_commits²)`。现在 quarantine
   同步维护内存 `BTreeSet<CheckpointHeight>`，查询为 `O(log number_of_checkpoints)`；索引仅由
   replay 确定性重建，不新增 DB 或协议字段。
2. `push_consensus_output` 会在每个 commit 后尝试落盘。没有已执行 checkpoint 可释放时，
   `commit_with_batch` 不会向 batch 写入任何内容，但原实现仍调用 `DBBatch::write()`；该接口
   固定 `sync=true`，会为一个空 batch 触发 WAL 同步。现在只有存在
   `sequence_number <= highest_executed_checkpoint` 的 builder summary 时才创建并写入 batch。

采样复核中，第一处热点从 365 个 top-of-stack 样本降到 2 个，空同步写路径从 59 个样本降到
1 个。两项都不前移 `D`、不跳过 replay，也不改变 quarantine 的原子提交边界。

### 4.7 不再用内部固定总超时杀恢复

`rtd-swarm` 移除 300 秒总恢复 deadline，改为每 5 秒输出阶段进度。外层脚本可以选择
多久停止等待，但不能因为命令行等待超时就杀掉仍在推进安全游标的 RTD 进程。

## 5. 代码范围

| 文件 | 作用 |
| --- | --- |
| `consensus/core/src/commit_consumer.rs` | durable 游标、1,000 commit 窗口、固定 drain ceiling 约束、恢复进度 |
| `consensus/core/src/core.rs` | proposal 和本地 commit 的 durable backpressure |
| `consensus/core/src/commit_syncer.rs` | 远端 commit sync 的 durable backpressure |
| `consensus/core/src/commit_observer.rs` | 暴露恢复目标和完整 replay 状态 |
| `consensus/core/src/storage/rocksdb_store.rs` | 启动前只读获取 consensus head |
| `crates/rtd-core/src/authority/consensus_quarantine.rs` | DB batch 成功后返回最高 durable commit；checkpoint height 内存索引；跳过空同步 batch |
| `crates/rtd-core/src/authority/authority_per_epoch_store.rs` | durable monitor 连接，anchor/ceiling 原子持久化与迁移 |
| `crates/rtd-core/src/consensus_manager/mod.rs` | 提前发布 monitor、准备固定恢复边界 |
| `crates/rtd-core/src/checkpoints/mod.rs` | effects 重试、任务 panic 监督、并行恢复和启动完成门槛 |
| `crates/rtd-node/src/lib.rs` | checkpoint 关键任务失败时关闭节点 |
| `crates/rtd-swarm/src/memory/container.rs` | 响应节点内部 shutdown |
| `crates/rtd-swarm/src/memory/swarm.rs` | 进度驱动恢复，移除 300 秒内部 deadline |

## 6. 恢复时间边界

部署本修复后，健康运行期间任意时刻强杀留下的 consensus 尾部最多为 1,000 个 commit。
按本机优化后的历史旧库回放速度约 300 commit/s，正常 1,000 commit 上限的纯 replay 约为
3--5 秒量级；加上
RocksDB 打开、checkpoint 重建和 fullnode catch-up，正常目标是几十秒内恢复，而不是分钟至
小时。最终端到端耗时以第 9 节的硬杀复测为准。

当前这份旧数据库已经带有约 343,820 commit 的修复前历史债务。第一次迁移仍必须完整、
有序处理它，不能通过伪造游标实现秒级恢复；优化后完整扫描 `338,608` 个 commit 的最终
实测约为 18 分 36 秒（另一轮 `343,830` 个 commit 为 18 分 53 秒），且速度不再随队列
长度恶化。恢复过程也不再被 300 秒反复杀掉，
持久化固定 ceiling 防止连续强杀继续扩大债务。第一次成功追平后，后续强杀进入上述有界
快速路径。

## 7. 验证要求

必须至少覆盖：

1. durable 容量耗尽会阻塞，DB batch 成功推进 durable 后解除；
2. replay 只有在 scan 完成且 handler 到达目标后才宣布完成；
3. 超大旧尾部的 anchor 和绝对 ceiling 原子持久化；用更高的新 startup head 模拟再次
   重启，ceiling 必须保持逐值相等；
4. ConsensusManager 可在全新 DB 上启动、停止并再次启动；
5. checkpoint builder 在启动完成前退出会被 swarm 报错；
6. 在真实保留数据库上启动，观察 replay target、processed commit 和 durable commit 持续推进；
7. 恢复后直接 `SIGKILL`，不运行 stop/drain；再次启动并确认 replay 尾部保持在有界窗口；
8. fullnode readiness 完成后执行转账，确认链可正常提交和查询。

## 8. 明确排除的方案

- 只延长 300 秒 timeout：会掩盖半活状态，不能阻止下次再形成巨大差距；
- 安全停机/drain：无法覆盖断电和 `SIGKILL`，不符合目标；
- 启动时把 `last_consensus_stats` 直接写成 consensus DB head：破坏 quarantine 的原子性；
- 删除 consensus 或 epoch DB：违反保留链数据的要求；
- checkpoint builder panic 后只打印日志：正是本次五小时差距形成的直接原因。

## 9. 真实保留数据库验证结果

验证使用同一份历史数据库，全程未删除 consensus、epoch、checkpoint、fullnode 或 index
数据；所有中断均为 `SIGKILL`，没有调用 `toggle_local_rtd.sh stop` 或任何 drain 动作。

### 9.1 连续强杀不会扩展迁移上限

第一次修正版启动读取到：

```text
D = 1,392,465
C = 1,731,063
A = 1,730,063
E = 2,408,259
```

恢复尚未完成时直接强杀；下一次启动再次逐值读取到相同的 `A=1,730,063`、
`E=2,408,259`，没有以新 `C` 重算。第二次完整扫描范围为 `1,392,456..=1,731,063`，
共 `338,608` 个 commit，于 2026-08-02 19:21:42 开始、19:40:18 完成，约 18 分 36 秒。

末段实测暴露并修复了 4.5 节所述的旧-target提前退出问题。修复后二次强杀现场为：

```text
D = 1,730,071
C = 1,784,738
A = 1,730,063
E = 2,408,259
replay = 54,677 commits
```

固定 `E` 再次保持不变。该小尾部在存储打开后约 64 秒完成完整启动回放，核心扫描区间约
13 秒；validator checkpoint drain 完成后，fullnode 只剩 24 个 checkpoint，约 0.7 秒追平。
脚本在内部 readiness 阶段 218 秒返回 HTTP 200，随后顺利从 epoch 5 进入 epoch 6。

### 9.2 健康运行期任意强杀恢复

epoch 6 健康运行时 quarantine 仅 5。直接 `SIGKILL` 后，使用未附加任何等待时间或日志
参数的原命令：

```bash
./toggle_local_rtd.sh start
```

默认 800 秒配置下，脚本在内部 readiness 阶段 6 秒返回 HTTP 200；包含脚本固定的启动后
3 秒进程检查，端到端约 9 秒。链 ID、epoch 和 checkpoint 数据均保留。

### 9.3 转账可用性

恢复后从
`0xc535a846ad8aecf2c353c12b557612f0f1ae3bb09ba7cd2c6c8fa6fa56bf0df9`
向
`0xfbc95a1fbdd68117b26163e4068da6befc2fde9b02cb570b2351237bec447ba3`
转账 `888 RTD`：

```text
digest = FoKLnyG1zZc2xPEpxVo73Kwyxh2182WRs5KGKxMnYFN3
status = success
confirmedLocalExecution = true
checkpoint = 1,520,927
CLI confirmation time = about 0.81 seconds
destination balance = 1,588 RTD -> 2,476 RTD
```

转账后 `/health?verbose=true` 仍返回 HTTP 200。

### 9.4 最终源码审计与自动化验证

9.1--9.3 的真实旧库恢复完成后，最终审计又补上了 4.5 节所述的迁移额度进程内退出条件，
以及 commit-sync 准入与退出之间的并发门锁。旧数据库已经完成不可逆的正常追平；为了遵守
不删除、不回滚链数据的约束，没有人为重造 34 万 commit 的历史债务。该末段边界使用
确定性单元测试覆盖，真实正常窗口的 `SIGKILL` 结果仍由 9.2 节覆盖。

最终生产二进制：

```text
path = target/debug/rtd
sha256 = 7649cbdafe1d26569ac36f983fddc5a5cab4c01d016caafc6823766d8f692910
size = 266,775,768 bytes
built_at = 2026-08-02 21:55:34 +0800
```

最终验证结果：

| 验证 | 结果 |
| --- | --- |
| `cargo test -p consensus-core --lib commit_consumer` | 4 passed |
| `cargo test -p consensus-core --lib test_recover_and_send_commits` | 1 passed |
| `cargo test -p rtd-core --lib authority::authority_per_epoch_store::consensus_quarantine::tests` | 2 passed |
| `cargo test -p rtd-core --lib consensus_manager_tests::test_consensus_manager` | 1 passed |
| `cargo test -p rtd-core --lib global_state_hasher::tests` | 3 passed |
| `cargo test -p rtd-core --lib node_readiness` | 9 passed |
| `cargo test -p rtd-rpc-api --lib service::health::tests` | 4 passed |
| `cargo test -p rtd-swarm --lib checkpoint_builder_shutdown_before_recovery_is_reported` | 1 passed |
| `cargo build -p rtd` | passed |
| `cargo xclippy` | passed；仅输出仓库既存 warning |
| `git diff --check` | passed |

最终复核时，保留数据库上的进程仍在运行，`/health?verbose=true` 返回 HTTP 200，epoch 为 6，
checkpoint 持续推进。
