# RTD dev restart 修复工作日志

本目录按时间顺序记录本轮修复中的调查证据、设计决策、文件编辑、测试命令和结果。记录只追加或勘误，不用后续结论覆盖早期事实。

## 2026-07-10

### 开始执行

- 用户要求按顺序完成：最新 Sui 对比、回退不符合结论的 Rust 改动、基于 `fullnode.yaml` 持久化复用 fullnode、落实结论中的全部修复、审计三个部署脚本。
- 当前仓库已有大量未提交修改，视为前序排查产物；本轮会逐项审计，不会整体重置或覆盖无关用户改动。
- 已确认执行方法：先完成最新 Sui 证据对比，再决定实现；所有行为改动遵循 TDD，先看到回归测试按预期失败。
- 已确认设计基线：用户已经审阅 `conclusion/README.md` 并明确要求围绕该结论执行，因此该文档作为已批准设计；若最新 Sui 有更直接的定向修复，将先更新结论再实现。

### 当前任务计划

1. 读取最新 Sui 的相关源码和提交历史。
2. 更新最终结论与实现计划。
3. 回退与最终方案冲突的 Rust 改动。
4. 为 fullnode 持久化、readiness、pending transaction 恢复编写失败测试。
5. 实现并验证代码。
6. 审计和更新部署脚本。
7. 执行 restart e2e、定向测试、构建、clippy 和完成审计。

### 最新 Sui 对比：发现同根因定向修复

- 对比仓库：`/Users/changzechuan/WenchuanProjects/RTD-Blockchain/temp/sui`。
- 当前版本：`sui_v1.76.0_1783574598_ci`，HEAD `b76bcd0e2bc3551fda92dfff1eeaea75622061ab`。
- 找到定向修复提交：`eced02468444d429a4e9a2b9622b7bd30a1710d4`，提交标题：
  `fix(sui): sui start resumes the embedded fullnode (index over the fullnode gRPC) (#26884)`。
- 上游提交描述的故障与 RTD 现场一致：持久化 validator 从 `authorities_db` 恢复，但 embedded RPC fullnode 每次生成新身份、新 DB，从 genesis 重新同步；`fullnode.yaml` 生成后从未被 `sui start` 读取；每次启动泄漏一个 `full_node_db/<key>`。
- 上游根因还确认了 genesis 的双重路径错误：
  `with_config_directory(FULL_NODE_DB_PATH.into())` 后 builder 再追加 `FULL_NODE_DB_PATH`，形成
  `full_node_db/full_node_db/<key>`。
- 上游修复只改两个核心文件：
  - `crates/sui/src/sui_commands.rs`：存在 `fullnode.yaml` 时读取 `NodeConfig`，保留身份和 DB，只刷新易冲突的监听端口与 RPC 配置；genesis 使用真实 config dir。
  - `crates/sui-swarm/src/memory/swarm.rs`：新增 `with_fullnode_config(NodeConfig)`，第一个 fullnode 使用预构建配置，不再随机 build。
- 冷启动或 `--force-regenesis` 没有持久化 `fullnode.yaml` 时，仍生成新身份；这与用户要求完全一致。
- 最新 Sui 没有放宽或覆盖 `ObjectLockConflict`；因此 RTD 的 `RTD_REPAIR_ORPHAN_OBJECT_LOCKS` 仍应撤销。
- 最新 Sui 的 `transaction_orchestrator` 仍会在 pending recovery 尝试后调用 `finish_transaction`，未提供针对 RTD 现场“retriable transport error 后 WAL 被删”的定向修复；该部分需要在 RTD 中按现场证据单独修复。
- 最新 Sui 另有 `9c47238288`、`291915754e` 两个 health-check 改进，要求 embedded live-object index 不能明显落后 executed checkpoint。RTD 当前版本没有相同 rpc-store cohort 架构，不能机械 cherry-pick，但其“未就绪不得报 healthy”原则会纳入 RTD readiness 设计。

### 当前 Rust 改动审计与回退决策

- `git diff --name-only -- '*.rs'` 共列出 43 个 Rust 文件，均与前序 dev restart 排查记录中的修改清单一致。
- 明确与最终结论冲突的行为改动：
  - `authority.rs`：让 validator 参与 JSON-RPC coin indexing。
  - `object_locks.rs`：`RTD_REPAIR_ORPHAN_OBJECT_LOCKS` 覆盖不同 digest 的 owned-object lock。
  - `wallet_context.rs`：`RTD_CLIENT_WAIT_FOR_EFFECTS_CERT` 绕过 local execution 确认。
  - `swarm.rs` / `rtd_commands.rs`：只固定 DB path，没有复用完整 `fullnode.yaml` NodeConfig。
- 其余修改为 `RTD_STATE_DIVERGENCE`/`RTD_TX_TRACE` 高频诊断日志和 rustfmt/import 排序噪声；根因已经由现场和最新 Sui 定向提交确认，正式修复不再需要这些广泛运行时改动。
- 回退策略：把这 43 个 Rust 文件全部恢复到当前 `HEAD`，保留所有 `doc/devRestartBugs` 文档；随后按 TDD 只重新引入最新 Sui 的 fullnode resume 方案、RTD 必需的 readiness 和 pending recovery 改动。
- 已执行 Rust 回退，命令退出码为 0；随后 `git diff --name-only -- '*.rs'` 为空，确认没有遗留旧 Rust 行为改动。
- 详细实现步骤已写入 `worklog/implementation-plan.md`。

