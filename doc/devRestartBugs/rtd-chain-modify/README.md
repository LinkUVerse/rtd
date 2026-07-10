# RTD 链侧修改审计记录

记录时间：2026-07-09

## 1. 背景

本轮排查的问题是：本地 dev 环境中的 RTD 链停机约 15 小时后重新启动，钱包转账曾出现旧对象版本错误：

```text
Object ID 0x4a3ced14c55e220d6ffeefd39111ce80df6753c4ef40947aedebc8c26a8fa178
Version 0x68 ... is not available for consumption, current version: 0x69
```

钱包侧后续已改为在签名前重新从 RPC 拉取 coin object refs，并把 fresh refs 写入交易 gas payment。用户提供的浏览器日志显示，签名前使用的 gas payment 已变为：

```text
objectId = 0x4a3ced14c55e220d6ffeefd39111ce80df6753c4ef40947aedebc8c26a8fa178
version = 105
digest = HH6vpAooefiJhizMRpaufLU4hzgGLz2GtEWgiUEV2rAL
```

`105` 十进制等于 `0x69`，与链上当前对象版本一致。因此最早的旧 gas object ref 问题已经从钱包侧被规避。

新的现象是：钱包调用 `executeTransactionBlock` 能拿到响应 digest，例如：

```text
9fWt8oeVio9KDd9zeGA1oKpNjUcfmYAxaVRBZiRWjL1M
confirmedLocalExecution = false
```

但后续链上查询不到该 digest，且 gas object 的 `version` 与 `previousTransaction` 未推进。这个现象需要链侧日志确认交易在 JSON-RPC、TransactionOrchestrator、TransactionDriver、EffectsCertifier 之间到底停在哪一步。

## 2. 本次修改范围

本次 RTD 链侧修改只做诊断日志增强和启动脚本默认日志级别调整：

- 没有修改交易构造逻辑。
- 没有修改对象版本校验逻辑。
- 没有修改共识、执行、effects 写入、checkpoint 或索引逻辑。
- 没有新增任何删除本地链数据的操作。
- 没有修改本地数据目录，例如 `~/.rtd/rtd_config`。
- 没有自动重启用户当前正在运行的 RTD 进程。

## 3. 修改文件总览

RTD 仓库内修改：

```text
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/rtd/crates/rtd-json-rpc/src/transaction_execution_api.rs
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/rtd/crates/rtd-core/src/transaction_orchestrator.rs
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/rtd/crates/rtd-core/src/transaction_driver/mod.rs
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/rtd/crates/rtd-core/src/transaction_driver/effects_certifier.rs
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/rtd/doc/devRestartBugs/chain-side-transaction-log-instrumentation.md
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/rtd/doc/devRestartBugs/rtd-chain-modify/README.md
```

RTD 仓库外修改：

```text
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/toggle_local_rtd.sh
```

钱包仓库历史修改也与本次定位相关，但不属于本目录的链侧源码修改，详见本文第 9 节。

## 4. JSON-RPC 入口日志

文件：

```text
crates/rtd-json-rpc/src/transaction_execution_api.rs
```

新增导入：

```rust
use tracing::{info, instrument};
```

新增日志 1：

```text
RTD executeTransactionBlock RPC accepted request
```

记录字段：

```text
digest
sender
request_type
input_object_count
include_events
include_input_objects
include_output_objects
show_effects
show_input
```

诊断目的：

- 确认钱包提交到 JSON-RPC 的 transaction digest。
- 确认 RPC 层解析出的 sender 和 input object 数量。
- 确认钱包实际请求的 response options。
- 确认用户看到的钱包 digest 与链侧入口 digest 是否一致。

新增日志 2：

```text
RTD executeTransactionBlock RPC returning orchestrator response
```

记录字段：

```text
digest
response_tx_digest
effects_status
finality_info
is_executed_locally
```

诊断目的：

- 确认 JSON-RPC 返回给钱包的 effects digest 是否与请求 digest 一致。
- 确认 RPC 返回前是否拿到了 orchestrator 响应。
- 确认返回状态是本地执行完成，还是只拿到 effects certificate。

## 5. TransactionOrchestrator 日志

文件：

```text
crates/rtd-core/src/transaction_orchestrator.rs
```

新增日志关键字：

```text
RTD transaction orchestrator started
RTD transaction orchestrator finished
RTD transaction orchestrator effects waiting started
RTD transaction orchestrator prepared submission
RTD transaction effects became locally available while driver was running
RTD transaction orchestrator returning local effects
RTD transaction driver returned finalized response
RTD transaction execution attempt failed, waiting for other attempts
RTD timeout waiting for transaction finality
RTD timeout waiting for transaction finality without recorded execution error
RTD transaction orchestrator implementation received request
RTD TransactionDriver returned success
RTD TransactionDriver returned error
```

重点记录字段：

```text
tx_digest
request_type
tx_type
executed_locally
response_tx_digest
effects_status
finality_info
elapsed_ms
include_events
include_input_objects
include_output_objects
enforce_live_input_objects
is_new_transaction
num_submissions
finality_timeout_secs
epoch
error
```

