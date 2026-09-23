# RTD 钱包转账 lock conflict 实时排查记录

日期：2026-07-10

目标：持续排查钱包转账失败，直到可以使用 `rtd client` 向 `0xfbc95a1fbdd68117b26163e4068da6befc2fde9b02cb570b2351237bec447ba3` 成功转账 `600 RTD`。

## 当前失败

用户提供的新错误：

```text
Transaction is rejected as invalid by more than 1/3 of validators by stake (non-retriable).
Non-retriable errors: [Object (0x4a3ced14c55e220d6ffeefd39111ce80df6753c4ef40947aedebc8c26a8fa178, SequenceNumber(106), o#3dntckSuZt17DYCn5CRdKwfc18iyJxm5ATZC4JbiyCVX) already locked by a different transaction: TransactionDigest(5e513qAHtvfiGEyu6PfrGv5R2JHE26haYbbb1T2NxNXr) { k#99f25ef6.. } with 10000 stake].
```

初步判断：

- 旧问题是 RPC 返回落后的 gas coin version。
- 新问题已经变成 validator 上 gas coin version `106` 被另一个 transaction digest 锁住。
- 需要追踪 digest `5e513qAHtvfiGEyu6PfrGv5R2JHE26haYbbb1T2NxNXr` 是否有 certificate/effects、是否执行成功、为什么 lock 没有随执行或重启状态推进而释放。

## 2026-07-10 13:59 CST

已开始系统化排查。当前先收集证据，不直接猜测修复：

1. 搜索 lock conflict 产生代码。
2. 搜索本地日志和文档中是否已经出现 `5e513qAHtvfiGEyu6PfrGv5R2JHE26haYbbb1T2NxNXr`。
3. 查看当前工作区已有改动，避免覆盖前一次排查中的代码和诊断日志。

## 2026-07-10 14:00-14:08 CST

新增证据：

- 日志显示 `5e513qAHtvfiGEyu6PfrGv5R2JHE26haYbbb1T2NxNXr` 在 2026-07-09 14:53:58 被 `handle_vote_transaction accepted`。
- 同一请求随后在 transaction driver 侧先出现 `TimedOutSubmittingTransaction`，之后多次重试，最终在 2026-07-09 15:10:01 仍返回 `RpcError("transport error", "Unknown error")`。
- 2026-07-10 05:55:44 新交易 `ABmvy2fgVBJiGBxVV8fNpTgpnAZZYqb9jGLoeePEt4uq` 使用同一个 gas object version 106，被 validator 拒绝为 `ObjectLockConflict`，pending transaction 正是 `5e513...`。
- `rtd_getTransactionBlock(5e513...)` 和 `rtd client tx-block 5e513...` 都返回找不到 referenced transaction。
- 当前 `rtd_getObject` 返回 gas object version 106、digest `3dntckSuZt17DYCn5CRdKwfc18iyJxm5ATZC4JbiyCVX`，previous transaction 仍是 `9fWt8...`，说明 `5e513...` 没有产生 effects。
- 代码确认 `handle_vote_transaction` 调用 `handle_transaction_impl(transaction, sign=false, ...)`。该路径会写 owned object lock，但由于 `sign=false`，传给 `write_transaction_locks` 的 `signed_transaction` 是 `None`，不会写入 `signed_transactions`。

当前假设：

- 根因不是 fullnode 继续落后，而是 submit_transaction 路径在 consensus submit 成功前已经持久化 owned-object lock；当提交到 consensus 阶段超时或 transport error 时，lock 留在 validator DB 中，但没有 signed transaction/effects 可通过 RPC 恢复。
- 需要继续对比 Sui 原实现和 RTD 当前实现，确认正确语义应该是：要么只有 consensus 接受后才持久化 lock，要么 submit 失败时可恢复/回滚本次 submit 写入的 lock。

## 2026-07-10 14:23 CST

继续排查。当前工作区已经包含一个受显式开关保护的本地/debug 修复：

