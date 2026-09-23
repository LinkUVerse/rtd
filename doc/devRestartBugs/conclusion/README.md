# RTD dev 链停机重启故障最终结论

日期：2026-07-10

## 1. 最终结论

本问题的主根因已经定位在 `rtd start` 的本地网络启动架构，而不是 Sui/RTD 的对象版本校验逻辑。

持久化 localnet 重启时，validator 会复用 `network.yaml` 中的稳定数据库：

```text
~/.rtd/rtd_config/authorities_db/99f25ef61f80
```

但原始 `rtd start` 没有复用 `rtd genesis` 已生成的 `fullnode.yaml`，而是每次通过
`SwarmBuilder -> FullnodeConfigBuilder::build` 随机生成一个新的 fullnode 身份。fullnode 数据库路径又由这个随机身份的 public key 前缀派生，因此每次重启都会切换到新的目录：

```text
~/.rtd/rtd_config/full_node_db/<random-key-prefix>
```

实际环境中已经留下了多个这样的目录，例如：

```text
819bc1dbe416
8c3702a4a524
8fdd20bc8ecf
971eb7b41e1e
a83c272c331b
b3ccc0c08bc6
...
```

结果是同一个 `rtd start` 进程里出现了两个不同进度的状态源：

```text
钱包 / JSON-RPC
    -> 新建或落后的 fullnode DB
    -> 返回旧 gas coin ObjectRef

TransactionDriver
    -> validator
    -> validator 复用稳定 DB，持有更新的 live object
    -> 拒绝旧 ObjectRef
```

因此用户看到：

```text
Version 0x6a ... is not available for consumption,
current version: 0x6b
```

是完全符合协议规则的结果。错误不在 validator 拒绝了旧版本，而在 RPC fullnode 向钱包提供了 validator 已经不能消费的旧版本。

要满足“停机后不删除任何链上数据，重新启动后转账和链上功能正常”，必须同时完成以下修复：

1. 持久化并复用同一个 fullnode 配置、身份、DB 和 pending transaction log。
2. fullnode 未追平 validator 以及 RPC 二级索引未就绪时，不能把节点声明为 ready，也不能允许交易构造/提交接口返回或使用旧对象引用。
3. 对重启前处于不确定状态的交易，必须恢复并重提同一个 transaction digest，不能覆盖 validator 上已有的 owned-object lock。

## 2. 能直接证明根因的证据

### 2.1 validator 与 fullnode 的启动 checkpoint 明显分叉

2026-07-10 当前进程启动日志记录：

```text
validator: executed=1418995 certified=1418995
fullnode:  executed=1339965 certified=1418993
```

2026-07-10 16:54 CST 实时查询：

```text
rtd_getLatestCheckpointSequenceNumber = 1376213
```

也就是说，RPC fullnode 启动约十小时后仍然没有执行到 validator 的启动高度 `1418995`。RPC 端口可连接并不代表该 fullnode 已经能够提供 validator 当前状态。

更早的复现中差距更大：

```text
validator executed checkpoint = 1209471
fullnode RPC checkpoint       = 808
```

这不是 coin index 单表偶发落后一条记录，而是整个 fullnode 执行状态落后约 120 万个 checkpoint。

### 2.2 错误中的 0x6a/0x6b 与两套状态精确对应

当前 RPC fullnode 返回目标 gas coin：

```text
object id = 0x4a3ced14c55e220d6ffeefd39111ce80df6753c4ef40947aedebc8c26a8fa178
version   = 106 (0x6a)
digest    = 3dntckSuZt17DYCn5CRdKwfc18iyJxm5ATZC4JbiyCVX
previous  = 9fWt8oeVio9KDd9zeGA1oKpNjUcfmYAxaVRBZiRWjL1M
```

历史排查已经确认交易：

```text
CzQbQQB8W4krhwthGKxPxDKLfSAskGYrtzV43yhsNacW
```

在 validator 侧成功执行，并把同一个 gas coin 更新为：

```text
version = 107 (0x6b)
digest  = Heo8EvVSR8o3brZxKGpSfT2jWRPWA9cfVaDR6ZwZSBBZ
```