诊断目的：

- 判断交易是否进入 orchestrator。
- 判断 request type 是 `WaitForEffectsCert` 还是 `WaitForLocalExecution`。
- 判断 orchestrator 是从本地 effects future 返回，还是从 TransactionDriver 返回。
- 判断是否出现 finality timeout。
- 判断 TransactionDriver 是否返回成功但本地 RPC 后续仍查不到交易。
- 判断 effects digest 与入口 digest 是否一致。

这组日志是定位“RPC 已返回 digest，但 fullnode 查询不到交易”的核心边界。

## 6. TransactionDriver 日志

文件：

```text
crates/rtd-core/src/transaction_driver/mod.rs
```

新增导入：

```rust
use rtd_types::effects::TransactionEffectsAPI as _;
use tracing::{info, instrument};
```

新增日志关键字：

```text
RTD transaction driver started
RTD transaction driver finalized transaction
RTD transaction driver submitter returned
RTD transaction driver effects certifier returned success
RTD transaction driver effects certifier returned error
```

重点记录字段：

```text
tx_digest
tx_type
ping
timeout_secs
amplification_factor
attempts
elapsed_ms
effects_status
finality_info
authority
submit_result_kind
error
```

`submit_result_kind` 可能值：

```text
submitted
executed_fast_path
executed
rejected
```

诊断目的：

- 判断交易是否被 driver 提交到 validator。
- 判断 validator 返回的是已提交、已执行、fast path 已执行，还是 rejected。
- 判断 effects certifier 是否拿到 certified/finalized effects。
- 判断 driver 返回成功后，orchestrator/RPC 是否继续正常返回。

## 7. EffectsCertifier 日志

文件：

```text
crates/rtd-core/src/transaction_driver/effects_certifier.rs
```

新增导入：

```rust
use tracing::{info, instrument};
```

新增日志关键字：

```text
RTD effects certifier started
RTD effects certifier collected acknowledgments and full effects
```

重点记录字段：

```text
tx_digest
tx_type
initial_target
returned_target
has_consensus_position
has_initial_full_effects
acknowledgments_ok
full_effects_ok
```

诊断目的：

- 判断 effects 认证流程是否启动。
- 判断是否收集到 validator acknowledgments。
- 判断是否收集到 full effects。
- 判断问题是在 validator effects 获取阶段，还是在 effects 已经返回后的本地可见性阶段。

## 8. 启动脚本修改

文件：

```text
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/toggle_local_rtd.sh
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/deploy_local_all.sh
```

新增默认日志变量：

```bash
DEFAULT_RUST_LOG="off,rtd_node=info,rtd_json_rpc::transaction_execution_api=info,rtd_core::transaction_orchestrator=info,rtd_core::transaction_driver=info,rtd_core::transaction_driver::effects_certifier=info"
```

启动 RTD 时的环境变量设置从固定值改为可覆盖默认值：

```bash
RUST_LOG="${RUST_LOG:-${DEFAULT_RUST_LOG}}" python3 - "$RTD_BIN_RESOLVED" "$FULLNODE_RPC_PORT" "$RTD_NODE_LOG" <<'PY'
```

效果：

- 默认会打开本轮新增链侧诊断日志。
- 用户仍可在启动脚本前通过外部 `RUST_LOG=...` 覆盖默认值。
- 脚本仍然是 toggle 模式：检测到已有 `rtd` 进程时只停止进程；没有进程时才启动。
- 脚本仍然使用已有数据启动：`rtd start --fullnode-rpc-port ...`。
- 脚本没有添加任何 `rm -rf`、数据目录重建、数据库清理或 genesis 重置操作。

2026-07-09 后续调整：为了缩短本地定位迭代时间，`toggle_local_rtd.sh` 和 `deploy_local_all.sh` 的 RTD 二进制选择策略已改为 debug 优先：

```text
1. RTD_BIN 环境变量显式指定的二进制
2. ${RTD_SOURCE_DIR}/target/debug/rtd
3. ${RTD_SOURCE_DIR}/target/release/rtd
4. PATH 中的 rtd（仅 deploy_local_all.sh 保留此兜底）
```

如果 debug 二进制不存在，脚本会输出 warning 并回退到 release 或 PATH 中的 `rtd`。因此后续本地功能定位建议先执行：

```bash
cargo build -p rtd
```

不再默认要求：

```bash
cargo build --release -p rtd
```

这个调整只影响启动时选择哪个本地二进制，不改变链上数据、genesis、RPC 参数或交易处理逻辑。

日志输出路径：

```text
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/rtd-local-node.log
```

## 9. 钱包侧相关修改摘要

钱包仓库路径：

```text
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/rtd-apps
```

钱包侧修改不是本目录 `rtd-chain-modify` 的链侧源码修改，但它是本次排查的前置条件。当前钱包仓库里与本问题相关的改动包括：

