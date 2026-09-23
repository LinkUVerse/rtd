# dev 环境重启后钱包可继续转账的调整方向调研

## 背景

已有结论见同目录 `README.md`：当前错误的直接含义是交易引用了旧版本 owned object/gas coin，例如交易消费 `version 0x64`，但链上 live object 已是 `0x69`。

本次继续调研两个代码库：

```text
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/rtd
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/rtd-apps/wallet
```

目标是：本地 dev 链无论停机多久，重新拉起 `rtd` 后，钱包仍能成功转账。

## 总结结论

需要调整，而且优先应先改钱包侧。

钱包当前会把 coin object refs 作为 React Query 数据持久化到 IndexedDB，默认缓存保留 24 小时。转账页会直接使用这些 coin refs 构造交易。对本地 dev 链来说，这个缓存策略不安全：进程停机、重启、pending transaction 恢复执行、或者 localnet 重新 genesis 后，缓存中的 `{ objectId, version, digest }` 很容易变成旧值。

如果实时 RPC 的 `rtdx_getCoins/rtdx_getAllCoins` 已经返回最新版本，那么仅修钱包即可满足需求。

如果清掉钱包缓存或强制 refetch 后仍报同样错误，则说明 RTD 节点的 coin index 长期返回旧版本，钱包无法单独保证成功，RTD 侧也需要修复 `getCoins` 返回值与 live object store 的一致性。

建议优先级：

1. 钱包端：coin refs 类查询不要持久化，不要在发送时复用缓存 refs。
2. 钱包端：本地链 chain identifier 变化时清空 query cache。
3. 钱包端：遇到 `ObjectVersionUnavailableForConsumption` 后，清 coin/object 查询并用最新 refs 重建交易，最多自动重试一次。
4. RTD 端：确保 `rtdx_getCoins/rtdx_getAllCoins` 返回 live object refs，或在索引未追上时拒绝/延迟服务。

## 钱包源码证据

### 1. React Query 缓存默认持久化 24 小时

文件：

```text
rtd-apps/wallet/src/ui/app/helpers/queryClient.ts
rtd-apps/wallet/src/ui/index.tsx
```

全局 QueryClient 配置：

```ts
staleTime: 30 * 1000
gcTime: 24 * 60 * 60 * 1000
```

并且使用 `PersistQueryClientProvider` + IndexedDB persister：

```ts
export const persister = createIDBPersister('queryClient.v1');
```

持久化过滤条件只有：

```ts
shouldDehydrateQuery: ({ meta }) => !meta?.skipPersistedCache
```

也就是说，除非单个 query 显式设置 `meta.skipPersistedCache = true`，否则会被持久化。

### 2. `useGetAllCoins` 会缓存完整 coin refs

文件：

```text
rtd-apps/wallet/src/ui/app/hooks/useGetAllCoins.ts
```

该 hook 的 query key 是：

```ts
['get-all-coins', address, coinType]
```

它循环调用：

```ts
rpc.getCoins({ owner, coinType, cursor, limit })
```

返回 `CoinStruct[]`，其中包含：

```text
coinObjectId
version
digest
balance
```

这个 query 没有设置：

- `meta: { skipPersistedCache: true }`
- 更短的 `gcTime`
- 本地链 chain id 作为 query key
- 发送前强制 refetch

因此它会继承全局 24 小时持久化缓存。

### 3. 转账页直接使用缓存 coin refs 构造交易

文件：

```text
rtd-apps/wallet/src/ui/app/pages/home/transfer-coin/SendTokenForm.tsx
rtd-apps/wallet/src/ui/app/pages/home/transfer-coin/utils/transaction.ts
```

`SendTokenForm` 读取 coin 列表：

```ts
const { data: coinsData } = useGetAllCoins(coinType, activeAddress!);
const { data: rtdCoinsData } = useGetAllCoins(RTD_TYPE_ARG, activeAddress!);
```

提交时把 `coins` 传给交易构造函数：

```ts
const data = {
  to,
  amount,
  isPayAllRtd,
  coins,
  coinIds: coinsIDs,
  gasBudgetEst,
};
```

`createTokenTransferTransaction` 在 PayAll RTD 场景中显式把缓存 refs 写入 gas payment：

```ts
tx.setGasPayment(
  coins
    .filter((coin) => coin.coinType === coinType)
    .map((coin) => ({
      objectId: coin.coinObjectId,
      digest: coin.digest,
      version: coin.version,
    })),
);
```

这正好会把旧 `{ objectId, version, digest }` 编进交易。

普通 RTD 转账主要依赖 SDK build 阶段自动选择 gas；非 RTD 转账会用缓存 coin object id，再由 SDK build 阶段解析 object ref。即使这些路径比 PayAll RTD 更安全，gas 选择仍依赖 RPC `getCoins`，所以如果节点 coin index 自身 stale，也会继续失败。

### 4. 成功后 invalidate 的 key 不覆盖 `useGetAllCoins`