所以错误不是推测：钱包从落后 fullnode 取得 `0x6a`，validator 已经执行到 `0x6b`，两者正好就是报错中的 provided/current version。

### 2.3 `getCoins` 与 `getObject` 同时落后，排除“只有 coin index 坏了”

多次现场对比都出现：

```text
rtdx_getCoins == rtd_getObject == fullnode 旧版本
validator live object          == 更新版本
```

例如曾经同时返回 version `100`，而 validator 为 version `106`；当前两者又同时返回 version `106`，而 validator 已执行到 version `107`。

如果根因只是 `coin_index_2` 没更新，应当看到：

```text
getCoins = 旧版本
getObject = 新版本
```

实际长期证据不是这个模式。因此 coin index 曾经可能存在局部问题，但它不是本次停机重启故障的主根因。

### 2.4 源码明确显示 fullnode DB 每次由随机 key 派生

调用链：

```text
crates/rtd/src/rtd_commands.rs::start
  -> Swarm::builder().with_network_config(...)
  -> SwarmBuilder::build
  -> FullnodeConfigBuilder::build(&mut OsRng, ...)
  -> ValidatorGenesisConfigBuilder::new().build(rng)
  -> get_key_path(random protocol key)
  -> <config_dir>/full_node_db/<random-key-prefix>
```

关键代码位置：

- `crates/rtd/src/rtd_commands.rs::start`
- `crates/rtd-swarm/src/memory/swarm.rs::SwarmBuilder::build`
- `crates/rtd-swarm-config/src/node_config_builder.rs::FullnodeConfigBuilder::build`

`FullnodeConfigBuilder::build` 每次生成新的 key，并在没有显式 `db_path` 时执行：

```rust
config_directory.join(FULL_NODE_DB_PATH).join(key_path)
```

这与现场出现的大量随机 fullnode DB 目录完全一致。

### 2.5 `rtd genesis` 已保存 fullnode.yaml，但 `rtd start` 没有使用

`rtd genesis` 在下面位置生成配置：

```text
~/.rtd/rtd_config/fullnode.yaml
```

源码位置：

```text
crates/rtd/src/rtd_commands.rs::genesis
```

但持久化启动分支只读取 `network.yaml` 中的 validator configs，随后再次随机 build fullnode。也就是说，项目生成了可持久化的 fullnode 配置，却没有在 `rtd start` 的重启路径中复用它。

这项行为继承自 RTD fork 时对应版本的 Sui 源码，不是 RTD 品牌替换造成的拼写错误。

### 2.6 最新 Sui 已对同一根因做定向修复

最新 Sui 仓库：

```text
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/temp/sui
```

版本与提交：

```text
sui_v1.76.0_1783574598_ci
b76bcd0e2bc3551fda92dfff1eeaea75622061ab
```

其中提交：

```text
eced02468444d429a4e9a2b9622b7bd30a1710d4
fix(sui): `sui start` resumes the embedded fullnode (index over the fullnode gRPC) (#26884)
```

就是本问题的上游定向修复。提交说明中的问题、根因和 RTD 现场逐项一致：

1. `sui start` 读取持久化 validator 的 `network.yaml`，但从不读取 genesis 保存的 `fullnode.yaml`。
2. 每次启动通过 `FullnodeConfigBuilder::build` 生成随机 fullnode 身份，由身份派生新的 DB 目录。
3. validator 从最新状态恢复，RPC fullnode 却从 genesis 重放整条链。
4. 每次启动遗留一个 `full_node_db/<key>` 目录。
5. genesis 使用 `with_config_directory(FULL_NODE_DB_PATH.into())`，造成
   `full_node_db/full_node_db/<key>` 双重路径。

上游修复方式：

- `sui_commands.rs` 在 `<config_dir>/fullnode.yaml` 存在时读取完整 `NodeConfig`，保留 fullnode identity 和 DB，只刷新 RPC 配置以及容易端口冲突的 metrics/admin/network/p2p listen address。
- `SwarmBuilder` 新增 `with_fullnode_config(NodeConfig)`，第一个 embedded fullnode 使用预构建配置，不再随机生成。
- `sui genesis` 改为把真实 config dir 传给 `FullnodeConfigBuilder`，修复双重相对路径。
- 没有 `fullnode.yaml` 的冷启动和 `--force-regenesis` 仍生成新身份，保持临时链语义。

