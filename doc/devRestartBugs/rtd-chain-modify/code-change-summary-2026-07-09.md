# RTD 本地重启状态分叉排查：代码改动汇总

日期：2026-07-09

本文档独立整理本轮排查中对项目代码、辅助脚本和诊断记录做过的改动。核心目标是定位并缓解本地 `rtd start` 重启后 RPC fullnode 返回旧 object ref，导致 validator 拒绝交易的问题。

## 结论概览

本轮改动分为三类：

1. 行为修复：让本地 `rtd start` 的 RPC fullnode 使用稳定 DB 路径，避免每次重启生成随机 fullnode DB 并从旧状态重新追赶。
2. 防误用保护：让本地重启脚本在 RPC 可连之后继续等待 fullnode checkpoint 追到 validator 启动高度，避免脚本过早报告 ready。
3. 诊断增强：在交易提交、validator 校验、object/cache/read path、coin index、checkpoint/settlement 等关键路径加 `RTD_STATE_DIVERGENCE` 日志，定位旧版本 object ref 是在哪里产生和被拒绝的。

当前最终运行态：

- 本地节点使用 `target/debug/rtd start --fullnode-rpc-port 9000` 运行。
- `~/.rtd/rtd_config/full_node_db/localnet-fullnode` 指向进度最高的可用 fullnode DB：`971eb7b41e1e`。
- 该 fullnode 仍在追赶 validator；最后记录的 checkpoint 为 `1107355`，validator 启动高度为 `1209471`。

## 行为修复：稳定 fullnode DB 路径

### `crates/rtd-swarm/src/memory/swarm.rs`

新增 `SwarmBuilder` 配置项：

- `fullnode_db_path: Option<PathBuf>`
- `with_fullnode_db_path(PathBuf)`

构建 fullnode 时，只对第一个 fullnode，也就是本地 RPC fullnode，应用显式 DB 路径：

- 目的：保持测试 swarm 默认行为不变。
- 影响面：只影响调用了 `with_fullnode_db_path` 的启动路径。

新增回归测试：

- `memory::swarm::test::rpc_fullnode_uses_configured_db_path`
- 测试不启动节点，只 build swarm 并断言第一个 fullnode 的 `NodeConfig.db_path` 等于传入路径。

### `crates/rtd/src/rtd_commands.rs`

在 `rtd start` 的 fullnode 分支中新增稳定路径：

```text
<config_dir>/full_node_db/localnet-fullnode
```

并通过 `SwarmBuilder::with_fullnode_db_path` 传给第一个 RPC fullnode。

修复前现象：

- validator DB 路径稳定，例如 `authorities_db/99f25ef61f80`。
- fullnode DB 路径由随机 fullnode key 派生，例如 `full_node_db/8c3702a4a524`、`971eb7b41e1e` 等。
- 每次重启可能创建新 fullnode DB，RPC 从很旧 checkpoint 或 genesis 开始追赶。

修复后预期：

- `rtd start` 重启会复用 `full_node_db/localnet-fullnode`。
- fullnode 不再因为随机 DB 路径而每次从旧状态重新追赶。

## 本地启动脚本防护

### `/Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/toggle_local_rtd.sh`

该文件在 repo 外，但本轮排查中有实际改动。

新增环境变量：

- `WAIT_FULLNODE_CATCHUP_TO_VALIDATOR`，默认 `1`。
- `WAIT_FULLNODE_CATCHUP_SECONDS`，默认沿用 `WAIT_RPC_SECONDS`。

新增逻辑：

- RPC 基本可连后，不立即报告 ready。
- 从本次启动后的日志中提取 validator 启动时的最高 checkpoint。
- 轮询 `rtd_getLatestCheckpointSequenceNumber`。
- 只有 fullnode checkpoint 大于等于 validator 启动高度，脚本才报告 ready。

用途：

- 防止 `rtd_getTotalTransactionBlocks` 已返回但 fullnode 仍落后时，脚本误报 ready。
- 防止客户端继续从落后 fullnode 读取旧 gas coin ref 并签出必然被 validator 拒绝的交易。

排查时如需快速启动但允许 fullnode 落后，可显式设置：

```bash
WAIT_FULLNODE_CATCHUP_TO_VALIDATOR=0 toggle_local_rtd.sh
```

## Coin Index 修复与 read path 诊断

### `crates/rtd-core/src/authority.rs`

将 coin indexing 的收集逻辑从 fullnode-only 改为 index-store 存在即可执行：