```text
core/src/hooks/useFormatCoin.ts
core/src/hooks/useGetCoins.ts
wallet/src/ui/app/WalletSigner.ts
wallet/src/ui/app/components/navigation/index.tsx
wallet/src/ui/app/hooks/useGetAllCoins.ts
wallet/src/ui/app/hooks/useGetAllCoins.test.ts
wallet/src/ui/app/pages/home/transfer-coin/SendTokenForm.tsx
wallet/src/ui/app/pages/home/transfer-coin/index.tsx
wallet/src/ui/app/pages/home/transfer-coin/utils/transaction.ts
wallet/src/ui/index.tsx
```

钱包侧主要目的：

- `useGetCoins`、`useGetAllCoins`：coin refs 查询设置 `staleTime: 0`、`gcTime: 0`、`meta: { skipPersistedCache: true }`，避免停机重启后继续使用旧持久化缓存。
- `useGetAllCoins`：抽出 `fetchAllCoins` 和 `getAllCoinsQueryOptions`，使转账前可以强制重新拉取所有 coin refs。
- `transfer-coin/index.tsx`：签名前调用 `fetchAllCoins` 获取 fresh coins，构造 fresh transaction；遇到对象版本不可用错误时清理 coin 相关 query 后重试一次。
- `transaction.ts`：显式把 fresh coin refs 写入 `tx.setGasPayment(...)`，包括 RTD 主币 transfer 使用的 gas coin。
- `WalletSigner.ts`：打印 build 前后 transaction data、gas payment、签名、执行请求和执行响应，确认 SDK build 后的 gas payment 是否已刷新。
- `ui/index.tsx` 和底部 navigation：添加弹窗页面和底部 tab 的 `console.info` 诊断日志，用于确认浏览器扩展页面实际载入的是新 dist。
- `PersistQueryClientProvider` 的 `shouldDehydrateQuery`：只持久化 `state.status === 'success'` 且未标记 `skipPersistedCache` 的 query，规避 `Promise could not be cloned` 类型的持久化错误。
- `useGetAllCoins.test.ts`：覆盖分页获取所有 coins 以及 coin refs query 不持久化、立即 stale 的行为。

钱包侧已解决的证据：

- 浏览器日志显示 gas payment version 已经是 `105`，即链上当前版本 `0x69`。
- 因此旧对象版本错误不再是当前“execute response 已返回但链上查不到交易”的直接原因。

## 10. 编译和验证记录

已执行 Rust 格式检查：

```bash
rustfmt --edition 2024 --config skip_children=true --check \
  crates/rtd-json-rpc/src/transaction_execution_api.rs \
  crates/rtd-core/src/transaction_orchestrator.rs \
  crates/rtd-core/src/transaction_driver/mod.rs \
  crates/rtd-core/src/transaction_driver/effects_certifier.rs
```

结果：通过。

## 2026-07-09 21:41 CST：coin index 滞后根因定位与窄修复

### 新证据

钱包转账失败链路已经排除钱包侧 stale cache：

- `rtdx_getCoins` 返回 gas coin version 20。
- `rtd_getObject` 返回同一个 gas coin version 21。
- `rtd_getObject` 新增的 `authority_cache_live` 同样是 version 21，说明 object store / authority execution cache 已经更新。
- 查询 `previous_transaction=7Dff6H2MM3DYfupNcdT5VDtomKLQJzz7sSohnukbMd6Q` 得到的 effects 明确显示：
  - gas input payment 为 version 20。
  - `mutated` / `gasObject` 输出为 version 21。
  - `objectChanges` 中该 coin 为 `previousVersion: "20"` -> `version: "21"`。

因此根因收敛为：交易执行已经把 gas coin 更新到新版本，但 JSON-RPC coin index 没有跟随更新，导致 `rtdx_getCoins` 后续仍返回旧 gas object ref。

### 根因候选

`crates/rtd-core/src/authority.rs` 中 `fullnode_only_get_tx_coins_for_indexing` 原逻辑：

```rust
if self.indexes.is_none() || self.is_validator(epoch_store) {
    return None;
}
```

这继承了 Sui 的 fullnode-only 假设：validator 不需要维护钱包 RPC 依赖的 coin index。

但当前本地 RTD 是 all-in-one validator + JSON-RPC 运行模式：

- `IndexStore` 存在，`rtdx_getCoins` 依赖它。
- 节点同时在 committee 中，因此 `self.is_validator(epoch_store)` 为 true。
- 结果是交易执行后 object store 更新，但 coin index 更新时 `tx_coins=None`，`index_coin` 直接跳过。

这解释了 `getObject` / authority cache 已经是新版本，而 `getCoins` 仍返回旧版本的现象。

### 本轮修改

1. 将 `fullnode_only_get_tx_coins_for_indexing` 改为 `get_tx_coins_for_indexing`。
2. 只在 `self.indexes.is_none()` 时跳过 coin 收集；不再因为节点同时是 validator 而跳过。
3. 在 `crates/rtd-core/src/jsonrpc_index.rs` 的 `index_coin` 中新增日志：