### TDD 任务 1：预构建 fullnode 配置（红灯）

- 新增测试：`prebuilt_fullnode_config_preserves_identity_and_db_path`。
- 首次运行：`cargo test -p rtd-swarm prebuilt_fullnode_config_preserves_identity_and_db_path`。
- 红灯包含目标失败：`SwarmBuilder` 不存在 `with_fullnode_config`。
- 同时发现测试错误地对不实现 `Clone` 的 `NetworkConfig` 调用了 clone；已移除这个与目标无关的前提，准备重新确认纯净红灯。
- 第二次运行得到纯净红灯：唯一错误为 `SwarmBuilder` 不存在 `with_fullnode_config`。
- 已按最新 Sui `eced024684` 写入最小实现：builder 保存可选 `NodeConfig`，第一个 fullnode 优先使用该 config；未传 config 和其他 fullnode 的随机生成行为不变。
- 绿灯命令：`cargo test -p rtd-swarm prebuilt_fullnode_config_preserves_identity_and_db_path`。
- 结果：退出码 0；`1 passed; 0 failed; 2 filtered out`。预构建 fullnode 的 protocol identity 和 DB path 均保持不变。

### TDD 任务 2：权威 fullnode.yaml 与 legacy DB 路径（测试已写）

- 已重新读取 `rtd_commands.rs`、`FullnodeConfigBuilder` 和上游 `eced024684` diff；RTD 当前 `start` 仍只读取 `network.yaml`，`genesis` 仍把 `FULL_NODE_DB_PATH` 当作 config dir。
- 已只读核对当前 `~/.rtd/rtd_config/fullnode.yaml`：其 DB 仍为 `full_node_db/full_node_db/b346e1e8d767`；现场稳定入口 `full_node_db/localnet-fullnode` 是指向已选 DB `971eb7b41e1e` 的 symlink。
- 添加两个先行测试，分别要求规范化双重相对路径，以及 persisted DB 缺失时优先复用已存在的 `localnet-fullnode`。此时生产 helper 尚不存在，下一步运行定向测试确认预期红灯。
- 红灯命令：`cargo test -p rtd normalizes_legacy_doubled_fullnode_db_path`。
- 结果：退出码 101；唯一目标错误为 `resolve_persisted_fullnode_db_path` 尚不存在。首次编译 `rtd` 测试依赖耗时较长，但没有掩盖目标失败。
- 已写入最小路径 helper，并移植上游 start/genesis 核心：读取完整 `NodeConfig`、迁移并回写 DB path、刷新运行期监听端口/RPC、交给 `with_fullnode_config`；genesis 改用真实 config dir。
- 绿灯命令：`cargo test -p rtd fullnode_db`。
- 结果：退出码 0；两个目标测试均通过，其他测试目标因 filter 未运行。
- 编译检查：`cargo check -p rtd`，退出码 0，耗时 4 分 19 秒。
- 局部 `rustfmt --check` 发现两个已修改 Rust 文件需要标准 import/换行格式；下一步只格式化这两个文件并复检，不触碰其他脏工作树文件。
- 已只格式化 `rtd_commands.rs` 和 `swarm.rs`，随后相同局部 `rustfmt --check` 退出码 0。

### TDD 任务 3：pending transaction WAL 安全恢复（测试已写）

- 根因数据流：`start_task_to_recover_txes_in_log` 从 WAL 取出原 signed transaction，调用一次带 60 秒 timeout 的 `TransactionDriver::drive_transaction`，随后不区分结果就执行 `finish_transaction`；因此 transport/validator unavailable 导致的 `TimeoutWithLastRetriableError` 也会永久删除 recovery record。
- 错误分类证据：`TimeoutWithLastRetriableError` 映射到 `ErrorCategory::Unavailable`，属于可重提；`ValidationFailed` 映射到 `InvalidTransaction`，属于永久失败。正常前台路径也用 `is_submission_retriable` 区分后台重试。
- 假设：recovery 应沿用 TransactionDriver 的 submission-retriable 分类；可重试错误必须保留同一 WAL record 并用同一 transaction bytes/digest 退避重提，成功或永久错误才清理。
- 已添加两个先行分类测试；生产 helper 尚不存在，下一步运行定向测试确认预期红灯。
- 红灯命令：`cargo test -p rtd-core recovery_`。
- 结果：退出码 101；唯一目标错误为 `should_retry_recovered_transaction` 尚不存在。
- 已实现分类 helper，并把一次性 recovery 改为循环：每次仍提交同一个 `Transaction`，可重试错误按 1 秒起、60 秒封顶退避且不调用 `finish_transaction`；成功或永久错误退出循环后才清理 WAL。日志明确区分 recovered、retrying 和 permanent failure。
- 绿灯命令：`cargo test -p rtd-core recovery_`。
- 结果：退出码 0；两个目标测试通过，`2 passed; 0 failed; 618 filtered out`。