- 旧逻辑：`fullnode_only_get_tx_coins_for_indexing(...)` 会跳过 validator。
- 新逻辑：`get_tx_coins_for_indexing(...)` 只检查 `self.indexes` 是否存在。

目的：

- 本地 all-in-one 或特殊启动模式下，避免 validator 路径没有更新 coin index，导致 `rtdx_getCoins` 和 object live state 分叉。

同时增加诊断日志：

- `authority handle_transaction_impl before deny checks`
- `authority handle_transaction_impl before acquire locks`
- `authority handle_transaction_impl acquired locks`
- `authority handle_vote_transaction accepted`
- `check_transaction_validity live object comparison`

这些日志记录：

- transaction digest
- input objects
- owned objects
- live object version/digest
- provided object version/digest
- previous transaction
- `enforce_live_input_objects`

### `crates/rtd-core/src/jsonrpc_index.rs`

新增 coin index 更新日志：

- `RTD_STATE_DIVERGENCE coin index update`

记录：

- tx digest
- add/delete count
- add/delete keys

用途：

- 验证交易执行后 coin index 是否同步更新。

### `crates/rtd-json-rpc/src/coin_api.rs`

新增 `getCoins` 返回日志：

- `RTD_STATE_DIVERGENCE getCoins read path returned coins`

记录：

- owner
- coin count
- coin object id
- version
- digest
- previous transaction
- balance
- coin type
- authority cache 中同一 object 的 live version/digest/previous transaction

为支持测试 mock，将诊断用 `authority_state` 改为 `Option<Arc<AuthorityState>>`：

- 真实节点路径记录 authority cache live object。
- 测试路径可为 `None`，不依赖真实 `AuthorityState`。

### `crates/rtd-json-rpc/src/read_api.rs`

新增 `getObject` 返回日志：

- `RTD_STATE_DIVERGENCE getObject read path returned object`

记录：

- object id
- returned object ref
- object version
- object digest
- previous transaction
- authority cache live object

用途：

- 对比 `rtd_getObject` 和 authority live object 是否一致。

### `crates/rtd-json-rpc/src/governance_api.rs`

新增系统状态 read path 日志：

- `RTD_STATE_DIVERGENCE getLatestRtdSystemState read path returned system state`

用于确认 Explorer / governance 读路径看到的 epoch、protocol version、system state version 等信息。

## 交易提交与执行路径诊断

### `crates/rtd-json-rpc/src/transaction_execution_api.rs`

新增 `executeTransactionBlock` trace 日志：

- `RTD_TX_TRACE_BEGIN executeTransactionBlock`
- `RTD_TX_TRACE_END executeTransactionBlock success`
- `RTD_TX_TRACE_END executeTransactionBlock error`

记录：

- digest
- request type
- options
- orchestrator error
- local execution flag

用途：

- 将 RPC 请求入口和 transaction driver / validator 返回结果串起来。

### `crates/rtd-core/src/transaction_orchestrator.rs`

新增 orchestrator 级别日志，覆盖：

- 执行开始
- 构造 transaction driver request
- 每个执行 attempt 成功/失败
- 等待其他 attempts
- 最终成功
- 最终失败
- timeout

用途：

- 区分是 RPC 层失败、driver 失败，还是 validator 返回 non-retriable invalid transaction。

### `crates/rtd-core/src/transaction_driver/mod.rs`

新增 transaction driver 总控日志：

- driver begin/end
- tx type
- ping 标识
- timeout
- amplification factor
- retry/non-retry 分类
- submit result 分类

用途：

- 明确旧 object ref 被拒绝后为什么不会 retry。

### `crates/rtd-core/src/transaction_driver/transaction_submitter.rs`

新增 submitter 日志：

- submitter started
- selected validator
- sending request
- validator returned success
- validator returned error
- rejected transaction

记录：

- authority
- authority display name
- tx digest
- tx type
- input objects
- error

用途：

- 证明 transaction driver 发送给 validator 的就是 RPC 读到并签出的旧 object ref。

### `crates/rtd-core/src/transaction_driver/effects_certifier.rs`

新增 effects certifier 日志：

- started
- returned target
- error/success 相关信息

用途：

- 追踪交易已提交后 effects certification 是否进入后续阶段。

## Validator / execution / cache 诊断

### `crates/rtd-core/src/authority_server.rs`

新增 validator submit transaction 日志：

- submit request received
- decoded submitted transaction
- before `handle_vote_transaction`
- accepted/rejected

记录：

- epoch
- transaction count
- submit type
- submitter client addr
- tx digest
- input objects
- validator error

用途：