- 修改文件：`crates/rtd-core/src/execution_cache/object_locks.rs`
- 新增环境变量：`RTD_REPAIR_ORPHAN_OBJECT_LOCKS`
- 默认行为：不开启该环境变量时，原 `ObjectLockConflict` 行为保持不变。
- 开启行为：当新交易遇到旧 lock，且旧 lock 对应的 transaction digest 同时满足：
  - `epoch_store.get_signed_transaction(old_digest)` 返回 `None`
  - `epoch_store.is_pending_consensus_certificate(old_digest)` 返回 `false`
  才认为这是 submit 失败遗留的孤儿 lock，并允许用新交易 lock 替换。
- 新增单测：`test_orphan_object_lock_repair_requires_explicit_flag`
  - 验证默认不开启时仍拒绝 orphan lock conflict。
  - 验证测试开关开启时可替换没有 signed transaction、也不在 pending consensus 中的旧 lock。

已确认启动脚本 `toggle_local_rtd.sh` 使用 `env=os.environ.copy()` 启动 `rtd start`，所以通过：

```bash
RTD_REPAIR_ORPHAN_OBJECT_LOCKS=1 /Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/toggle_local_rtd.sh
```

可以把修复开关传入节点进程。

下一步：

1. 重新构建 `target/debug/rtd`。
2. 使用 `RTD_REPAIR_ORPHAN_OBJECT_LOCKS=1` 重启本地节点。
3. 查询当前 gas object 状态。
4. 使用 `rtd client transfer-rtd` 从已配置账户向 `0xfbc95a1fbdd68117b26163e4068da6befc2fde9b02cb570b2351237bec447ba3` 转账 `600 RTD`。
5. 过滤日志确认是否出现 `RTD_STATE_DIVERGENCE repairing orphan owned object lock`，并记录真实交易结果。

## 2026-07-10 14:34 CST

执行构建和重启：

```bash
cargo build -p rtd
RTD_REPAIR_ORPHAN_OBJECT_LOCKS=1 /Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/toggle_local_rtd.sh
```

结果：

- `cargo build -p rtd` 完成。
- 第一次执行 `toggle_local_rtd.sh` 因脚本是 toggle 语义，只停止了旧进程。
- 第二次执行成功启动新进程，PID 为 `49183`。
- RPC 在约 102 秒后 ready。
- readiness guard 等待 fullnode checkpoint catch-up 超时：
  - RPC checkpoint 从 `1339974` 推进到约 `1340625`
  - validator 启动目标高度为 `1418995`
  - 追平差距很大，短时间内无法等待完成。

当前 RPC 查询：

```text
rtd_getLatestCheckpointSequenceNumber => 1340699
rtd_getObject(0x4a3c...) => version=106, digest=3dntckSuZt17DYCn5CRdKwfc18iyJxm5ATZC4JbiyCVX, previousTransaction=9fWt8...
```

执行真实转账：

```bash
target/debug/rtd client transfer-rtd \
  --to 0xfbc95a1fbdd68117b26163e4068da6befc2fde9b02cb570b2351237bec447ba3 \
  --rtd-coin-object-id 0x4a3ced14c55e220d6ffeefd39111ce80df6753c4ef40947aedebc8c26a8fa178 \
  --amount 600000000000 \
  --gas-budget 5000000000 \
  --json
```

client 返回：

```text
Failed to confirm tx status for TransactionDigest(CzQbQQB8W4krhwthGKxPxDKLfSAskGYrtzV43yhsNacW) within 61 seconds.
```

日志证据：

- `CzQbQQB8W4krhwthGKxPxDKLfSAskGYrtzV43yhsNacW` 没有再被旧 lock 拒绝。
- 出现修复日志：

```text
RTD_STATE_DIVERGENCE repairing orphan owned object lock
obj_ref=(0x4a3ced14c55e220d6ffeefd39111ce80df6753c4ef40947aedebc8c26a8fa178, SequenceNumber(106), o#3dntckSuZt17DYCn5CRdKwfc18iyJxm5ATZC4JbiyCVX)
orphan_lock=TransactionDigest(5e513qAHtvfiGEyu6PfrGv5R2JHE26haYbbb1T2NxNXr)
replacement_lock=TransactionDigest(CzQbQQB8W4krhwthGKxPxDKLfSAskGYrtzV43yhsNacW)
```