```text
RTD_STATE_DIVERGENCE coin index update
```

该日志打印：

- `tx_digest`
- `delete_count`
- `add_count`
- `delete_keys`
- `add_keys`

用于验证每笔交易是否删除旧 coin index key 并插入新 coin index key。

4. 本地启动脚本 `DEFAULT_RUST_LOG` 新增：

```text
rtd_core::jsonrpc_index=info
```

### 已执行验证

```bash
rustfmt --edition 2024 crates/rtd-core/src/authority.rs crates/rtd-core/src/jsonrpc_index.rs

bash -n /Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/toggle_local_rtd.sh \
  /Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/deploy_local_all.sh

cargo build -p rtd
```

结果：通过。

### 下一步自动验证计划

1. 使用 `toggle_local_rtd.sh` 非破坏性重启本地节点。
2. 调用 `rtdx_getCoins` 和 `rtd_getObject` 记录初始 version。
3. 使用 `rtd client transfer-rtd` 从 active address `0xc535a846ad8aecf2c353c12b557612f0f1ae3bb09ba7cd2c6c8fa6fa56bf0df9` 向 `0xfbc95a1fbdd68117b26163e4068da6befc2fde9b02cb570b2351237bec447ba3` 转少量 RTD。
4. 过滤日志确认出现 `RTD_STATE_DIVERGENCE coin index update`。
5. 再次调用 `rtdx_getCoins` 和 `rtd_getObject`，确认 coin index version 与 object live version 一致。

已执行启动脚本语法检查：

```bash
bash -n /Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/toggle_local_rtd.sh
```

结果：通过。

已执行 diff 空白检查：

```bash
git diff --check -- \
  crates/rtd-json-rpc/src/transaction_execution_api.rs \
  crates/rtd-core/src/transaction_orchestrator.rs \
  crates/rtd-core/src/transaction_driver/mod.rs \
  crates/rtd-core/src/transaction_driver/effects_certifier.rs
```

结果：通过。

已执行定向 check：

```bash
cargo check -p rtd-core -p rtd-json-rpc
```

结果：通过。

已执行 release 构建：

```bash
cargo build --release -p rtd
```

结果：通过。构建期间仅出现 Move 相关既有 warning，例如：

```text
move-regex-borrow-graph: unused variables other, r; dead method check_invariants
move-compiler: unused variable tv
```

release 产物：

```text
target/release/rtd
size = 174519248 bytes
mtime = 2026-07-09 16:57:11 CST
```

后续为了缩短本地定位迭代时间，脚本已改为默认使用 debug 二进制。当前建议的构建命令是：

```bash
cargo build -p rtd
```

如果 debug 首次构建卡在 `librocksdb-sys` 等 native 依赖编译阶段，属于构建耗时问题，不代表脚本选择逻辑失败。脚本会在 `${RTD_SOURCE_DIR}/target/debug/rtd` 存在时优先使用它。

## 11. 当前运行状态说明

前一次 release 构建生成了：

```text
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/rtd/target/release/rtd
```

后续脚本默认优先使用：

```text
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/rtd/target/debug/rtd
```

如果 debug 二进制不存在，则回退到 release 二进制。但脚本不会自动替换正在运行的进程。若修改前的 `rtd` 进程仍在运行，它仍然是旧二进制逻辑，不会输出本轮新增日志。

由于 `toggle_local_rtd.sh` 是 toggle 模式：

1. 如果当前已有 `rtd` 进程，第一次执行脚本会停止进程。
2. 第二次执行脚本才会启动当前可用的 `target/debug/rtd`，debug 不存在时才回退到 `target/release/rtd`。

这个过程不会删除本地链数据。

## 12. 复现和日志检索方式

重启链并从钱包发起一笔转账后，用钱包返回的 digest 检索链侧日志：

```bash
rg '9fWt8oeVio9KDd9zeGA1oKpNjUcfmYAxaVRBZiRWjL1M|RTD executeTransactionBlock|RTD transaction orchestrator|RTD transaction driver|RTD effects certifier' \
  /Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/rtd-local-node.log
```

判断方法：

- 如果能看到 `RTD executeTransactionBlock RPC accepted request`，说明钱包请求已进入链侧 RPC。
- 如果看不到入口日志，说明钱包请求没有打到当前脚本启动的节点，或当前运行的不是新构建二进制。
- 如果看到 `RTD transaction driver submitter returned` 且 `submit_result_kind = rejected`，说明 validator 在提交阶段拒绝交易。
- 如果看到 `RTD effects certifier collected acknowledgments and full effects` 且 `full_effects_ok = false`，说明问题在 effects 获取或认证阶段。
- 如果看到 `RTD transaction driver effects certifier returned success`、`RTD TransactionDriver returned success`、`RTD executeTransactionBlock RPC returning orchestrator response`，但 `rtd_getTransactionBlock` 长期查不到同一 digest，问题更可能在 fullnode 本地执行落盘、RPC 读路径、checkpoint/index 可见性或本地状态同步阶段。
- 如果 `digest` 和 `response_tx_digest` 不一致，需要回到钱包/SDK 的 transaction bytes 和签名路径继续核对。