### TDD 任务 4：三个部署脚本 readiness（审计与测试已写）

- 脚本实际位于相邻目录 `../smartContract-rtd/all-in-one-deploy/localDeploy`，该目录不属于 Git worktree，且其上级没有额外 `AGENTS.md`/`CLAUDE.md`。
- `deploy_local_all.sh` 和 `deploy_local_release_all.sh` 目前只等待任意 RPC result 就继续发布合约，没有 checkpoint gate。
- `toggle_local_rtd.sh` 已有初版 gate：按本次日志 offset 读取 validator `executed==certified` 的 startup checkpoint，再查询 fullnode RPC；但可以用 `WAIT_FULLNODE_CATCHUP_TO_VALIDATOR=0` 绕过，超时信息也没有输出最终 current/target。
- release 脚本默认 `RUST_LOG=off,rtd_node=info`，不会稳定产生 gate 所依赖的 `rtd_core::authority::backpressure` startup 行；三个脚本的启动参数和日志策略也不一致。
- 三个脚本当前 `bash -n` 均退出码 0；静态检索确认没有 `RTD_REPAIR_ORPHAN_OBJECT_LOCKS` 或 `RTD_CLIENT_WAIT_FOR_EFFECTS_CERT`。
- 已添加共享 readiness helper 的先行 shell 测试，覆盖 startup target 提取、追平成功和超时必须非零且报告 current/target。helper 尚不存在，下一步运行测试确认预期红灯。
- 红灯命令：`bash test_local_rtd_readiness.sh`。
- 结果：退出码 1；预期失败为共享 `local_rtd_readiness.sh` 尚不存在。
- 已新增共享 helper 并让三个脚本统一调用：RPC 可访问后，必须读取本次启动日志的 validator checkpoint，并等待 fullnode RPC checkpoint 达标；删除 toggle 的绕过开关，所有超时均返回非零并包含 current/target。
- 三个脚本统一了 checkpoint gate 所需的最小 `RUST_LOG`，避免 release 缺少 startup 行，也停止默认启用此前产生超大日志的广泛高频模块日志。
- 首次绿灯运行仍失败；`bash -x` 定位到共享 helper 在 catch-up 返回 1 后无条件 `return 0`，仅在调用者 `set -e` 时偶然表现正确。已改为显式传播 catch-up 状态，确保 helper 自身语义正确。
- 绿灯命令：`bash test_local_rtd_readiness.sh`；结果退出码 0，三个 readiness 测试通过。
- `bash -n` 分别检查三个部署脚本、共享 helper 和测试脚本，五个命令均退出码 0。
- 静态检索确认三个入口都 source 同一 helper、都调用 `wait_for_rtd_readiness`、都启用相同 startup 日志，并且没有 readiness 绕过变量或两个已回退 debug 环境变量。环境未安装 `shellcheck`，因此无法运行该附加检查。

### 任务 5：当前链无损 restart e2e（重启前基线）

