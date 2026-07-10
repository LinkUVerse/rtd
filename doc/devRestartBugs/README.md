# RTD 本地 dev 环境停机后重启转账失败调研

## 背景

现象：

本地 dev 环境启动 RTD 链后，杀掉 `rtd` 进程。大约 15 小时后再次启动，节点可以正常启动，但钱包转账失败：

```text
Transaction is rejected as invalid by more than 1/3 of validators by stake (non-retriable).
Non-retriable errors: [Object ID 0x4a3ced14c55e220d6ffeefd39111ce80df6753c4ef40947aedebc8c26a8fa178
Version 0x64 Digest BMvtMF8vJtfsuGC8okSaw9wnqg7nYfsCFqvZWm9fRXKZ is not available for consumption,
current version: 0x69 { k#99f25ef6.. } with 10000 stake].
```

本项目 RTD fork 自 Sui。调研时也对比了本地 Sui 原始源码：

```text
/Users/changzechuan/WenchuanProjects/SuiTestProjects/Sui-Origin/sui
```

## 结论

这个报错本身不是“链停机几个小时后协议不允许转账”的设计，也不是 epoch 过期导致账户失效。

直接原因是：钱包提交的交易里引用了一个 owned object/gas coin 的旧版本。链上 live object 已经是 `0x69`，但交易仍在消费 `0x64`，验证器按 Sui/RTD 的对象模型必须拒绝这种交易。拒绝旧版本对象是正常设计。

更准确地说：

- `Version 0x64 ... current version: 0x69` 说明同一个 object 在链上已经被后续交易更新过。
- 转账交易中的 gas coin 或输入 coin 仍引用旧 object ref。
- 验证器不会把“历史版本对象”当成可消费对象，因为 owned object 必须精确匹配当前 live version 和 digest。
- `non-retriable` 表示继续重试同一笔交易没有意义，必须重新查询最新 object ref 并重新构造/签名交易。

停机 15 小时是一个强相关触发条件，但从源码看不是根因本身。它更可能让本地 dev 环境出现下面两类问题：

1. 钱包端/SDK 端缓存了停机前的 coin object ref，重启后仍用旧版本构造交易。
2. 本地 fullnode 的 JSON-RPC coin index 与 live object store 在重启恢复、checkpoint 补执行或索引追赶期间短暂不一致，`getCoins/getAllCoins` 返回旧版本 coin，随后交易校验按 live object 拒绝。

因此，这个问题需要拆成两层判断：

- 协议层拒绝旧 object version：正常设计。
- 本地 dev 重启后 RPC/钱包为什么拿到旧 object ref：需要进一步现场日志和状态对比。若确认 `getCoins` 在节点已同步完成后仍长期返回旧版本，则是本地索引/恢复 bug；若只是钱包缓存或索引追赶窗口，则是使用方式/本地环境一致性问题。

## 源码证据

### 错误文案来源

错误枚举定义在：

```text
crates/rtd-types/src/error.rs
```

`UserInputError::ObjectVersionUnavailableForConsumption` 的格式就是：

```text
Object ID ... Version ... Digest ... is not available for consumption, current version: ...
```

对应含义是：交易提供了某个 object ref，但当前可消费 live object 版本不是它。

交易驱动把超过 1/3 stake 的非重试错误格式化成用户看到的：

```text
crates/rtd-core/src/transaction_driver/error.rs
```

关键文案：

```text
Transaction is rejected as invalid by more than 1/3 of validators by stake (non-retriable).
Non-retriable errors: [...]
```

这说明该错误来自验证器投票/提交阶段，而不是钱包本地 UI 自己生成的错误。

### live object 版本校验

核心校验在：

```text
crates/rtd-core/src/execution_cache/object_locks.rs
```

`verify_live_object` 要求 owned object 的版本必须等于 live object 版本：

```rust
if obj_ref.1 != live_object.version() {
    return Err(RtdErrorKind::UserInputError {
        error: UserInputError::ObjectVersionUnavailableForConsumption {
            provided_obj_ref: *obj_ref,
            current_version: live_object.version(),
        },
    }.into());
}
```

这就是为什么 `0x64` 无法在当前版本 `0x69` 时继续消费。

### fullnode 提前校验也会拒绝旧版本

RTD/Sui 在 fullnode 交易编排器里还有提前校验：

```text
crates/rtd-core/src/authority.rs
crates/rtd-core/src/transaction_orchestrator.rs
crates/rtd-config/src/node.rs
```

`check_transaction_validity` 会读取 live object：

