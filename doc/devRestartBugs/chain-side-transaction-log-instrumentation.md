# RTD dev 链重启后交易不落链的链侧日志定位记录

## 当前证据

钱包侧已确认不再提交旧 gas object ref：

```text
gas payment after build:
objectId = 0x4a3ced14c55e220d6ffeefd39111ce80df6753c4ef40947aedebc8c26a8fa178
version = 105
```

`105` 十进制等于 `0x69`，与链上 `rtd_getObject` 返回的当前版本一致。因此最早的
`Version 0x68 ... current version: 0x69` 旧对象版本问题已经不再是本轮转账失败的直接原因。

新的现象是：

- 钱包调用 `executeTransactionBlock` 后拿到了 digest，例如 `9fWt8oeVio9KDd9zeGA1oKpNjUcfmYAxaVRBZiRWjL1M`。
- 返回值里 `confirmedLocalExecution = false`。
- 很久后通过 `rtd_getTransactionBlock` 查询该 digest 仍返回找不到。
- gas object 的 `version` 和 `previousTransaction` 没有推进，说明该 digest 对应的 effects 没有在本 fullnode 可见状态中落盘。

这已经从“钱包提交旧 object ref”转为“RTD 节点交易提交/确认/本地执行链路中，某一步返回给 RPC 但没有形成可查询交易”的问题。

## 为什么需要 RTD 链侧日志

`confirmedLocalExecution = false` 在源码语义上不一定是错误，因为 JSON-RPC 默认 request type 是 `WaitForEffectsCert`，不是 `WaitForLocalExecution`。

但本地单节点 dev 环境中，如果 RPC 返回 digest 后长期查不到交易，就必须确认以下边界：

1. JSON-RPC 是否真的把同一个 digest 交给 `TransactionOrchestrator`。
2. `TransactionOrchestrator` 是从本地 effects future 返回，还是从 `TransactionDriver` 返回。
3. `TransactionDriver` 是否拿到了 validator 的 finalized effects。
4. `EffectsCertifier` 是否收到了 acknowledgments 和 full effects。
5. 返回给钱包的 effects 里的 transaction digest 是否等于 RPC 请求 digest。

这些边界只靠钱包日志无法判断，所以本次在 RTD 链侧添加了诊断日志。

## 已添加的日志位置

```text
crates/rtd-json-rpc/src/transaction_execution_api.rs
crates/rtd-core/src/transaction_orchestrator.rs
crates/rtd-core/src/transaction_driver/mod.rs
crates/rtd-core/src/transaction_driver/effects_certifier.rs
```

主要日志关键字：

```text
RTD executeTransactionBlock RPC accepted request
RTD executeTransactionBlock RPC returning orchestrator response
RTD transaction orchestrator started
RTD transaction orchestrator effects waiting started
RTD transaction orchestrator prepared submission
RTD transaction driver started
RTD transaction driver submitter returned
RTD effects certifier started
RTD effects certifier collected acknowledgments and full effects
RTD transaction driver effects certifier returned success
RTD TransactionDriver returned success
RTD transaction driver returned finalized response
RTD transaction orchestrator finished
```

这些日志都会带 `tx_digest`、`effects_status`、`finality_info`、`elapsed_ms` 等字段。

## 启动脚本调整

脚本：

```text
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/toggle_local_rtd.sh
```

已把默认 `RUST_LOG` 从：

```text
off,rtd_node=info
```

扩展为：

```text
off,rtd_node=info,rtd_json_rpc::transaction_execution_api=info,rtd_core::transaction_orchestrator=info,rtd_core::transaction_driver=info,rtd_core::transaction_driver::effects_certifier=info
```

脚本仍然只停止/启动进程，不删除任何本地链数据。

## 下一次复现时的判断方法

重启 RTD 后，再从钱包发起一笔转账，然后在日志中按钱包返回的 digest 搜索：

```bash
rg '9fWt8oeVio9KDd9zeGA1oKpNjUcfmYAxaVRBZiRWjL1M|RTD executeTransactionBlock|RTD transaction driver|RTD effects certifier' \
  /Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/rtd-local-node.log
```

如果能看到 `effects certifier returned success` 和 `orchestrator finished`，但随后 `rtd_getTransactionBlock` 仍查不到同一 digest，则问题在“已返回 finalized effects 之后，本 fullnode 的执行缓存/RPC index/checkpoint 可见性没有落盘或没有追上”。

如果日志停在 `submitter returned` 或 `effects certifier collected... full_effects_ok=false`，则问题在 validator effects 获取或确认阶段。

如果 RPC 入口日志中的 digest 和钱包返回 digest 不一致，则需要回到钱包/SDK 侧查签名和提交的 transaction bytes。

## 验证

已执行：

```bash
rustfmt --edition 2024 --config skip_children=true --check \
  crates/rtd-json-rpc/src/transaction_execution_api.rs \
  crates/rtd-core/src/transaction_orchestrator.rs \
  crates/rtd-core/src/transaction_driver/mod.rs \
  crates/rtd-core/src/transaction_driver/effects_certifier.rs

bash -n /Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/toggle_local_rtd.sh

git diff --check -- \
  crates/rtd-json-rpc/src/transaction_execution_api.rs \
  crates/rtd-core/src/transaction_orchestrator.rs \
  crates/rtd-core/src/transaction_driver/mod.rs \
  crates/rtd-core/src/transaction_driver/effects_certifier.rs

cargo check -p rtd-core -p rtd-json-rpc
cargo build --release -p rtd
```

`cargo build --release -p rtd` 已通过，仅有 Move 依赖中的既有 warning。