- 当前旧进程 PID `49183`，命令为 `target/debug/rtd start --fullnode-rpc-port 9000`，已连续运行约 3 小时 39 分。
- chain identifier 为 `4099c63d`；RPC fullnode checkpoint 为 `1394488`，本次 validator startup checkpoint 为 `1418995`，仍在追赶。
- `fullnode.yaml` protocol key value 为 `EoZJgUr3VNU5jiT1mYc9WZUinSIKO6Ieg7GsOMo/38c=`，legacy DB path 为 `full_node_db/full_node_db/b346e1e8d767`；稳定 symlink `localnet-fullnode` 指向 `971eb7b41e1e`。
- 重启前 `rtd_getObject` 与 `rtdx_getCoins` 对目标 gas coin 一致：version `106`、digest `3dntckSuZt17DYCn5CRdKwfc18iyJxm5ATZC4JbiyCVX`。交易 `CzQbQQ...` 尚不能从落后 RPC 查询。
- 稳定 DB 的 `live/fullnode_pending_transactions` RocksDB 存在且包含 SST/log 文件，说明 restart 必须复用该 DB 才能恢复 WAL。
- `full_node_db` 顶层共有 15 个入口。后续比较此数量，确认不再生成随机 DB 目录。
- 新代码构建命令：`cargo build -p rtd`；退出码 0，耗时 3 分 22 秒。下一步备份配置并 graceful stop，不删除任何链数据。
- 已备份 `fullnode.yaml.pre-resume-fix-20260710-1810`，内容与迁移前配置完全一致；随后通过 `toggle_local_rtd.sh` 向 PID `49183` 发送 SIGTERM，脚本退出码 0，进程 graceful stop。DB 入口仍为 15 个。
- 第一次新代码启动创建 PID `89367`，并把 `fullnode.yaml` DB 原子语义回写为绝对稳定路径 `/Users/changzechuan/.rtd/rtd_config/full_node_db/localnet-fullnode`；protocol key 不变，DB 入口仍为 15 个。
- 现场发现 readiness parser 的新边界：本次 validator startup 行为 `executed=1468047 certified=1468048`，两者相差 1；fullnode 为 `executed=1395674 certified=1468045`。旧 parser 只接受相等值，导致 target unavailable。已中止外层等待脚本（PID `89367` 节点继续运行），并先把该真实边界加入测试；生产 parser 尚未修改。
- 边界测试红灯：`bash test_local_rtd_readiness.sh` 退出码 1。已把 parser 改为取本次所有 startup 行的最大 executed checkpoint；embedded fullnode 不可能领先同进程 validator，因此该值正是需要等待的 validator startup target，且不依赖 certified 恰好相等。
- 边界修复绿灯：`bash test_local_rtd_readiness.sh` 退出码 0；helper/test 的 `bash -n` 同样退出码 0。用本次真实日志 offset 验证 parser 输出 target `1468047`。
- PID `89367` 在外层等待中断后保持运行；protocol key、绝对稳定 DB path 和 15 个 DB 入口均保持不变。startup recovery 日志为 `Recovering 0 pending transactions` / `Recovered 0 out of 0`，证明复用的 stable DB WAL 已被打开且当前没有有效 recovery record。
- 第一次 readiness 起点约为 fullnode `1396058` / validator target `1468047`。追平速率约每秒 5 个 checkpoint，因此改用 60 秒轮询和 5 小时硬超时继续等待；超时仍会非零失败，不会提前报告 ready。
- Debug 节点运行约 31 分钟后推进到约 `1404325`，但空 checkpoint 生成使 validator 也持续前进，debug fullnode 追平预计仍需数小时，且会让后续 WaitForLocalExecution e2e 长期看不到新交易。
- 为完成真实 catch-up 而不修改 DB 或放宽 readiness，额外执行 `cargo build -p rtd --release`；退出码 0，耗时 12 分 14 秒。仅出现外部 Move crates 的既有 unused/dead-code warnings。下一步 graceful stop debug 进程并用同一代码的 release binary 继续同一 DB。
- 已 graceful stop debug PID `89367`，随后用新构建的 release binary 启动 PID `77258`；仍使用相同 protocol key、stable DB 和 15 个 DB 入口。
- Release 第一次 RPC checkpoint `1407992`，validator startup target `1476172`；约 9 分钟后 readiness 在 `current_checkpoint=1476583 >= target_checkpoint=1476172` 时才返回成功。
- Release 随后追到 live 速率（30 秒从 `1479231` 到 `1479374`，约 4.8 checkpoint/s，CPU 从 catch-up 高负载降到约 24%）。
- ready 后旧交易 `CzQbQQB8W4krhwthGKxPxDKLfSAskGYrtzV43yhsNacW` 可查询且 effects success；`rtd_getObject` 与 `rtdx_getCoins` 对 gas coin 一致返回 version `107`、digest `Heo8EvVSR8o3brZxKGpSfT2jWRPWA9cfVaDR6ZwZSBBZ`。
- 未设置任何 debug 环境变量，先固定新交易 digest `9hhXyEuo2nHAvs7JtLsV7zQu4BH1HsFTz1LA7dmeUcqa`，再执行 1,000,000 mist 转账。CLI 退出码 0，`confirmedLocalExecution=true`，effects success，checkpoint `1479483`，gas coin 推进到 version `108` / digest `H82RJiu8hJSFrXe6wzfFiKpHPrSGNVhFKAm4i1vHn8fK`。
- 交易后立即查询 transaction/object/getCoins 均成功且 version/digest 一致；DB 入口仍为 15。下一步用 SIGKILL 模拟 crash，再用同一 config/DB 第二次启动验证 RocksDB/WAL 恢复、identity 和 checkpoint 不回退。
- 已对 release PID `77258` 发送 SIGKILL，确认进程退出且未删除/移动任何 DB；随后用相同 release binary/config 启动 PID `8004`。
- crash restart 的 validator 与 fullnode startup executed checkpoint 都是 `1479754`；readiness 首次检查即以 `current_checkpoint=1479758 >= target_checkpoint=1479754` 成功。startup pending recovery 再次为 0/0，无 panic/error。
- 第二次启动后 chain identifier 仍为 `4099c63d`，RPC checkpoint 继续到 `1479909`；protocol key 和 stable DB path 不变，DB 入口仍为 15，没有随机目录泄漏或 checkpoint 回退。
- 新交易 `9hhX...` 仍可查询 success；object/getCoins 均保持 version `108`、digest `H82RJiu8hJSFrXe6wzfFiKpHPrSGNVhFKAm4i1vHn8fK`。当前链由 release PID `8004` 正常运行。

### 任务 6：最终验收

- 最终定向测试全部退出码 0：
  - `cargo test -p rtd-swarm prebuilt_fullnode_config_preserves_identity_and_db_path`：1 passed。
  - `cargo test -p rtd fullnode_db`：2 passed。
  - `cargo test -p rtd-core recovery_`：2 passed。