因此 RTD 不需要自行发明 fullnode 持久化协议，应按品牌和版本 API 差异移植该上游修复。

最新 Sui 同时证明两点：

- 它没有增加任何覆盖不同 transaction digest owned-object lock 的逻辑，`ObjectLockConflict` 仍是安全边界。
- 它的 pending transaction recovery 仍存在“恢复尝试结束后调用 `finish_transaction`”的代码，没有覆盖 RTD 现场的 retriable transport error/WAL 丢失问题；这部分仍需基于 RTD 现场单独修复。

最新 Sui 的 rpc-store health check 还增加了 executed checkpoint 与 live-object index frontier 的 lag 检查。RTD 当前版本没有相同的 rpc-store cohort 架构，不能直接复制代码，但“索引未追平时 health/readiness 必须失败”的原则与本文方案一致。

## 3. 为什么“停机 15 小时”不是协议阈值

不存在“停机超过 15 小时，owned object 自动失效”的规则。

链完全停机时不会因为墙上时间流逝自动把普通 owned object 从 `0x6a` 改为 `0x6b`。真正相关的是：

1. 停机前链已经积累了大量 checkpoint。
2. 重启时 validator 从稳定 DB 的最新高度继续运行。
3. RPC fullnode 却换了随机 DB，从 genesis 或某个很旧的高度追赶。
4. validators 恢复产块后，fullnode 还要一边追历史、一边追新增 checkpoint。
5. 钱包在 fullnode 未追平时读取并签入旧 ObjectRef。

所以 15 小时只是稳定触发问题的运行场景，不是根因，也不是修复时应引入的超时常量。

## 4. 对之前排查结论和修改的最终评价

### 4.1 应保留并完善：稳定 fullnode DB 路径

当前工作区给 `SwarmBuilder` 增加：

```text
with_fullnode_db_path(...)
```

并让 `rtd start` 使用：

```text
<config_dir>/full_node_db/localnet-fullnode
```

方向正确，它能阻止未来每次启动继续创建随机 DB。

但这还是不完整修复：

- 只固定了 DB path，没有复用 `fullnode.yaml` 中的 node identity 和完整配置。
- 已存在的链需要选择并迁移正确的旧 DB；一个新的空稳定目录仍会产生一次全量追赶。
- 没有从进程内部阻止落后 fullnode 对外提供交易输入。
- 当前 `localnet-fullnode` 还是指向旧候选 DB `971eb7b41e1e` 的人工 symlink，属于现场迁移措施，不是通用升级逻辑。

### 4.2 应保留为纵深防御：钱包不持久化 ObjectRef

钱包发送前重新获取 coins、避免长期持久化 `{ objectId, version, digest }`、失败后重新 build/sign，而不是重试旧签名交易，这些修改是合理的。

但它们不能修复本次主问题。钱包即使每次实时 refetch，如果 RPC fullnode 本身落后，得到的仍然是“最新的旧状态”。

### 4.3 应撤销：让 validator 也执行 JSON-RPC coin indexing

`AuthorityState::fullnode_only_get_tx_coins_for_indexing` 跳过 validator 是 Sui 的正常架构假设。当前 localnet 虽然由一个 `rtd start` 进程管理，但 validator 和 RPC fullnode 仍是两个独立 `AuthorityState`/DB，不是一个 authority 同时承担两种角色。

fullnode 只要正常执行 checkpoint，就会走既有 coin index 更新逻辑。把 validator 也纳入 JSON-RPC coin indexing：

- 没有解决 RPC fullnode checkpoint 落后。
- 增加 validator 执行路径开销。
- 扩大与上游 Sui 的行为差异。

该修改应撤销，除非未来真的设计“同一个 AuthorityState 同时作为 validator RPC 服务”的新架构，并为此单独评审。

### 4.4 必须撤销：`RTD_REPAIR_ORPHAN_OBJECT_LOCKS`