## 13. 风险和回滚

风险：

- 新增 `info` 日志会增加 `rtd-local-node.log` 的体积。
- 日志包含 transaction digest、sender、authority、effects 状态等诊断信息，不包含私钥或助记词。
- 默认 `RUST_LOG` 目前关闭了大部分模块，只打开本次相关模块，日志量应可控。

临时关闭新增日志：

```bash
RUST_LOG=off,rtd_node=info /Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/toggle_local_rtd.sh
```

回滚方式：

- 撤回 4 个 Rust 文件中的 `tracing::info` 日志新增代码和相关导入。
- 撤回 `toggle_local_rtd.sh` 中的 `DEFAULT_RUST_LOG` 变量，以及启动处的 `RUST_LOG="${RUST_LOG:-${DEFAULT_RUST_LOG}}"` 调整。
- 删除或保留文档均不影响链运行。

## 14. 审计结论

本次 RTD 链侧修改属于非行为型诊断改动。它的目标是定位“钱包已提交 fresh gas object ref，RPC 已返回 execute response，但链上查不到交易”的链侧断点。

截至本文记录时，链侧业务逻辑没有被修复或改变；共识、执行、对象版本推进、effects 写入和 checkpoint/index 流程均保持原逻辑。若后续日志证明 TransactionDriver 已返回 success 但 RPC 查询长期不可见，才需要继续进入 fullnode local execution、transaction cache、authority store、checkpoint executor 或 index 读路径做下一轮定位。

## 15. 2026-07-09 精准分界日志补充

为了定位 `rtd_getObject`/钱包读到 object version `100`，但 validator 执行路径报 `current version: 0x6a` 的状态不一致问题，后续又补充了更容易按单笔交易提取的分界日志。

新增 RPC 层分界线：

```text
================ RTD_TX_TRACE_BEGIN executeTransactionBlock ================
================ RTD_TX_TRACE_END executeTransactionBlock success ================
================ RTD_TX_TRACE_END executeTransactionBlock error ================
```

RPC 入口日志 `RTD executeTransactionBlock RPC accepted request` 额外记录：

```text
input_objects
raw_transaction_bytes
```

其中 `input_objects` 用来确认 JSON-RPC 收到的交易输入对象 ref 是否就是钱包打印的版本，例如 `version = 100 / 0x64`。

新增 driver 层分界线：

```text
================ RTD_TX_TRACE_DRIVER_BEGIN ================
================ RTD_TX_TRACE_DRIVER_END success ================
================ RTD_TX_TRACE_DRIVER_END non_retriable_error ================
```

新增 driver 失败结构化日志：

```text
RTD transaction driver user transaction retriable failure
RTD transaction driver user transaction non-retriable failure
```

新增 orchestrator 错误收口日志：

```text
RTD transaction orchestrator finished with execution error
RTD executeTransactionBlock RPC orchestrator error
```

这些日志只用于诊断，不改变交易执行、对象版本校验或本地数据。

推荐提取命令：

```bash
LOG=/Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/rtd-local-node.log

rg 'RTD_TX_TRACE|RTD executeTransactionBlock RPC accepted request|RTD transaction driver user transaction|RTD transaction orchestrator finished with execution error|current version|is not available for consumption' "$LOG"
```

本次补充后已执行：

```bash
rustfmt --edition 2024 --config skip_children=true --check \
  crates/rtd-json-rpc/src/transaction_execution_api.rs \
  crates/rtd-core/src/transaction_orchestrator.rs \
  crates/rtd-core/src/transaction_driver/mod.rs

git diff --check -- \
  crates/rtd-json-rpc/src/transaction_execution_api.rs \
  crates/rtd-core/src/transaction_orchestrator.rs \
  crates/rtd-core/src/transaction_driver/mod.rs

cargo build -p rtd
```

结果：通过。debug 产物：

```text
target/debug/rtd
size = 722520368 bytes
mtime = 2026-07-09 17:27:11 CST
```

## 16. 2026-07-09 状态分裂日志补充

后续复现确认：

```text
rtdx_getCoins 返回 gas object version = 100
rtd_getObject 返回 gas object version = 100
rtd_tryGetPastObject(version=106) 返回 VersionTooHigh latest_version = 100
交易执行校验返回 current version = 0x6a = 106
```

同时观察到 RTD Explorer `http://localhost:3000/` 的 Epoch 卡在旧值。Explorer 源码中 Epoch 主要来自：

```text
rtd-apps/rtd-explorer/src/explorer/src/pages/epochs/utils.ts
rtd-apps/rtd-explorer/src/core/src/hooks/useGetTimeBeforeEpochNumber.ts
```

这些组件使用：

```text
useRtdClientQuery("getLatestRtdSystemState")
```