- `cargo build -p rtd` 退出码 0；`cargo xclippy` 退出码 0，完整 workspace 检查完成。
- 三个已修改 Rust 文件的局部 `rustfmt --check` 退出码 0；`git diff --check` 退出码 0。
- `cargo fmt --all -- --check` 退出码 1，但输出是仓库 HEAD 中大量未触碰文件的既有 import 排序差异（13,975 行 diff）；本轮没有据此格式化或改写无关文件。
- readiness shell 测试再次通过；三个部署脚本、共享 helper 和 shell 测试的 `bash -n` 均通过，且没有行尾空白。环境仍未安装 `shellcheck`。
- 最终 Rust diff 仅有三个目标文件：`transaction_orchestrator.rs`、`swarm.rs`、`rtd_commands.rs`。静态审计未发现 `RTD_REPAIR_ORPHAN_OBJECT_LOCKS`、`RTD_CLIENT_WAIT_FOR_EFFECTS_CERT`、`RTD_STATE_DIVERGENCE`、`RTD_TX_TRACE` 或 validator coin-indexing 改动。
- 追加链上回归：checkpoint `1479483` 可查询且含 11 笔交易；system state 返回 epoch `5` / protocol version `105`；transaction index 可分页；event query 成功返回空页；以 immutable shared Clock `0x6` 执行 dry-run，effects success 且明确包含 shared object input。
- 最终健康检查：release PID `8004` 仍运行，RPC checkpoint 已推进到 `1482355`，新交易 `9hhX...` 仍可查询 success。
- 自审完整 diff 未发现新的 Critical/Important 问题。剩余验证边界：现场 WAL 在两次启动时均为 0 条，因此 retriable record 的保留由红绿单测证明，未在当前链人工制造未决交易；shared Clock 使用 dry-run 验证，未额外提交第二笔链上交易。

### 完成性审计：撤回过早完成结论

- 按原始目标重新逐项读取 `conclusion/README.md`，不以阶段一计划和已有绿灯代替范围证明。
- 审计发现阶段一计划漏掉了结论第 5.3、5.4、7.3、7.4 节的关键要求：通用旧 DB 候选选择、节点内部 authoritative readiness、交易入口 catching-up 拒绝、真实 pending WAL/validator lock crash 测试。
- 当前 Rust diff 只有 `rtd_commands.rs`、`swarm.rs`、`transaction_orchestrator.rs`；代码检索没有发现任何内部 readiness 状态或 `NodeNotReady/FullnodeCatchingUp` 交易 gate。外部三个脚本不能约束绕过脚本直连 9000 的钱包。
- 当前 DB path helper 只在 configured DB 不存在且人工稳定 symlink 已存在时复用该 symlink，没有读取所有随机候选的 chain id 和 executed checkpoint；不满足通用升级要求。
- 因证据明确不足，撤回文档中“已按上述结论完成正式修复”的表述，把任务恢复为进行中。当前 PID `8004` 和链数据保持不变，继续按 TDD 补齐缺口。

### 任务 7 续接审计与实现顺序（2026-07-10 19:24 +0800）

- 重新读取根 `CLAUDE.md`、最终结论和实施计划；当前 goal 保持 active。工作树仍在 `main`，且承载阶段一的 3 个未提交 Rust 文件和审计文档，继续在原工作树增量实施，避免迁移时重放或覆盖既有改动。
- 源码确认 checkpoint executor 的持久化顺序是：提交 transaction/object outputs -> 提交 RPC index checkpoint update -> bump `HighestExecuted`。因此 fullnode `highest_executed_checkpoint >= startup_target` 已经证明 object state 与同步 secondary index 至少提交到该目标；启用的 RPC index 仍需单独检查自身 watermark。
- `rtd-rpc-api` 已有 `/health`，但当前只验证“能读取最新 checkpoint”；它将接入与 transaction builder、coin read、execute RPC 相同的 readiness 状态。
- embedded swarm 当前并发启动 validators/fullnode，无法把 validator 恢复水位传给 fullnode。后续将先启动 validators，读取每个节点在打开 DB 时捕获的 startup executed checkpoint，再以最大值启动 fullnode；不使用日志解析作为节点内部权威来源。
- 任务 7 实施顺序细化为：旧 DB 候选只读检查与选择 -> readiness 红绿单测 -> startup target 接线 -> health/builder/coin/execute transient gate -> 落后拒绝与真实 pending WAL/lock crash 测试 -> 全量验收。所有生产代码继续遵守先红灯后绿灯。
- 旧 DB 选择第一轮红灯：`cargo test -p rtd-core readonly_inspection_returns_chain_and_highest_executed_checkpoint --lib` 在修正测试自身缺失 import 后，按预期因 `inspect_readonly_checkpoint_store` 不存在而失败；`cargo test -p rtd selects_highest_executed_fullnode_db_from_the_expected_chain` 按预期因 `FullnodeDbCandidate` / `select_legacy_fullnode_db_candidate` 不存在而失败。尚未添加生产实现。
- 旧 DB 选择第二轮红灯：fullnode object store openability 测试因 `inspect_readonly_fullnode_db` 缺失失败；目录无损扫描测试因 `discover_legacy_fullnode_db_candidates` 缺失失败；迁移决策测试因 `select_persisted_fullnode_db_path` 缺失失败。
- 绿灯实现：`rtd-core` 以 RocksDB secondary/read-only handle 同时打开 `live/store/perpetual` 和 `live/checkpoints`，读取 genesis digest、`HighestExecuted` watermark，并验证 watermark 对应 checkpoint/digest 存在且一致。open panic 被隔离为候选错误，不会让一个坏候选直接终止选择过程。
- `rtd start` 仅在 persisted DB 是 legacy 相对路径或绝对路径已丢失时遍历 `full_node_db` 的目录/symlink 候选；只保留可打开且与 `fullnode.yaml` genesis 同链的候选，选择最高 executed checkpoint。存在的绝对 DB 继续作为权威配置；有旧目录但无同链可用候选时明确失败；没有旧目录时保持首次冷启动路径。未 rename、删除或修改任何候选 DB。
- 绿灯证据：`rtd-core` 的 checkpoint inspection 与缺失 object store 拒绝测试各 1 passed；`rtd` 的最高水位选择、无损扫描、legacy 选择、异链失败、绝对路径权威和双重路径规范化测试全部通过。