当前 debug 修改用以下条件判断旧 lock 是“孤儿锁”：

```text
get_signed_transaction(old_digest) == None
is_pending_consensus_certificate(old_digest) == false
```

然后允许新 transaction digest 覆盖旧 lock。这个判定对 Mysticeti fastpath 并不成立，而且有安全风险。

源码明确显示：

1. `handle_vote_transaction` 调用 `handle_transaction_impl(..., sign=false)`。
2. owned-object lock 会持久化，但因为 `sign=false`，本来就不会写 `signed_transactions`。
3. `insert_pending_consensus_transactions` 对 `is_mfp_transaction()` 明确返回 `None`，fastpath UserTransaction 本来就不会进入 pending consensus transaction 表。
4. `pending_consensus_certificates` 只跟踪 certified transaction，不证明一个 fastpath UserTransaction 没有被投票、提交或即将执行。

因此当前所谓“孤儿锁”条件会把正常 fastpath vote 形成的 lock 也判为可覆盖。允许不同 digest 抢占它，破坏了 owned object 防双花所依赖的锁不变量。

这个开关虽然让一次测试交易绕过了 `ObjectLockConflict`，但不能作为正式修复，必须删除，不能默认开启，也不应保留为运维工具。

### 4.5 不应作为修复：`RTD_CLIENT_WAIT_FOR_EFFECTS_CERT`

把 CLI 从 `WaitForLocalExecution` 改成 `WaitForEffectsCert`，只能让命令在 validator 已给出成功 effects、但本地 fullnode 尚未执行时提前返回成功。

它会掩盖以下事实：

- `rtd_getTransactionBlock` 仍查不到交易。
- `rtd_getObject` 和 `getCoins` 仍可能返回交易前的 object version。
- 下一笔交易仍可能使用旧 gas ref，再次被 validator 拒绝。

所以正式默认行为必须保留 `WaitForLocalExecution`。`WaitForEffectsCert` 只能作为明确理解一致性语义的诊断选项，不能用来证明重启问题已经解决。

### 4.6 诊断日志可以保留到回归完成，再缩减

`RTD_STATE_DIVERGENCE`、transaction driver、orchestrator、effects certifier 等日志已经完成定位任务。回归测试完成前可以保留；正式合入时应把高频 `info` 日志降级或删除，避免 checkpoint 追赶时产生巨量日志和 I/O 压力。

## 5. 正确修复方案

### 5.1 P0：先恢复协议安全边界

立即撤销 `crates/rtd-core/src/execution_cache/object_locks.rs` 中基于
`RTD_REPAIR_ORPHAN_OBJECT_LOCKS` 覆盖不同 transaction digest lock 的逻辑和测试。

正确原则是：

```text
同一个 ObjectRef 已锁给 tx A
    -> 允许再次提交完全相同的 tx A
    -> 不允许 tx B 覆盖
```

如果 tx A 的最终状态不确定，应恢复 tx A，而不是创建 tx B 抢锁。

### 5.2 P1：让持久化 rtd start 复用完整 fullnode 配置

推荐直接移植最新 Sui 提交 `eced024684` 的核心方案：不是只给随机 fullnode 指定固定 DB，而是让持久化启动分支加载并复用：

```text
<config_dir>/fullnode.yaml
```

需要调整：

```text
crates/rtd/src/rtd_commands.rs
crates/rtd-swarm/src/memory/swarm.rs
crates/rtd-swarm-config/src/node_config_builder.rs（如需要）
```

目标行为：

1. `rtd genesis` 生成一次 fullnode protocol/account/worker/network keys。
2. `fullnode.yaml` 使用绝对或以 config dir 正确解析的稳定 DB：

   ```text
   <config_dir>/full_node_db/localnet-fullnode
   ```

3. 后续每次 `rtd start` 读取同一份 `NodeConfig`，只按 CLI 参数覆盖允许变化的 RPC address/port。
4. 不重新生成 fullnode keys，不重新派生随机 DB path。
5. fullnode 的 object store、checkpoint store、indexes、rpc index、epoch tables 和
   `fullnode_pending_transactions` 全部随同一个 DB 恢复。