```rust
if obj_ref.1 < live_object.version() {
    return Err(RtdErrorKind::UserInputError {
        error: UserInputError::ObjectVersionUnavailableForConsumption {
            provided_obj_ref: *obj_ref,
            current_version: live_object.version(),
        },
    }.into());
}
```

`TransactionDriverConfig.enable_early_validation` 默认是 `true`，所以 fullnode 会尽早把旧版本输入拒绝掉，避免继续提交明显无效的交易。

### 与 Sui 原始源码对比

对比 Sui 原始源码：

```text
/Users/changzechuan/WenchuanProjects/SuiTestProjects/Sui-Origin/sui/crates/sui-core/src/authority.rs
/Users/changzechuan/WenchuanProjects/SuiTestProjects/Sui-Origin/sui/crates/sui-core/src/execution_cache/object_locks.rs
```

上述校验逻辑在 Sui 中同样存在。也就是说，拒绝旧版本 owned object 不是 RTD 品牌替换引入的独有行为，而是 Sui 对象模型的正常规则。

## 为什么停机重启后更容易触发

### epoch 时长不是唯一解释

`rtd genesis` 持久化 localnet 默认 epoch duration 是 24 小时：

```text
crates/rtd-config/src/genesis.rs
```

`GenesisCeremonyParameters::default_epoch_duration_ms()` 返回：

```rust
24 * 60 * 60 * 1000
```

而 `rtd start --force-regenesis` 的临时链默认 epoch duration 是 60 秒：

```text
crates/rtd/src/rtd_commands.rs
```

```rust
const DEFAULT_EPOCH_DURATION_MS: u64 = 60_000;
```

所以“停机 15 小时后必然因为 epoch 跨越导致无法转账”这个判断不成立：

- 如果用持久化 genesis 默认配置，15 小时小于 24 小时，未必跨 epoch。
- 如果用 `--force-regenesis` 或显式设置短 epoch，15 小时会跨很多 epoch，但同一条错误仍然是在说 owned object 版本旧了，不是在说 epoch 不可用。

### coin API 依赖二级索引

钱包通常会通过 JSON-RPC 的 `getCoins/getAllCoins` 查询可用 coin。RTD 的 coin API 走二级索引：

```text
crates/rtd-json-rpc/src/coin_api.rs
crates/rtd-json-rpc/src/authority_state.rs
crates/rtd-core/src/jsonrpc_index.rs
```

调用链大致是：

```text
getCoins/getAllCoins
-> StateRead::get_owned_coins
-> AuthorityState::get_owned_coins_iterator_with_cursor
-> IndexStore::coin_index_2
```

索引项 `CoinInfo` 保存了 coin 的：

- object id
- version
- digest
- balance
- previous_transaction

交易校验则读取 live object store。这意味着如果 coin index 滞后于 live object store，RPC 可能给钱包旧版本 coin，而交易提交时又被 live object 校验拒绝。

### coin index 的更新依赖交易执行/检查点索引流程

二级索引更新在交易执行后的 post process 和 checkpoint RPC index 中完成：

```text
crates/rtd-core/src/authority.rs
crates/rtd-core/src/jsonrpc_index.rs
crates/rtd-core/src/checkpoints/checkpoint_executor/mod.rs
crates/rtd-core/src/rpc_index.rs
```

`index_coin` 会删除 input coin 的旧索引 key，再插入 written coin 的新索引 key。如果节点异常停止、重启后补执行 checkpoint、索引写入还没追上，或者 fullnode 索引状态与 object store 状态不一致，就可能短时间返回旧 coin version。

源码中也能看到 RPC checkpoint index 是先 stage pending update，再按 checkpoint 顺序 commit：

```text
RpcIndex::index_checkpoint
RpcIndex::commit_update_for_checkpoint
```

这类设计允许恢复时追赶，但也意味着本地启动“进程可服务 RPC”不等价于“所有索引已经追上 live object 状态”。

## 是否是 bug

分场景判断：

### 不是 bug 的部分

下面行为是正常设计：

- 使用旧 object version 构造交易会被拒绝。
- 错误是 non-retriable。
- 必须重新查询最新 object ref，并重新构造/签名交易。
- 停机后重启不会让历史 object version 重新变成可消费版本。

### 可能是 bug 的部分

如果满足以下条件，则应按 bug 继续排查：

1. 节点已经完成 checkpoint/execution/index 追赶。
2. 钱包没有使用本地缓存，而是每次都实时调用当前 fullnode RPC。
3. `getObject(coin_id)` 返回 version `0x69`，但 `getCoins/getAllCoins` 仍返回同一个 coin 的 version `0x64`。
4. 这种不一致长期存在，而不是启动后短暂追赶窗口。