- transaction driver/orchestrator 均显示 success：

```text
RTD transaction driver finalized transaction tx_digest=CzQb... effects_status=Success finality_info=QuorumExecuted(4)
RTD executeTransactionBlock RPC returning orchestrator response digest=CzQb... effects_status=Success finality_info=QuorumExecuted(4)
```

但随后 RPC 读路径仍然查不到该交易：

```text
rtd_getTransactionBlock(CzQb...) => Could not find the referenced transaction
rtd_getObject(0x4a3c...) => 仍返回 version=106
```

当前结论：

- 原始 `ObjectLockConflict` 根因已经被当前 debug 修复绕过，真实交易在 validator/orchestrator 路径拿到 `Success`。
- `rtd client` 命令仍然以失败码退出，是因为提交成功后确认交易状态依赖 read path/index/fullnode catch-up；当前 fullnode checkpoint 明显落后，导致 61 秒内查不到刚成功的交易。
- 下一步需要定位 `rtd client transfer-rtd` 的确认逻辑，以及本地 fullnode/index 为什么不能及时服务 `rtd_getTransactionBlock(CzQb...)` 和最新 gas object。

## 2026-07-10 14:45 CST

定位 `rtd client transfer-rtd` 失败原因：

- CLI 入口：`crates/rtd/src/client_commands.rs`
- 执行路径：`dry_run_or_execute_or_serialize` -> `WalletContext::execute_transaction_may_fail`
- SDK 入口：`crates/rtd-sdk/src/wallet_context.rs`
- `execute_transaction_may_fail` 固定传入 `ExecuteTransactionRequestType::WaitForLocalExecution`
- `crates/rtd-sdk/src/apis.rs` 中 JSON-RPC 的 `WaitForLocalExecution` 是客户端模拟：
  1. 先调用 RPC `executeTransactionBlock`
  2. RPC server 实际按 `WaitForEffectsCert` 提交并返回 effects cert
  3. SDK 再轮询 `getTransactionBlock(tx_digest)` 最多 60 秒
  4. 当前 fullnode/index 读路径大幅落后，所以第 3 步超时

新增本地 debug 开关：

- 修改文件：`crates/rtd-sdk/src/wallet_context.rs`
- 新增环境变量：`RTD_CLIENT_WAIT_FOR_EFFECTS_CERT`
- 默认行为：仍使用 `WaitForLocalExecution`，保持原 CLI 语义。
- 开启行为：当环境变量非空且不等于 `0` 时，`WalletContext::execute_transaction_may_fail` 使用 `WaitForEffectsCert`，不再额外轮询落后的 `getTransactionBlock`。
- 新增单测：
  - `test_execution_request_type_defaults_to_local_execution`
  - `test_execution_request_type_can_use_effects_cert_for_local_debug`

测试过程：

- 先添加测试，确认红灯失败：`execution_request_type_for_cli` 不存在。
- 实现 helper 后重新执行，测试已注册：

```text
wallet_context::tests::test_execution_request_type_can_use_effects_cert_for_local_debug: test
wallet_context::tests::test_execution_request_type_defaults_to_local_execution: test
```

重新构建：

```bash
cargo build -p rtd
```

下一步：

- 等待 read path 追到前一笔成功交易 `CzQbQQB8W4krhwthGKxPxDKLfSAskGYrtzV43yhsNacW`，避免用 stale gas object version 重发。
- 之后用：

```bash
RTD_CLIENT_WAIT_FOR_EFFECTS_CERT=1 target/debug/rtd client transfer-rtd ...
```

执行下一笔 `600 RTD` 转账，目标是让 `rtd client` 命令本身以成功状态返回。

## 2026-07-10 15:11 CST

为避免重复转账，先固定 gas price 并重建 transaction digest：