链侧实现位于：

```text
crates/rtd-json-rpc/src/governance_api.rs
crates/rtd-json-rpc/src/authority_state.rs
crates/rtd-core/src/authority/authority_store.rs
```

其中 `AuthorityStore::get_rtd_system_state_object_unsafe` 的注释说明它直接读 DB，reconfiguration 同时发生时不保证读到新旧哪一个系统状态。因此 Explorer Epoch 卡住是有价值的旁证：重启后不只是钱包 gas object，系统状态/epoch 读源也可能与执行路径不同步。

为定位状态来源，新增统一关键字：

```text
RTD_STATE_DIVERGENCE
```

新增日志位置：

```text
crates/rtd-json-rpc/src/read_api.rs
crates/rtd-json-rpc/src/coin_api.rs
crates/rtd-json-rpc/src/governance_api.rs
crates/rtd-core/src/authority.rs
crates/rtd-core/src/execution_cache/object_locks.rs
crates/rtd-core/src/authority/authority_store.rs
```

新增日志含义：

```text
RTD_STATE_DIVERGENCE getObject read path returned object
```

记录 `rtd_getObject` 读路径返回的 `object_ref`、`object_version`、`object_digest`、`previous_transaction`。

```text
RTD_STATE_DIVERGENCE getCoins read path returned coins
```

记录 `rtdx_getCoins` 返回的钱包 coin refs。

```text
RTD_STATE_DIVERGENCE getLatestRtdSystemState read path returned system state
```

记录 Explorer 依赖的系统状态 epoch、protocol version、system state version、epoch 起始时间和持续时间。

```text
RTD_STATE_DIVERGENCE check_transaction_validity live object comparison
RTD_STATE_DIVERGENCE object lock live object mismatch
RTD_STATE_DIVERGENCE live owned object marker missing requested version
```

记录执行校验、object lock、live owned object marker 看到的 live object 或 latest live lock 版本，用来确认 `current version: 0x6a` 到底来自 execution cache、object lock，还是 live marker 表。

推荐提取命令：

```bash
LOG=/Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/rtd-local-node.log

rg 'RTD_STATE_DIVERGENCE|RTD_TX_TRACE|current version|is not available for consumption|getLatestRtdSystemState' "$LOG"
```

本次补充后已执行：

```bash
rustfmt --edition 2024 --config skip_children=true \
  crates/rtd-json-rpc/src/read_api.rs \
  crates/rtd-json-rpc/src/coin_api.rs \
  crates/rtd-json-rpc/src/governance_api.rs \
  crates/rtd-core/src/authority.rs \
  crates/rtd-core/src/execution_cache/object_locks.rs \
  crates/rtd-core/src/authority/authority_store.rs

cargo build -p rtd
```

结果：通过。构建耗时约 2 分 50 秒。

注意：新增 `RTD_STATE_DIVERGENCE` 日志后，启动脚本的默认 `RUST_LOG` 也已同步扩展，否则只会看到 `RTD_TX_TRACE`，看不到 read/object/coin/system state/live lock 日志。

当前默认 `RUST_LOG` 包括：

```text
rtd_json_rpc::read_api=info
rtd_json_rpc::coin_api=info
rtd_json_rpc::governance_api=info
rtd_core::authority=info
rtd_core::execution_cache::object_locks=info
rtd_core::authority::authority_store=info
```

涉及脚本：

```text
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/toggle_local_rtd.sh
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/deploy_local_all.sh
```

已执行：

```bash
bash -n /Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/toggle_local_rtd.sh
bash -n /Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/deploy_local_all.sh
```

结果：通过。

## 2026-07-09 22:15 CST：debug 节点重启后转账复现

- 已使用 `target/debug/rtd start --fullnode-rpc-port 9000` 运行的本地节点自动执行 RPC 和转账验证。
- `rtdx_getCoins` 与 `rtd_getObject` 当前一致返回 gas coin version 100，digest `BMvtMF8vJtfsuGC8okSaw9wnqg7nYfsCFqvZWm9fRXKZ`，previous tx `9gicaew6zcrUMhNqA5LifnC11jXkmiaUoHHTDxHAJHpj`。
- `rtd_getLatestCheckpointSequenceNumber` 当前返回 `808`；重启日志显示 validator checkpoint store 初始化时 `executed=1209471 certified=1209471`，说明 RPC fullnode 仍在从较早 checkpoint 追赶。
- 已自动执行允许的测试转账：`target/debug/rtd client transfer-rtd --to 0xfbc95a1fbdd68117b26163e4068da6befc2fde9b02cb570b2351237bec447ba3 --rtd-coin-object-id 0x4a3ced14c55e220d6ffeefd39111ce80df6753c4ef40947aedebc8c26a8fa178 --amount 1000000 --gas-budget 5000000000 --json`。
- 转账稳定失败，validator 返回：RPC 使用 `Version 0x64`，但 validator live 是 `current version: 0x6a`。
- 当前证据表明：coin index 修复后 `getCoins` 和 `getObject` 已保持一致，但二者一致的是 fullnode 落后状态；本地 all-in-one RPC 在 fullnode 未追平 validator 时仍对外提供旧对象 refs，导致客户端签出必然被 validator 拒绝的交易。