- 明确 validator 拒绝的是哪笔交易、哪个 object ref、当前 live version 是多少。

### `crates/rtd-core/src/execution_cache.rs`

新增 execution cache fallback 日志：

- `RTD_STATE_DIVERGENCE execution cache missing requested version, using live objref`

用途：

- 证明 validator 执行校验时，requested version 不可用，最终拿 live objref 与 requested objref 比较。

### `crates/rtd-core/src/execution_cache/writeback_cache.rs`

新增 writeback cache 日志：

- `check_owned_objects_are_live cache hit`
- `writeback cache live objref lookup`

记录：

- requested/provided object id
- version
- digest
- live version
- live digest
- previous transaction

用途：

- 判断 read path / validator path 是否读到了同一 cache live object。

### `crates/rtd-core/src/execution_cache/object_locks.rs`

新增 object lock mismatch 日志：

- `RTD_STATE_DIVERGENCE object lock live object mismatch`

用途：

- 检查旧版本 object ref 是否在 object lock 阶段就被识别。

### `crates/rtd-core/src/authority/authority_store.rs`

新增 live object marker 诊断：

- `RTD_STATE_DIVERGENCE live owned object marker missing requested version`

用途：

- 追踪 live marker 表中 requested version 不存在时的状态。

### `crates/rtd-core/src/execution_scheduler/execution_scheduler_impl.rs`

新增 settlement / execution scheduling 诊断：

- schedule settlement transactions
- wait for settlement transactions
- execution scheduling 相关状态

用途：

- 解释 validator DB 副本作为 fullnode 启动时为什么能读到 object version 106，但还会忙于补 settlement/index，无法作为完整 RPC fullnode DB 使用。

### `crates/rtd-core/src/authority/authority_per_epoch_store.rs`

新增 barrier / settlement transaction 诊断：

- wait for settlement transactions
- settlement transaction completed
- wait for barrier transaction

用途：

- 与 execution scheduler 日志配合，定位节点启动/追赶过程中是否卡在 settlement 相关任务。

## Import / fmt 产生的改动

多处文件出现 import 排序、分组和测试模块 import 调整，主要由 `rustfmt` 或前述新增日志所需依赖触发。涉及文件包括：

- `crates/rtd-core/src/authority/*`
- `crates/rtd-core/src/execution_cache/*`
- `crates/rtd-core/src/unit_tests/*`
- `crates/rtd-transaction-checks/src/lib.rs`

这些文件中不少 diff 只涉及 `use` 顺序或测试 import 补充，不改变运行时行为。

## 诊断文档与记录

### `doc/devRestartBugs/rtd-chain-modify/README.md`

持续记录了本轮排查过程：

- RPC 返回旧 gas coin ref 的复现。
- `rtdx_getCoins` / `rtd_getObject` 的对比。
- validator 拒绝旧 object version 的日志证据。
- fullnode DB 随机路径根因。
- 各个 fullnode DB 候选扫描。
- validator DB 副本实验。
- `9fWt8...` 交易证明 version 106 的来源。
- 构建、测试、clippy 验证结果。

### 本文档

本文档是独立改动汇总，便于后续 review 或拆分 commit。

## 验证命令

已执行并通过：

```bash
cargo xclippy
cargo build -p rtd
cargo test -p rtd-swarm rpc_fullnode_uses_configured_db_path
bash -n /Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/toggle_local_rtd.sh
```

关键验证结果：

- `cargo xclippy`：`Finished dev profile`
- `cargo build -p rtd`：`Finished dev profile`
- 定向单测：`1 passed; 0 failed`
- 脚本语法检查：退出码 0

## 未完全解决的问题

稳定 DB 路径修复了“每次重启换随机 fullnode DB”的问题，但不能让已经落后的 fullnode 立刻追上 validator。

当前现有可用 fullnode DB `971eb7b41e1e` 仍未执行到交易 `9fWt8oeVio9KDd9zeGA1oKpNjUcfmYAxaVRBZiRWjL1M`，所以 RPC 仍返回 gas coin version 105。validator 中该交易已经成功执行，并把 gas coin mutate 到 version 106。

因此：

- 在 fullnode 追上前，不应继续用该 RPC 发送依赖最新 gas coin ref 的转账。
- 默认启动脚本现在会阻止这种“fullnode 明显落后但脚本报告 ready”的情况。
- 若后续要进一步做链层保护，需要设计 RPC/transaction builder 在 fullnode 落后 validator 时拒绝构造或提交交易；当前 fullnode RPC 模块本身没有直接拿到同进程 validator checkpoint 水位。