文件：

```text
rtd-apps/wallet/src/ui/app/pages/home/transfer-coin/index.tsx
```

成功后执行：

```ts
queryClient.invalidateQueries({ queryKey: ['get-coins'] });
queryClient.invalidateQueries({ queryKey: ['coin-balance'] });
```

但转账表单使用的是：

```ts
['get-all-coins', address, coinType]
```

所以即使之前有成功交易，`get-all-coins` 也不会被这两条 invalidate 精确覆盖。失败时也没有清理 coin refs 查询。

### 5. localnet 重新 genesis 时 query key 无法区分新旧链

钱包按网络切换会 remount React tree：

```text
rtd-apps/wallet/src/ui/index.tsx
```

key 是：

```ts
`${apiEnv}_${customRPC}`
```

但 localnet/dev 自建链常见情况是：RPC URL 不变，chain identifier 变了。当前 coin query key 只包含 `address` 和 `coinType`，没有 chain identifier。这样 `http://127.0.0.1:9000` 上的新链可能继续读旧链的 IndexedDB query cache。

这对 `rtd start --force-regenesis` 尤其危险：每次都是新链，但钱包缓存 namespace 仍可能相同。

## SDK 源码证据

文件：

```text
rtd-ts-sdk/packages/typescript/src/jsonRpc/json-rpc-resolver.ts
rtd-ts-sdk/packages/typescript/src/transactions/Transaction.ts
```

SDK build 阶段会解析未解析对象：

```ts
resolveObjectReferences(...)
client.multiGetObjects(...)
```

如果交易只传 `tx.object(objectId)`，SDK 会用 `multiGetObjects` 查 live object ref。

但如果交易已经显式设置 gas payment：

```ts
setGasPayment(payments: ObjectRef[])
```

SDK 会直接使用调用方给出的 `{ objectId, version, digest }`。

另外，默认 gas 选择逻辑是：

```ts
client.getCoins({ owner, coinType: RTD_TYPE_ARG })
```

然后把 `coin.version` 和 `coin.digest` 写入 gas payment。也就是说，SDK 默认信任 `rtdx_getCoins` 的 refs。如果节点 `getCoins` 返回旧版本，SDK 会构造旧版本 gas payment，钱包侧无法通过普通 build 自动修正。

## RTD 节点侧判断

RTD 节点当前有两套读路径：

- `getObject/multiGetObjects` 读取 live object/object store。
- `getCoins/getAllCoins` 读取 JSON-RPC coin index。

前一次调研已确认：

```text
crates/rtd-json-rpc/src/coin_api.rs
crates/rtd-json-rpc/src/authority_state.rs
crates/rtd-core/src/jsonrpc_index.rs
```

`getCoins` 走 `coin_index_2`，返回 `CoinInfo.version/digest`。

交易校验走 live object：

```text
crates/rtd-core/src/execution_cache/object_locks.rs
crates/rtd-core/src/authority.rs
```

因此如果 `coin_index_2` 和 live object store 不一致，就会出现：

```text
getCoins 返回 version 0x64
live object 是 version 0x69
交易提交被拒绝
```

等待十几分钟仍失败有两种可能：

1. 钱包仍在使用 IndexedDB 的旧 query cache，没有真正请求最新 `getCoins`。
2. 钱包已经实时请求 RPC，但 RTD 节点的 coin index 长期 stale。

这两种需要用现场数据区分。

## 为实现需求应做的调整

### 钱包侧必须做的调整

#### 1. coin refs 查询不要持久化

对以下查询设置 `meta.skipPersistedCache = true`：

- `['get-all-coins', address, coinType]`
- `['get-coins', address, coinType, maxCoinsPerRequest]`
- 其他直接保存 `{ objectId, version, digest }` 的 object/coin 查询

理由：object refs 是交易输入，不是普通展示数据。它们具有强时效性，不能按 24 小时持久化缓存处理。

#### 2. 转账发送前强制 refetch coins

在点击 `Send Now` 或进入 Review 前，不应信任表单打开时的 `coins`。

建议发送前重新请求：

```text
rtdx_getCoins / rtdx_getAllCoins
```

并用最新结果重建 transaction。

关键点：不能重试同一份已 build/sign 的交易 bytes，必须重新构造 `Transaction`，重新 build，重新签名。

#### 3. 本地链 chain id 变化时清 query cache

钱包可以通过 SDK：

```ts
client.getChainIdentifier()
```

该方法读取 genesis checkpoint digest。建议把 chain identifier 纳入：

- query key
- persisted cache namespace
- 或启动时的 cache invalidation 判断

当当前 RPC URL 不变但 chain identifier 变化时，应清除 coin/object/transaction 查询缓存。这样 `rtd start --force-regenesis` 后不会复用旧链 refs。

#### 4. 修正转账成功后的 invalidate key

当前只 invalidate：

```ts
['get-coins']
['coin-balance']
```

还应覆盖：

```ts
['get-all-coins']
['getAllBalances']
```