当前 `rtd genesis` 使用：

```rust
.with_config_directory(FULL_NODE_DB_PATH.into())
```

会让保存的 `fullnode.yaml` 出现类似：

```text
db-path: full_node_db/full_node_db/<id>
```

这里也应改为基于真实 `rtd_config_dir` 的路径，或直接显式传入稳定 DB path。

如果暂时只采用当前 `with_fullnode_db_path` 窄修复，也必须把它视为第一阶段，并补上配置持久化、旧链迁移和 readiness gate。

### 5.3 P1：为已有链提供一次性无损迁移

现有环境不能简单创建一个空 `localnet-fullnode` 后立即对外服务。升级逻辑应：

1. 读取所有旧 `full_node_db/<random-id>` 候选的 chain identifier/genesis。
2. 只接受与当前 `network.yaml`/`genesis.blob` 属于同一条链的 DB。
3. 从候选中选择 `highest_executed_checkpoint` 最大且数据库可正常打开的一份。
4. 原子 rename 到稳定目录，或把稳定配置明确指向该目录。
5. 保留备份，不删除 validator DB 和其他候选，确认回归通过后再人工清理。

不能只按目录 mtime 或大小选择，因为 validator DB 副本、启动失败的半成品 DB、只同步了 certified checkpoint 但未执行的 DB 都可能误导判断。

当前现场使用 `localnet-fullnode -> 971eb7b41e1e` 是合理的保守迁移起点，但它还在追赶，不能据此宣布整个问题已修复。

### 5.4 P1：在节点内部实现真正的 readiness gate

外部 `toggle_local_rtd.sh` 比较 checkpoint 的逻辑是正确方向，但正式保证不能只依赖脚本。

`rtd start` 应在启动时记录 validator 恢复出的最高 executed checkpoint，定义启动目标：

```text
startup_target = max(all validators' highest executed checkpoint at startup)
```

fullnode ready 至少需要同时满足：

```text
fullnode highest executed checkpoint >= startup_target
fullnode object state 已提交到该 checkpoint
fullnode JSON-RPC secondary indexes 已处理到该 checkpoint
fullnode RPC/checkpoint index 已 commit 到该 checkpoint
pending transaction recovery 已开始且没有因节点未就绪被一次性丢弃
```

在此之前：

- health/readiness 应返回 `not ready`。
- transaction builder、gas coin 选择和 `executeTransactionBlock` 应返回明确、可重试的
  `NodeNotReady/FullnodeCatchingUp` 错误。
- 不能返回一个看似正常、实际会被 validator 拒绝的交易输入。

只打印“Cluster started”或 RPC 端口成功监听不构成 ready。

为了避免 RPC 已监听期间钱包抢跑，建议在 transaction-oriented RPC 入口增加服务端 gate；只靠启动脚本等待无法约束直接访问 9000 端口的浏览器钱包。

### 5.5 P1：修复 pending transaction 重启恢复

随机 fullnode DB 不仅丢失 RPC 状态，也会丢失：

```text
<fullnode db>/fullnode_pending_transactions
```

这会切断“validator 已给 tx A 加锁，但 fullnode 重启后应继续重提 tx A”的恢复链。

当前源码还有一个已由日志证实的次生 liveness 问题：

```text
TransactionOrchestrator::start_task_to_recover_txes_in_log
```

对恢复交易调用 `drive_transaction` 后，无论成功还是失败，最后都会执行：

```rust
pending_tx_log.finish_transaction(&tx_digest)
```

现场 `5e513...` 的过程是：

1. fullnode 重启时确实恢复出 1 笔 pending transaction。
2. validator/transport 尚未就绪，连续返回 retriable `RpcError("transport error", "Unknown error")`。
3. 恢复任务失败后仍删除 WAL。
4. 下一次启动显示 `Recovering 0 pending transactions`。
5. validator DB 中原有 lock 仍指向 `5e513...`，新 digest 因此遇到 `ObjectLockConflict`。

正确修改应为：

