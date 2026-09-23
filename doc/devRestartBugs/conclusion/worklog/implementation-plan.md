# RTD dev restart 修复实现计划

**执行状态：** 2026-07-10 已完成全部实现与验收项；逐项红绿测试、restart、链上证据及既有测试基线限制见同目录 `README.md`。

**目标：** 移植最新 Sui 对 embedded fullnode 的持久化恢复修复，安全恢复 pending transaction，并保证三个本地部署脚本只在 fullnode 追平后报告 ready。

**架构：** `rtd genesis` 只在首次初始化时生成 fullnode 身份并把正确绝对 DB path 写入 `fullnode.yaml`；持久化 `rtd start` 读取完整 `NodeConfig` 并交给 `SwarmBuilder`，只刷新本次运行需要的监听端口。现有旧链通过确定性路径迁移把 legacy `fullnode.yaml` 指向已经选定的 `localnet-fullnode`，不删除任何 DB。pending WAL 对可重试错误保留同一 signed transaction 并重试，不覆盖 owned-object lock。部署脚本统一执行 checkpoint readiness gate。

**技术栈：** Rust、Tokio、RocksDB/typed-store、RTD Swarm、Bash/Python 启动脚本。

---

### 任务 1：移植预构建 fullnode NodeConfig（已完成）

**文件：**

- 修改：`crates/rtd-swarm/src/memory/swarm.rs`
- 测试：`crates/rtd-swarm/src/memory/swarm.rs` 内 `test` 模块

- [x] 先添加 `prebuilt_fullnode_config_preserves_identity_and_db_path` 测试，调用尚不存在的 `with_fullnode_config`。
- [x] 运行 `cargo test -p rtd-swarm prebuilt_fullnode_config_preserves_identity_and_db_path`，确认因 API 缺失而失败。
- [x] 按 Sui `eced024684` 增加 `fullnode_config: Option<NodeConfig>`、builder 传递和 `with_fullnode_config`。
- [x] 第一个 fullnode 优先使用预构建 config；其他 fullnode 和未设置 config 的所有调用保持原随机 builder 行为。
- [x] 重新运行定向测试并确认通过。

### 任务 2：让 genesis 和 start 以 fullnode.yaml 为权威配置（已完成）

**文件：**

- 修改：`crates/rtd/src/rtd_commands.rs`
- 测试：`crates/rtd/src/rtd_commands.rs` 内单元测试

- [x] 抽出纯函数测试，覆盖 legacy 相对双重路径规范化：
  `full_node_db/full_node_db/<id>` -> `<config_dir>/full_node_db/<id>`。
- [x] 添加测试：当 `<config_dir>/full_node_db/localnet-fullnode` 已存在且 persisted DB 不存在时，优先复用该稳定路径。
- [x] 运行定向测试确认两个测试因 helper 缺失而失败。
- [x] `rtd genesis` 使用真实 `rtd_config_dir` 构造 fullnode config，消除双重路径。
- [x] `rtd start` 存在 `fullnode.yaml` 时读取 `NodeConfig`；保留 key pairs 和 DB，只刷新 RPC、metrics、admin、network、p2p listen 端口及显式 data ingestion dir。
- [x] legacy DB path 发生迁移时先持久化回 `fullnode.yaml`，再只对内存副本刷新临时端口。
- [x] 没有 `fullnode.yaml`（冷启动/`--force-regenesis`）时保持原随机 fullnode 行为。
- [x] 运行相关单测和 `cargo check -p rtd`。

### 任务 3：安全恢复 pending transaction WAL（已完成）

**文件：**

- 修改：`crates/rtd-core/src/transaction_orchestrator.rs`
- 测试：`crates/rtd-core/src/transaction_orchestrator.rs` 内测试模块