尤其 `SendTokenForm` 依赖 `get-all-coins`，否则成功交易后同一个页面/后续页面仍可能保留旧 refs。

#### 5. 对 `ObjectVersionUnavailableForConsumption` 做恢复策略

当执行交易失败且错误包含：

```text
ObjectVersionUnavailableForConsumption
is not available for consumption, current version
```

钱包可以：

1. remove/invalidate coin 和 object queries。
2. 重新获取 coins。
3. 重新构造交易。
4. 自动重试一次。

只应重试一次，避免真实并发消费 coin 时循环失败。

#### 6. PayAll RTD 特别处理

PayAll RTD 当前必须显式 `setGasPayment`，所以它最容易固化旧 refs。

应保证 PayAll 使用的 refs 来自发送前即时查询，并可选地对每个 coin 再调用 `multiGetObjects` 校验：

- object still exists
- owner still is active address
- version/digest 与 `getCoins` 一致

如果不一致，应以 `multiGetObjects` 的 live ref 为准，或丢弃该 coin 后重新选择。

### RTD 节点侧可能需要做的调整

如果确认钱包已禁用缓存并强制 refetch，但 `getCoins` 仍返回旧版本，则需要修 RTD。

#### 1. 让 `getCoins` 返回前与 live object store 对齐

在本地 dev/fullnode 模式下，`rtdx_getCoins` 可以在返回前对 coin index 结果做 live object 校验：

- 对每个 coin id 调用 live object store。
- 如果 live object 不存在、已删除、owner 不匹配、不是 coin，则过滤掉。
- 如果 live object version/digest/balance 更新，则返回 live object 的 version/digest/balance。

这样即使 coin index stale，也不会把旧 refs 暴露给钱包。

代价是 `getCoins` 更慢，但本地 dev 环境可接受。生产环境可由配置开关控制。

#### 2. 启动阶段增加索引 ready gate

节点启动后，如果 checkpoint executor/RPC index 仍在追赶，应该避免给钱包返回 stale coin refs。

可选策略：

- `getCoins` 在 index 未追上 checkpoint 时返回明确错误。
- fullnode RPC readiness endpoint 暴露 index watermark。
- `rtd start` 本地模式等索引追上后再显示可用。

#### 3. 启动后验证或重建 coin index

对 dev/localnet，可以提供低成本修复：

- 启动时扫描 live objects 重建 `coin_index_2`。
- 或提供 `rtd repair-index --coin-index`。
- 或在发现 secondary index inconsistent 时自动重建。

这能解决异常 kill 进程导致索引落后或部分写入的问题。

#### 4. SDK 默认 gas 选择可增加 live 校验

SDK 的 `setGasPayment` 默认从 `client.getCoins` 取 refs。如果 RTD 节点短期无法保证 `getCoins` live，一种客户端兜底是 SDK 在选择 gas 后再 `multiGetObjects` 校验并替换 refs。

这属于 SDK 层防御，不替代节点修复。

## 建议现场确认

为了判断该改钱包还是改 RTD，建议在失败后立即做三组查询。

### 1. 确认钱包是否使用持久化缓存

在浏览器 DevTools 检查 IndexedDB：

```text
queryClient.v1
```

查找 query key：

```text
get-all-coins
get-coins
```

看其中是否存在错误 object id，以及版本是否是 `0x64`。

如果有，先清除 IndexedDB 的 `queryClient.v1` 或在钱包 More Options 里 logout/清缓存，再试转账。

### 2. 对比 RPC coin 与 live object

调用当前钱包连接的 RPC：

```text
rtdx_getCoins(owner, coinType)
rtd_getObject(objectId)
```

判断：

- `getCoins` 返回 `0x64`，`getObject` 返回 `0x69`：RTD coin index stale。
- `getCoins` 返回 `0x69`，钱包仍提交 `0x64`：钱包缓存 stale。

### 3. 清钱包缓存后再试

如果清缓存后立即恢复，基本可定性为钱包缓存策略问题。

如果清缓存后仍失败，重点查 RTD：

- `coin_index_2` 是否长期 stale
- checkpoint executor 是否卡住
- post process index tx 是否失败
- 启动恢复时是否有 pending tx 推进 coin version

## 推荐落地方案

为了满足“dev 环境无论停机多久，重新拉起后钱包还能转账”，建议组合方案：

1. 钱包立即修：coin refs 查询不持久化，发送前强制 refetch，错误后清 cache 并重建交易重试一次。
2. 钱包增强：query cache namespace 加 chain identifier，localnet chain id 变化自动清缓存。
3. RTD 排查：验证 `getCoins` 与 `getObject` 是否一致。
4. RTD 兜底：本地 dev 模式下让 `getCoins` 返回 live object 校验后的 refs，或启动时重建/校验 coin index。

最小满足需求的改动大概率在钱包端；但长期正确性应由 RTD 节点保证 `getCoins` 不返回不可消费的 object refs。