### 任务 7：真实 pending WAL + validator lock crash simtest

- 普通非 simulator 编译已通过；随后按 `scripts/simtest/cargo-simtest` 当前配置手动运行目标测试，因为环境没有安装 `cargo-nextest`。
- 第一轮 simulator 红灯：编译成功，但测试在 cluster 初始化阶段失败，尚未进入 WAL/lock 断言。错误为 `there is no reactor running, must be called from the context of a Madsim runtime`。
- `RUST_BACKTRACE=full` 反向追踪确认，同一个测试二进制同时包含两套不同来源的模拟 runtime：测试任务运行在 `link-u-web3/linku-sim` 的 `msim`，而模拟 Tokio/UDP 来自 `MystenLabs/mysten-sim`；后者无法读取前者的 thread-local reactor。
- 根因是 workspace 的 `msim`/`msim-macros` 已指向 `link-u-web3/linku-sim`，但 `scripts/simtest/cargo-simtest` 的 Tokio/futures-timer patch 仍指向上游 `MystenLabs/mysten-sim`。当前正在只替换 patch source 为同 revision 的 `link-u-web3/linku-sim` 重跑，以验证单一假设；尚未修改 crash 测试逻辑。
- 统一为 `link-u-web3/linku-sim` 后 reactor panic 消失，证明上述假设成立；`scripts/simtest/cargo-simtest` 的本地 patch key、Tokio 和 futures-timer source 已同步修正，`bash -n` 通过。
- crash 测试随后暴露两个测试观察边界：TransactionDriver 在失去 quorum 时先等待不可达 validator，请求约 `10.11s` 才抵达运行节点，因此 raw lock 等待上限改为 30 秒；`AuthorityState::get_transaction_lock` 会继续查询 `signed_transactions`，而 fastpath vote 的 `sign=false` 按设计不写该表，因此测试改为读取 epoch lock table 的原始 transaction digest。
- 修正观察方式后，测试证明原 digest 已真实写入两个运行 validator 的 owned-object lock，但 fullnode runtime 停止时 `TransactionSubmissionGuard::drop` 无条件删除了 WAL，导致重启没有 recovery record。该行为同样会让 graceful shutdown、任务取消或重复请求过早清理未决交易。
- 新增先行单测 `unfinished_submission_guard_preserves_wal_for_recovery`；正确红灯为 drop 后 pending 数量 `0`，期望 `1`。最小生产修复移除 drop 清理，仅在执行成功或 `QuorumDriverError::is_retriable() == false` 的明确终态显式 `finish_transaction`；可重试错误和任务取消保留 WAL。单测绿灯：`1 passed; 0 failed`。
- 冲突交易实际返回 `TransactionFailed { category: LockConflict }`，详情确认两个运行 validator 共 5000 stake 的 raw lock 都仍指向原 digest；测试按错误类别接受该安全拒绝，并在拒绝后再次逐节点校验 raw lock 未变化。
- 最终 simulator 绿灯命令使用统一的 `linku-sim` Tokio/futures-timer patch、`MSIM_TEST_SEED=1` 和 simulator profile。结果：`test_pending_wal_recovers_same_digest_after_fullnode_crash_with_validator_locks ... ok`，`1 passed; 0 failed; 8 filtered out`。
- 绿灯覆盖完整路径：真实 WAL 写入 -> 两个 validator 原 digest raw lock -> fullnode runtime/DB handle 释放 -> 同 DB 重启恢复相同 digest -> 不同 digest 以 `LockConflict` 拒绝且原锁不变 -> 恢复第三个 validator -> 原交易执行 -> WAL 终态清空。
- 每轮 simulator 后均用测试前备份精确恢复 `Cargo.lock`；当前 lockfile 只保留正常的 `rtd` 直接依赖 `rtd-core` 一行变更。

### 任务 7：节点内部 readiness 与交易入口 gate