- 成功拿到 finalized effects：删除 WAL。
- 明确证明交易已执行：删除 WAL。
- 明确的永久 invalid transaction：记录终态后删除 WAL。
- `Unavailable`、transport error、timeout、节点尚未 ready：保留 WAL，带退避继续重提同一个 digest。
- recovery task 应等待 validator/fullnode 基础服务 ready，不能启动即失败一次后永久丢弃。

这样可以按协议安全地解决 lock liveness，不需要、也不允许覆盖 lock。

### 5.6 P2：钱包/SDK 保留防御，但以节点 ready 为前提

钱包仍应：

1. 不持久化可消费 ObjectRef。
2. 首次发送前重新获取 coins/objects，再 build 和签名。
3. transaction digest 已提交但结果不确定时，保留并重提同一份 signed bytes/同一 digest，不立即生成消费同一 gas coin 的新 digest。
4. 只有明确收到 stale object 等永久无效错误时，才重新获取对象、重新 build/sign 新交易。
5. 保留 `WaitForLocalExecution` 作为正常成功标准。

但是节点未 ready 时，钱包应直接收到明确的 catching-up 错误，而不是依靠客户端猜测 `getCoins` 是否可信。

## 6. 建议删除或回退的当前工作区行为改动

正式实现前，建议把当前 43 个 Rust 文件的改动拆分审计，至少按以下方式处理：

| 修改 | 处理意见 |
| --- | --- |
| 稳定 fullnode DB path | 保留方向，升级为复用完整 `fullnode.yaml` |
| `rpc_fullnode_uses_configured_db_path` 测试 | 保留，并扩展为两次启动状态恢复测试 |
| 启动脚本 checkpoint readiness | 保留为外部防线，节点内部再实现 authoritative gate |
| 钱包 fresh coin refs/cache 修复 | 保留为纵深防御 |
| validator coin indexing 修改 | 回退 |
| `RTD_REPAIR_ORPHAN_OBJECT_LOCKS` | 必须回退 |
| `RTD_CLIENT_WAIT_FOR_EFFECTS_CERT` | 不作为正式修复；默认行为回退 |
| 高频诊断日志 | 回归完成后降级/删除 |
| rustfmt/import-only 大面积变化 | 与功能修复拆开或回退，避免干扰审查 |

## 7. 必须补充的自动化测试

只有编译、clippy 和单个 DB path 单测通过，不能证明停机重启问题已经解决。至少需要以下测试。

### 7.1 持久化 fullnode 配置测试

同一 config dir 连续构建/启动两次：

```text
fullnode protocol key 相同
fullnode network key 相同
fullnode db_path 相同
chain identifier 相同
第二次启动 checkpoint 不回退
```

### 7.2 完整 restart e2e

步骤：

1. `rtd genesis` 创建持久化 localnet。
2. 启动 validator + fullnode。
3. 执行转账，记录 tx digest、gas object version、checkpoint、chain id。
4. 不删除任何数据，停止整个 `rtd` 进程。
5. 使用同一 config dir 重启。
6. 等待节点 readiness，而不是只等端口。
7. 断言 chain id 不变、checkpoint 不下降。
8. 断言 `getCoins` 与 `getObject` 对同一 coin 的 version/digest 一致。
9. 再执行一笔转账，要求：

   ```text
   effects.status = success
   confirmedLocalExecution = true
   rtd_getTransactionBlock 可立即查询
   gas object version 正确推进
   sender/receiver balance 正确
   ```

测试应同时覆盖 graceful shutdown 和 kill -9。

不需要真的 sleep 15 小时；要构造的是“存在历史 checkpoint + fullnode 重启”的状态。15 小时不是协议条件。

### 7.3 fullnode 落后时的拒绝测试

刻意让 validator 高于 fullnode：

- readiness 必须为 false。
- transaction builder/execute RPC 必须返回可重试 catching-up 错误。
- 不允许产生 `ObjectVersionUnavailableForConsumption` 这种由本节点内部状态分叉制造的交易。
- fullnode 追平后同一调用自动恢复正常。

### 7.4 pending transaction/lock 恢复测试

在 validator 已写 owned-object lock、fullnode 尚未拿到最终结果的边界强制终止进程：