这种情况说明 JSON-RPC coin index 与 live object store 不一致，属于 fullnode 二级索引恢复/更新问题。

### 更可能的实际原因

从你给出的错误看，最可能的是钱包或 SDK 使用了过期 coin ref：

```text
provided version: 0x64
current version: 0x69
```

版本差了 5，说明这个 coin 在之前已经被执行过多次变更。常见来源：

- 浏览器钱包缓存了账户 coins，重启链后没有刷新。
- 钱包在网络短暂不可用期间保留了旧 coin list。
- 本地 fullnode 刚启动时 coin index 还没追上，钱包提前读取并缓存了旧结果。
- 之前有后台重试/恢复的 pending transaction 在节点重启后继续执行，把 coin 推进到新版本，而钱包还持有旧版本。

## 建议现场排查步骤

以下命令不需要改代码，目的是确认旧版本来自钱包缓存还是 RPC 索引。

### 1. 查询当前 live object 版本

对错误里的 object id 查询：

```bash
rtd client object 0x4a3ced14c55e220d6ffeefd39111ce80df6753c4ef40947aedebc8c26a8fa178 --json
```

或用 JSON-RPC `rtd_getObject`。

重点看：

- `version`
- `digest`
- `owner`
- `previousTransaction`

如果这里是 `0x69`，说明链上 live object 确实已推进。

### 2. 查询 coin API 返回版本

```bash
rtd client gas --json
```

或调用 JSON-RPC：

```text
rtdx_getCoins / rtdx_getAllCoins
```

重点看同一个 `coinObjectId` 的：

- `version`
- `digest`
- `previousTransaction`

如果 coin API 仍返回 `0x64`，而 object API 返回 `0x69`，就是 RPC coin index 与 live object store 不一致。

如果 coin API 已返回 `0x69`，但钱包交易仍提交 `0x64`，就是钱包/SDK 缓存了旧 object ref。

### 3. 等待索引追赶后重复查询

重启节点后不要立刻转账，先等待 checkpoint executor/indexer 追赶完成，再重复步骤 1 和 2。

如果等待后恢复一致，说明这是本地 dev 环境启动恢复窗口问题。

如果一直不一致，则应继续查：

- `secondary indexes are inconsistent`
- `Post processing - Couldn't index tx`
- `commit_update_for_checkpoint`
- checkpoint executor 是否卡住

### 4. 检查是否有 pending transaction 恢复执行

交易编排器会维护：

```text
fullnode_pending_transactions
```

源码位置：

```text
crates/rtd-core/src/transaction_orchestrator.rs
```

重启后 pending tx 可能恢复提交或查询结果。若之前钱包提交过交易但 UI 没拿到结果，重启后链上 coin 版本可能已经推进，而钱包仍认为交易未发生。

## 对使用方的规避建议

本地 dev 环境重启后：

1. 钱包端清理/刷新 coin 缓存。
2. 转账前重新调用 `getCoins/getAllCoins`，不要复用停机前保存的 object ref。
3. 如果使用浏览器钱包，断开并重新连接 localnet，必要时清除该网络缓存。
4. 如果是脚本/SDK，遇到 `ObjectVersionUnavailableForConsumption` 后不要重试同一笔签名交易，应重新选择 coin、重新构造并签名。
5. 等待 fullnode checkpoint 和索引追赶完成后再发第一笔交易。
6. 若使用临时 dev 链，优先明确区分：
   - `rtd start --force-regenesis`：每次新链，不应复用旧钱包状态。
   - 持久化 `rtd genesis` + `rtd start`：状态保留，但仍需刷新最新 coin refs。

## 后续可验证假设

为了最终定性是否存在 RTD 本地索引 bug，建议在复现现场记录一个四列表：

| 时间点 | API | coin id | version/digest |
| --- | --- | --- | --- |
| 重启刚完成 | `getObject` | 错误中的 object id | live version |
| 重启刚完成 | `getCoins/getAllCoins` | 同一 object id | coin index version |
| 等待 N 分钟后 | `getObject` | 同一 object id | live version |
| 等待 N 分钟后 | `getCoins/getAllCoins` | 同一 object id | coin index version |

判断标准：

- `getObject == getCoins`，但钱包仍提交旧版本：钱包缓存问题。
- `getObject != getCoins`，短时间后恢复：索引追赶窗口。
- `getObject != getCoins`，长期不恢复：二级索引恢复/更新 bug。