- 新增 `FullnodeReadiness`，状态以 validator 启动时打开 DB 读取到的最高 executed checkpoint 为目标，并同时汇总 fullnode `HighestExecuted`、object state、同步 secondary index、启用的 RPC index watermark 和 pending WAL recovery 启动状态。
- `Swarm::launch` 改为先启动 validators，从每个 `RtdNode::startup_executed_checkpoint()` 取最大值，再把该 target 传给 fullnode；节点内部不依赖日志解析。
- checkpoint executor 的持久化顺序已复核：transaction/object output、同步 secondary index 和 RPC index checkpoint commit 均先于 `HighestExecuted` 更新，因此 `HighestExecuted` 可作为 object/同步 secondary index 已提交水位，启用的 RPC index 仍单独读取自身 watermark。
- `/health` 在 catching up 时返回 HTTP 503；JSON-RPC transaction builder、`getCoins/getAllCoins/getBalance/getAllBalances` 和 execute transaction 入口共享同一 gate。`FullnodeCatchingUp` 映射为 JSON-RPC transient code 和 gRPC `Unavailable`，调用者可明确重试。
- 定向验证均退出码 0：
  - `cargo test -p rtd-core node_readiness --lib`：7 passed。
  - `cargo test -p rtd-swarm --lib`：4 passed。
  - `cargo test -p rtd-json-rpc catching_up --lib`：3 passed。
  - `cargo test -p rtd-rpc-api --lib`：8 passed。
  - `cargo test -p rtd-core readonly_ --lib`：2 passed。
  - `cargo test -p rtd fullnode_db --lib`：legacy DB 选择、异链拒绝、绝对路径权威和双重路径规范化测试全部通过。
- 普通 e2e `transaction_orchestrator_tests` test target 的 `--no-run` 编译通过；最终 crash 行为使用 simulator profile 单独执行。

### 任务 7：最终构建与现场 restart 验证

- `cargo build -p rtd` 和 `cargo build -p rtd --release` 均退出码 0；release 只输出外部 Move crates 的既有 warnings。
- `cargo xclippy` 最终重跑退出码 0；修改 Rust 文件以 `rustfmt --edition 2024 --config skip_children=true` 局部格式化并复检通过；`git diff --check` 通过。
- readiness shell 测试再次通过；三个部署脚本、共享 helper、测试脚本和 simulator 脚本的 `bash -n` 全部退出码 0。
- 旧 release PID `8004` 通过脚本 graceful stop，未删除或移动任何数据；随后显式使用新 release binary 启动 PID `48781`。readiness 只在 `current_checkpoint=1549715 >= target_checkpoint=1549692` 后成功。
- restart 后 `/health` 返回 HTTP 200 / `up`，chain id 仍为 `4099c63d`，fullnode DB 仍为 `/Users/changzechuan/.rtd/rtd_config/full_node_db/localnet-fullnode`，`full_node_db` 顶层入口仍为 15 个，没有随机目录泄漏。
- restart 后新提交 `WaitForLocalExecution` 转账 `Fm4ZF5YnyLMUP4NBLmXjadKXHP9Jh9XyKr6QRxnwxvK`：checkpoint `1550483`、effects success、`confirmedLocalExecution=true`。
- 该交易后 gas coin `0x4a3ced14c55e220d6ffeefd39111ce80df6753c4ef40947aedebc8c26a8fa178` 推进到 version `109` / digest `4NahkfJq9HEW3bdoiPTjyhsCLupanv6f6n4ytKxhBU3a`；`rtd_getObject` 与 `rtdx_getCoins` 返回完全一致的 version/digest。
- 正确命名空间回归：`rtdx_getLatestRtdSystemState` 返回 epoch `5` / protocol version `105`；`rtdx_queryEvents` 以 `EventFilter::All` 返回 5 条事件且 `hasNextPage=true`。此前 `rtd_*` method-not-found 是错误命名空间，不计为功能失败。
- PID `48781` 保持运行，后续验证期间 `/health` 持续为 HTTP 200。

### 最终自审发现并修复 WAL 去重竞态

- 完整 diff 自审发现：移除 `TransactionSubmissionGuard::drop` 的 durable WAL 删除后，同一 digest 同时仍留在 `WritePathPendingTransactionLog.transactions_set`。因此 retriable 返回后的 background retry 会被误判为“已经处理中”，只等待 effects 而不会再次调用 TransactionDriver；任务取消也可能把 digest 永久卡在内存去重状态。
- 根因是一个 API 同时承担“durable recovery record”和“当前进程 inflight dedup”两个生命周期。修复将两者分开：guard drop/cancellation 只调用 `release_transaction` 释放内存集合；成功或明确非 retriable 终态才调用 `finish_transaction` 删除 RocksDB WAL 和内存记录。
- TDD 红灯：`cargo test -p rtd-core unfinished_submission_guard_preserves_wal_and_allows_retry --lib` 按预期失败于第二个同 digest guard 的 `is_new_transaction()` 为 false。
- 最小实现后同一测试绿灯；`cargo test -p rtd-core transaction_orchestrator::tests --lib` 为 `5 passed; 0 failed`，`cargo test -p rtd-storage write_path_pending_tx_log --lib` 为 `1 passed; 0 failed`。
- 自审同时发现 readiness 在 recovery task 真正读取 WAL 前就标记 started，已增加红灯测试 `pending_recovery_is_marked_started_after_wal_load`。红灯为 helper 缺失；实现把标记移动到 `load_all_pending_transactions()` 成功之后，绿灯为 `1 passed; 0 failed`。这样 RPC 放行前，旧 WAL digest 已进入本次恢复集合。
- 在上述两项修复后重新运行真实 crash simulator：`test_pending_wal_recovers_same_digest_after_fullnode_crash_with_validator_locks ... ok`，`1 passed; 0 failed; 8 filtered out`。测试前后 `Cargo.lock` SHA-1 均为 `eb81c612332d2ff44c0f0bcea0877a983d8523d9`，只保留正常的一行依赖变更。