## 2026-07-09 22:24 CST：fullnode DB 路径根因和修复设计

- 检查 `~/.rtd/rtd_config/full_node_db` 发现多个随机短 id 子目录，例如 `819bc1dbe416`、`8fdd20bc8ecf`、`b3ccc0c08bc6` 等；这些目录创建时间与多次 `rtd start` 重启时间对应。
- validator DB 路径稳定为 `~/.rtd/rtd_config/authorities_db/99f25ef61f80`，所以 validator 能保留最新 live object，而 fullnode RPC 每次使用新的随机 fullnode DB 从旧 checkpoint 追赶。
- 代码定位到 `crates/rtd-swarm-config/src/node_config_builder.rs::FullnodeConfigBuilder::build`：fullnode 默认使用随机生成的 key 派生 `key_path`，从而得到 `full_node_db/<random-key-prefix>`。
- 窄修复：给 `SwarmBuilder` 增加可选 `with_fullnode_db_path(PathBuf)`；`rtd start` 为第一个 RPC fullnode 显式传入稳定路径 `rtd_config_path/full_node_db/localnet-fullnode`。
- 该修改只影响 CLI `rtd start` 本地启动的第一个 fullnode；默认测试 swarm 未调用该新方法，仍保持原随机 fullnode DB 行为。


## 2026-07-09 22:27 CST：稳定 fullnode DB 首次验证

- 编译通过：`cargo build -p rtd`。
- 非破坏性停止当前节点后，将最近一次 fullnode DB `~/.rtd/rtd_config/full_node_db/819bc1dbe416` 复制为稳定路径 `~/.rtd/rtd_config/full_node_db/localnet-fullnode`，没有删除旧 fullnode DB，也没有改动 validator DB。
- 使用新 debug 二进制重启后，日志显示 validator checkpoint store `executed=1209471 certified=1209471`；fullnode checkpoint store 从 `executed=918 certified=919` 初始化，而不是之前的 `executed=0 certified=0`。
- RPC ready 后 `rtd_getLatestCheckpointSequenceNumber` 返回 `958`，说明稳定 DB 路径已生效，fullnode 从复制过来的最近进度继续追赶。


## 2026-07-09 22:35 CST：候选 fullnode DB 继续验证

- 当前仍运行 PID 77943：`target/debug/rtd start --fullnode-rpc-port 9000`。
- 稳定路径 `~/.rtd/rtd_config/full_node_db/localnet-fullnode` 当前是指向 `8c3702a4a524` 的 symlink；该候选启动较慢，但 RPC 已可用。
- 启动日志显示 validator checkpoint store 为 `executed=1209471 certified=1209471`，fullnode 从 `executed=722746 certified=1171713` 初始化。
- RPC 当前 checkpoint 从约 `722926` 推进到 `723369`；`rtdx_getCoins` 与 `rtd_getObject` 一致返回 gas coin version 105、digest `HH6vpAooefiJhizMRpaufLU4hzgGLz2GtEWgiUEV2rAL`。
- 自动转账仍失败：客户端使用 version `0x69`，validator live 为 `current version: 0x6a`。说明稳定 DB 修复了“每次从 0 追赶”的问题，但 fullnode 落后一版时仍会对外暴露旧 owned-object ref。


## 2026-07-09 22:43 CST：稳定 DB 路径回归测试

- 新增轻量单元测试 `memory::swarm::test::rpc_fullnode_uses_configured_db_path`，不启动节点，只 build swarm 并断言第一个 RPC fullnode 的 `NodeConfig.db_path` 等于显式传入路径。
- 已运行：`cargo test -p rtd-swarm rpc_fullnode_uses_configured_db_path`。
- 结果：通过，`1 passed; 0 failed`。

## 2026-07-09 22:48 CST：fullnode DB 候选扫描

- 采用非破坏方式扫描候选：只切换 `~/.rtd/rtd_config/full_node_db/localnet-fullnode` symlink，不删除任何 DB 目录。
- `971eb7b41e1e`：RPC ready，checkpoint `1101871`，gas coin version 105，digest `HH6vpAooefiJhizMRpaufLU4hzgGLz2GtEWgiUEV2rAL`。
- `a83c272c331b`：RPC ready，checkpoint `10442`，gas coin version 100，digest `BMvtMF8vJtfsuGC8okSaw9wnqg7nYfsCFqvZWm9fRXKZ`。
- `8c3702a4a524`：RPC ready，checkpoint `724581`，gas coin version 105，digest `HH6vpAooefiJhizMRpaufLU4hzgGLz2GtEWgiUEV2rAL`。
- 当前已将稳定 symlink 切到进度最高的 `971eb7b41e1e` 并重启；RPC ready 时 checkpoint 仍为 `1101871`。