1. 重启后从稳定 WAL 恢复原 transaction bytes。
2. 重提相同 digest。
3. 原交易最终成功或得到明确终态。
4. 不同 digest 的冲突交易继续被拒绝。
5. WAL 只在终态后删除。

这个测试用于替代不安全的 orphan-lock 覆盖测试。

### 7.5 链上功能回归

为了证明“链上各项功能正常”，restart e2e 还应至少覆盖：

- owned object 转账。
- shared object Move call。
- checkpoint 查询。
- transaction/effects/events 查询。
- `getCoins/getAllCoins` 和 balance 查询。
- epoch/system state 查询。
- RPC index/secondary index 在重启后持续推进。

## 8. 验收标准

以下条件全部满足后，才能宣布问题修复：

1. 连续多次使用同一 config dir 启停，不再产生新的随机 fullnode DB 目录。
2. 重启后 fullnode identity、DB、chain id 和 pending tx WAL 均保持不变。
3. 节点未追平时不对交易客户端报告 ready，也不提供可签名的 stale ObjectRef。
4. ready 后 `getCoins`、`getObject`、validator live object 对同一 owned object 的 version/digest 一致。
5. 不开启任何 object lock 覆盖开关，重启后的 pending 交易仍能按同 digest 恢复。
6. `WaitForLocalExecution` 下转账成功，交易可立即查询，本地 object/balance 已推进。
7. 完整 restart e2e 在 graceful shutdown 和 kill -9 两种场景通过。
8. shared object、checkpoint、events、system state 和索引查询回归通过。

## 9. 当前链的无损处理结果

当前链没有清除 validator/fullnode 数据，也没有重新 genesis。旧 `fullnode.yaml` 的 legacy DB path 已持久化为：

```text
/Users/changzechuan/.rtd/rtd_config/full_node_db/localnet-fullnode
```

处理结果：

1. fullnode protocol identity、stable DB 和 chain id `4099c63d` 在 graceful shutdown、SIGKILL 和再次 graceful restart 后均保持不变。
2. `full_node_db` 顶层入口始终为 15 个，没有再生成随机 identity 目录；没有删除、rename 或覆盖任何旧候选 DB。
3. 最近一次 restart 的内部/脚本 readiness 为
   `current_checkpoint=1549715 >= target_checkpoint=1549692`，之后 `/health` 持续返回 HTTP 200 / `up`。
4. restart 后新转账 `Fm4ZF5YnyLMUP4NBLmXjadKXHP9Jh9XyKr6QRxnwxvK` 在 checkpoint `1550483` 成功，`confirmedLocalExecution=true`。
5. 实际 shared object Move call `5NULRpHndEum8reYLVgHQaR4ckt7wFiu4rEyyEDRUGx3` 在 checkpoint `1562299` 成功；effects 明确包含只读 shared Clock `0x6`，交易和 checkpoint 可立即查询。
6. shared call 后 `rtd_getObject` 与 `rtdx_getCoins` 对 gas coin 一致返回 Lamport version `5648282`、digest `GFHji9SRTphTnVtw5YrRhDmZKLMiGJaHt2sxTPwWpWth` 和相同 balance/previous transaction。

当前 release 进程 PID `48781` 继续运行同一 config/DB；不需要对现有链执行额外数据修复。

## 10. 最终实施与验收结果

2026-07-10 已完成本文第 5、7、8 节定义的正式修复与验收，不再保留阶段一复核时列出的四项缺口。

### 10.1 fullnode 持久化与通用旧 DB 选择

1. 按最新 Sui 定向修复 `eced024684` 移植预构建 fullnode `NodeConfig`：持久化 `rtd start` 读取完整 `fullnode.yaml`，复用 keys、DB 和 pending transaction log，只刷新本次运行的 RPC/监听端口及显式 ingestion dir。
2. `rtd genesis` 使用真实 config dir，不再生成 `full_node_db/full_node_db/<id>`。
3. legacy 配置升级会以只读方式打开 object/checkpoint stores，验证 genesis chain id、`HighestExecuted` checkpoint 及 digest 一致性，选择同链、可打开且 executed checkpoint 最高的候选。
4. 已存在的绝对 DB path 继续是权威配置；候选选择不删除、不移动、不改写旧 DB，没有同链可用候选时明确失败。