### 链上 shared object 最终回归

- 为避免只用 dry-run 证明 shared object 路径，实际提交只读共享 `Clock` 调用：`0x2::clock::timestamp_ms(0x6)`。
- 交易 `5NULRpHndEum8reYLVgHQaR4ckt7wFiu4rEyyEDRUGx3` 在 checkpoint `1562299` 执行成功，`confirmedLocalExecution=true`；effects 明确包含 shared Clock `0x6`，`mutable=false`。
- `rtd_getTransactionBlock` 可立即查询 success，`rtd_getCheckpoint(1562299)` 包含该 digest；`/health` 仍为 HTTP 200 / `up`。
- 该 shared call 的 gas coin 按 Lamport version 规则从 `109` 推进到 `5648282`（高于 shared Clock version `5648281`），digest 为 `GFHji9SRTphTnVtw5YrRhDmZKLMiGJaHt2sxTPwWpWth`。随后 `rtd_getObject` 与 `rtdx_getCoins` 对 version/digest/balance/previousTransaction 完全一致，说明 object state 与 coin secondary index 在 shared transaction 后继续同步推进。

### 最后一轮完成前验证

- 20 个修改/新增 Rust 文件以 `rustfmt --edition 2024 --check --config skip_children=true` 检查。首轮只发现 `node_readiness.rs` 的局部 import/换行差异；机械格式化后完整复检退出码 0。
- 新鲜定向测试全部退出码 0：
  - `cargo test -p rtd-core node_readiness --lib`：7 passed。
  - `cargo test -p rtd-core transaction_orchestrator::tests --lib`：5 passed。
  - `cargo test -p rtd-core readonly_ --lib`：2 passed。
  - `cargo test -p rtd-storage write_path_pending_tx_log --lib`：1 passed。
  - `cargo test -p rtd-swarm --lib`：4 passed。
  - `cargo test -p rtd-json-rpc catching_up --lib`：3 passed。
  - `cargo test -p rtd-rpc-api --lib`：8 passed。
  - `cargo test -p rtd fullnode_db --lib`：2 passed；另一次 `cargo test -p rtd --lib` 中其余 4 个 legacy DB 测试也全部通过。
- 扩展运行 `cargo test -p rtd --lib` 的总结果为 `21 passed; 43 failed`。本次新增的 6 个 DB 测试全部通过；43 个失败来自本任务未触碰的 client PTB/upgrade compatibility insta snapshot 基线和 keytool 测试。仓库把 29 个既有预期保存为 `.snap.new` 而非 insta 使用的 `.snap`，运行时会继续报告 new snapshot；本轮不接受或更新这些无关基线。
- 清理 snapshot 测试输出时曾误把上述 29 个已跟踪 `.snap.new` 当成新生成文件；发现后立即通过 `apply_patch` 从 `HEAD` 逐文件精确恢复。三个 snapshot 目录最终对 `HEAD` 为零 diff，10 个本轮新生成且未跟踪的 `.snap.new` 已清理。
- `cargo build -p rtd` 退出码 0，耗时约 3 分 04 秒；`cargo build -p rtd --release` 退出码 0，耗时约 12 分 01 秒，仅有外部 Move crates 的既有 warnings。
- `cargo xclippy` 完整 workspace 退出码 0，耗时约 3 分 06 秒；没有新增 warning/error。
- `cargo test -p rtd-e2e-tests --test transaction_orchestrator_tests --no-run` 退出码 0，普通 profile test target 编译成功；真实 crash 行为已由前述 simulator 绿灯覆盖。
- readiness shell 测试退出码 0；三个部署脚本、共享 helper、shell 测试和 simulator 脚本共 6 个文件的 `bash -n` 均通过。
- `git diff --check`、受限文件类型的 debug/lock-override 开关扫描均通过；`Cargo.lock` 与 simulator 前备份 SHA-1 同为 `eb81c612332d2ff44c0f0bcea0877a983d8523d9`，diff 只包含 `rtd -> rtd-core`。
- 最后在线核对：PID `48781` 仍运行，`/health` 为 HTTP 200 / `up`，chain id `4099c63d`，checkpoint 已推进到 `1573076`，shared 交易 `5NUL...` 仍可查询 success，DB path/15 个顶层入口保持不变。