```bash
target/debug/rtd client transfer-rtd \
  --to 0xfbc95a1fbdd68117b26163e4068da6befc2fde9b02cb570b2351237bec447ba3 \
  --rtd-coin-object-id 0x4a3ced14c55e220d6ffeefd39111ce80df6753c4ef40947aedebc8c26a8fa178 \
  --amount 600000000000 \
  --gas-budget 5000000000 \
  --gas-price 1000 \
  --tx-digest
```

输出：

```text
CzQbQQB8W4krhwthGKxPxDKLfSAskGYrtzV43yhsNacW
```

该 digest 与前一笔已经获得 effects cert success 的交易完全相同，因此可以安全地幂等重提，不会生成第二笔不同 digest 的转账。

使用新的 client debug 开关执行：

```bash
RTD_CLIENT_WAIT_FOR_EFFECTS_CERT=1 target/debug/rtd client transfer-rtd \
  --to 0xfbc95a1fbdd68117b26163e4068da6befc2fde9b02cb570b2351237bec447ba3 \
  --rtd-coin-object-id 0x4a3ced14c55e220d6ffeefd39111ce80df6753c4ef40947aedebc8c26a8fa178 \
  --amount 600000000000 \
  --gas-budget 5000000000 \
  --gas-price 1000 \
  --json
```

结果：命令 exit code 为 `0`，交易成功。

关键返回值：

```text
digest=CzQbQQB8W4krhwthGKxPxDKLfSAskGYrtzV43yhsNacW
effects.status=success
executedEpoch=4
gas object:
  objectId=0x4a3ced14c55e220d6ffeefd39111ce80df6753c4ef40947aedebc8c26a8fa178
  version=107
  digest=Heo8EvVSR8o3brZxKGpSfT2jWRPWA9cfVaDR6ZwZSBBZ
created recipient coin:
  objectId=0xcf7057c8eaea560c9ff59b02f222f4206778eea6099b88227ee15945440194a3
  version=107
  digest=9uUugrLb7EaaTMBH3nKuxg1a4dPuCQrLYzt8ATwjD49u
recipient balance change=+600000000000
sender balance change=-600001997880
confirmedLocalExecution=false
```

其中 `600000000000` 最小单位等于 `600 RTD`。

最终结论：

1. validator 的旧 `ObjectLockConflict` 由孤儿 owned-object lock 导致。
2. `RTD_REPAIR_ORPHAN_OBJECT_LOCKS=1` 允许本地 debug 节点在严格条件下替换无 signed transaction、无 pending consensus certificate 的孤儿 lock。
3. 交易第一次提交已经获得 `Success / QuorumExecuted(4)`，但 fullnode/index 读路径仍大幅落后。
4. `rtd client` 默认模拟 `WaitForLocalExecution`，因轮询落后 read path 而在 60 秒后错误退出。
5. `RTD_CLIENT_WAIT_FOR_EFFECTS_CERT=1` 让本地调试 CLI 直接以 effects cert 为成功条件。
6. 已使用 `rtd client transfer-rtd` 成功向目标地址转账 `600 RTD`。

## 2026-07-10 15:22 CST

最终验证：

```bash
rustfmt --edition 2024 --check crates/rtd-core/src/execution_cache/object_locks.rs crates/rtd-sdk/src/wallet_context.rs
cargo build -p rtd
target/debug/deps/rtd_sdk-46ba8c11666ba856 execution_request_type --nocapture
target/debug/deps/rtd_core-53a01d0b2f1ff541 test_orphan_object_lock_repair_requires_explicit_flag --nocapture
cargo xclippy
```

结果：

- 局部 rustfmt 检查通过。
- `cargo build -p rtd` 通过。
- `rtd-sdk` 两个 request type 单测通过：`2 passed; 0 failed`。
- `rtd-core` 孤儿 lock 修复单测通过：`1 passed; 0 failed`。
- `cargo xclippy` 全仓通过：`Finished dev profile`，exit code 为 `0`。

注意：

- 全仓 `cargo fmt --all -- --check` 仍会报告大量无关文件 import 排序差异，这些不是本次修改引入的文件，未做批量格式化，避免扩大改动面。
- 本次只格式化并检查了修改过的 Rust 文件。