- [x] 添加分类测试：`TimeoutWithLastRetriableError` 必须保留 WAL，`ValidationFailed` 才能结束 recovery record。
- [x] 运行定向测试，确认因分类 helper 缺失而失败。
- [x] recovery 对成功或永久错误调用 `finish_transaction`；对可重试错误保留 WAL，指数退避后重提完全相同 transaction bytes/digest。
- [x] 增加清晰日志，区分 `recovered`、`permanent failure`、`retrying`，不恢复前序高频诊断日志。
- [x] 运行定向测试和 rtd-core 相关测试。

### 任务 4：统一部署脚本 readiness（已完成）

**文件：**

- 审计/修改：`smartContract-rtd/all-in-one-deploy/localDeploy/deploy_local_all.sh`
- 审计/修改：`smartContract-rtd/all-in-one-deploy/localDeploy/deploy_local_release_all.sh`
- 审计/修改：`smartContract-rtd/all-in-one-deploy/localDeploy/toggle_local_rtd.sh`

- [x] 比较三个脚本的启动命令、config dir、日志和 readiness 实现。
- [x] 抽取或同步相同条件：RPC 可访问后，从本次启动日志取得 validator startup executed checkpoint，等待 fullnode RPC checkpoint 达到目标。
- [x] 不再设置 `RTD_REPAIR_ORPHAN_OBJECT_LOCKS` 或 `RTD_CLIENT_WAIT_FOR_EFFECTS_CERT`。
- [x] readiness 超时必须退出非零并明确报告当前/目标 checkpoint，不能输出 ready。
- [x] 对三个脚本运行 `bash -n`，并用静态检索验证三者语义一致。

### 任务 5：旧链无损迁移和 restart e2e（已完成）

**文件：**

- 更新：`doc/devRestartBugs/conclusion/README.md`
- 追加：`doc/devRestartBugs/conclusion/worklog/README.md`

- [x] 停止当前 `rtd start` 进程但不删除任何数据。
- [x] 备份当前 `fullnode.yaml`，让新代码把 legacy path 持久化到已存在的 `localnet-fullnode`。
- [x] 构建 debug `rtd`，使用同一 config dir 启动。
- [x] 记录 fullnode identity、DB path、validator/fullnode checkpoint、pending WAL 恢复日志。
- [x] 等待 readiness 后确认 `getObject` 与 `getCoins` 返回同一 version/digest。
- [x] 在不生成冲突新 digest 的前提下验证交易查询和一笔新的 `WaitForLocalExecution` 转账。
- [x] 再次非破坏性重启，确认 fullnode identity、DB、checkpoint 不回退且不新增随机 DB 目录。

### 任务 6：阶段一验收（已完成）

### 任务 7：补齐最终结论范围（已完成）

- [x] 实现通用旧 fullnode DB 候选的 chain id/openability/highest-executed 校验和无损选择。
- [x] 在节点内部记录 validator startup target，并汇总 fullnode object/index/pending-recovery 水位。
- [x] 对 health、transaction builder、gas coin 选择和 execute RPC 增加可重试 catching-up gate。
- [x] 添加 fullnode 落后拒绝测试以及 pending WAL + validator lock crash recovery 测试。
- [x] 重新执行 restart e2e 和三个脚本审计。

- [x] 运行局部 rustfmt 检查。
- [x] 运行新增长测试及相关 crate 测试。
- [x] 运行 `cargo build -p rtd` 和 `cargo build -p rtd --release`。
- [x] 运行 `cargo xclippy`。
- [x] 运行 `git diff --check`。
- [x] 审计工作树，确认不存在被回退的 debug 开关和 validator coin indexing 修改。
- [x] 对照用户五项要求和 conclusion 验收标准逐项写出证据。

## 计划自检

- 规格覆盖：包含最新 Sui 对比、Rust 回退、`fullnode.yaml` 持久化、pending WAL、readiness、三个脚本和实际 restart 验证。
- 安全边界：不覆盖 owned-object lock，不清除 validator/fullnode 链数据，不用 effects-cert 绕过本地执行。
- 兼容性：无 `fullnode.yaml` 的临时链不改变；已有 persisted config 进行确定性路径迁移。
- 测试顺序：所有新行为先写失败测试，再写实现。