## 2026-07-09 22:56 CST：validator DB 副本实验

- 非破坏性停止节点后，将 validator DB `~/.rtd/rtd_config/authorities_db/99f25ef61f80` clone 到新的 fullnode 候选目录 `localnet-fullnode-from-validator-202607092254`。
- 使用该副本启动 fullnode 后，`rtd_getLatestCheckpointSequenceNumber` 立即返回 `1209471`，`rtd_getObject` 返回 gas coin version 106、digest `3dntckSuZt17DYCn5CRdKwfc18iyJxm5ATZC4JbiyCVX`。
- 但 `rtdx_getCoins` 返回空数组，因为 validator DB 没有 fullnode 的 `indexes` / `rpc-index` 数据；显式 coin 转账请求随后超时，日志显示节点在大量补 settlement/index 相关任务。
- 结论：validator DB 副本可以证明 validator object store 是最新的，但不能直接作为完整 RPC fullnode DB 使用；已切回 `971eb7b41e1e`，当前 `rtdx_getCoins` 正常返回 version 105。

## 2026-07-09 23:03 CST：本地启动脚本 readiness 防护

- 修改 `/Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/toggle_local_rtd.sh`。
- 新增默认开启的 `WAIT_FULLNODE_CATCHUP_TO_VALIDATOR=1`：RPC 基本可连后，脚本从本次启动日志里提取 validator 启动时的最高 checkpoint，并要求 fullnode RPC 的 `rtd_getLatestCheckpointSequenceNumber` 追到该高度才报告 ready。
- 新增 `WAIT_FULLNODE_CATCHUP_SECONDS`，默认沿用 `WAIT_RPC_SECONDS`；如需排查时快速启动但允许 fullnode 落后，可显式设置 `WAIT_FULLNODE_CATCHUP_TO_VALIDATOR=0`。
- 已执行 `bash -n toggle_local_rtd.sh`，结果通过。
- 注意：这不是链层根因修复，只是避免脚本在 fullnode 明显落后时误报 ready，从而减少客户端继续签出旧 object ref 的概率。

## 2026-07-09 23:12 CST：定位 version 106 的来源交易

- 临时切到 validator DB 副本，只查询 `9fWt8oeVio9KDd9zeGA1oKpNjUcfmYAxaVRBZiRWjL1M`，随后已切回 `971eb7b41e1e`。
- 该交易在 validator DB 中存在且成功：输入 gas payment 是 gas coin version 105、digest `HH6vpAooefiJhizMRpaufLU4hzgGLz2GtEWgiUEV2rAL`；effects 将同一 gas coin mutate 到 version 106、digest `3dntckSuZt17DYCn5CRdKwfc18iyJxm5ATZC4JbiyCVX`。
- 因此当前 fullnode 返回 version 105 的直接原因是它还没有同步/执行到交易 `9fWt8...`；validator 拒绝后续 version 105 交易是正确行为。
- 切回后，`localnet-fullnode -> 971eb7b41e1e`，RPC checkpoint 为 `1103926`。

## 2026-07-09 23:15 CST：构建与定向测试

- 已运行 `rustfmt --edition 2024 --config skip_children=true crates/rtd-swarm/src/memory/swarm.rs crates/rtd/src/rtd_commands.rs`。
- 已运行 `cargo build -p rtd`，结果通过。
- 已再次运行 `cargo test -p rtd-swarm rpc_fullnode_uses_configured_db_path`，结果通过：`1 passed; 0 failed`。

## 2026-07-09 23:31 CST：最终验证

- 为通过 clippy，补充修正：
  - `crates/rtd-core/src/authority.rs`：按 clippy 建议用 `?` 简化 `indexes` 空值检查。
  - `crates/rtd-json-rpc/src/coin_api.rs`：将诊断用 `authority_state` 改为可选，测试 mock 不再需要真实 `AuthorityState`。
- 已运行 `cargo xclippy`，结果通过：`Finished dev profile`。
- 已重新运行 `cargo build -p rtd`，结果通过：`Finished dev profile`。
- 已重新运行 `cargo test -p rtd-swarm rpc_fullnode_uses_configured_db_path`，结果通过：`1 passed; 0 failed`。

## 2026-07-09 23:35 CST：最终运行态

- 已使用新编译的 `target/debug/rtd` 通过 `toggle_local_rtd.sh` 重启本地节点；排查用启动显式设置了 `WAIT_FULLNODE_CATCHUP_TO_VALIDATOR=0`，脚本默认仍会执行 catch-up readiness 防护。
- 当前进程：`target/debug/rtd start --fullnode-rpc-port 9000`。
- 当前稳定 fullnode DB：`localnet-fullnode -> ~/.rtd/rtd_config/full_node_db/971eb7b41e1e`。
- 当前 RPC checkpoint：`1107355`，仍未追到 validator 启动高度 `1209471`，因此 gas coin 仍预计返回 version 105，暂不应继续用该 RPC 发需要最新 gas ref 的转账。