### 10.2 节点内部 authoritative readiness

1. validators 先启动，并从各自已打开的 checkpoint DB 捕获 startup executed checkpoint；fullnode target 为这些值的最大值，不依赖日志解析。
2. `FullnodeReadiness` 同时检查 `HighestExecuted`、object state、启用的同步 secondary index、RPC index watermark 和 pending WAL 已成功读取。
3. checkpoint executor 的提交顺序保证 object/同步 secondary index/RPC index commit 先于 `HighestExecuted` 更新；启用的 RPC index 仍单独检查自身 watermark。
4. 未 ready 时 `/health` 返回 503；transaction builder、coin/balance 读取和 execute transaction 返回明确、可重试的 `FullnodeCatchingUp`。JSON-RPC 使用 transient error code，gRPC 使用 `Unavailable`。
5. 三个本地部署脚本继续保留相同 checkpoint gate 作为外部防线；RPC 端口可连接不再被当作 ready，超时会非零退出并报告 current/target。

### 10.3 pending WAL 与 owned-object lock 安全

1. 成功或明确永久错误才删除 durable WAL；timeout/unavailable/transport error、任务取消和 graceful/crash shutdown 均保留原 signed transaction bytes/digest。
2. durable recovery record 与当前进程的内存去重生命周期已分离：retriable/cancellation 释放内存 inflight 标记，使后台可再次提交同 digest，但不删除 RocksDB WAL。
3. readiness 只在 durable WAL 成功枚举后标记 recovery started，避免 RPC 放行早于 pending digest 进入恢复集合。
4. recovery 对 retriable error 指数退避并持续提交完全相同的 transaction；不同 digest 仍受 `ObjectLockConflict` 安全拒绝，不存在 lock 覆盖路径。

### 10.4 自动化与现场证据

1. legacy DB inspection/selection、readiness watermarks、health、builder、coin、execute 和错误映射的定向单测均通过。
2. simulator crash 测试覆盖：真实 WAL -> 两个 validator 原 digest raw lock -> fullnode 停止/同 DB 重启 -> 恢复同 digest -> 异 digest `LockConflict` 且原锁不变 -> 恢复 quorum -> 原交易成功 -> WAL 终态清空。最终结果为 `1 passed; 0 failed; 8 filtered out`。
3. 当前链完成 graceful shutdown、SIGKILL 和再次 graceful restart；identity/DB/chain id/checkpoint 均保持，入口数量不变。
4. owned transfer、shared Clock Move call、checkpoint、transaction/effects、events、system state、coins/balances 和 object/secondary index 推进均有在线 RPC 证据。
5. 三个部署脚本的共享 readiness shell 测试与 `bash -n` 通过；代码通过定向测试、debug/release build、局部 rustfmt、`cargo xclippy` 和 `git diff --check`。
6. 最终静态审计确认不存在 `RTD_REPAIR_ORPHAN_OBJECT_LOCKS`、`RTD_CLIENT_WAIT_FOR_EFFECTS_CERT`、validator coin-indexing 或前序高频诊断开关。
7. 扩展的 `cargo test -p rtd --lib` 中，本次新增的 6 个 DB 测试全部通过；其余 43 个 client PTB/upgrade snapshot 与 keytool 既有基线失败不属于本修复，未接受或改写相关 snapshot。具体输出和恢复审计见工作日志。

完整命令、TDD 红绿记录、checkpoint、transaction digest、PID 和 lockfile 恢复证据见 `worklog/README.md`。

## 11. 一句话定性

本问题是 `rtd start` 把“持久化 validator 集群”和“每次随机新建的临时 RPC fullnode”错误地组合在一起造成的重启状态分叉；正确修复是持久化 fullnode、阻止未追平 RPC 对外提供交易输入，并安全恢复同 digest pending transaction，而不是放宽对象版本校验、覆盖 object lock，或让客户端忽略本地执行。
