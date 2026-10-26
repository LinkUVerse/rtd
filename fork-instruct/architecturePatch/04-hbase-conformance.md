# 专题 D：HBase 兼容层的一致性门禁

## 1. 范围

RTD 用 HBase 3.0 替代托管宽表服务，通过 `hbase-compat-gateway` 把官方客户端实际使用的宽表数据面请求翻译为 HBase Java API。该替换属于允许的中间件替换，但必须保证：

- 官方客户端当前使用的 RPC、过滤器、行键和列族语义不变；
- 官方客户端升级后，若不兼容，必须显式失败，而不是静默读错/写错。

## 2. 当前实现

- 网关只实现：`PingAndWarm`、`ReadRows`、`MutateRows`、`CheckAndMutateRow`。
- 过滤器只支持当前客户端实际使用的子集：`Chain`、`CellsPerColumnLimitFilter(1)`、`FamilyNameRegexFilter("rtd")`、精确 qualifier 正则、CAS 的 `ValueRangeFilter`。
- 14 张表名与官方一致；`deploy/local/hbase/init-tables.rb` 创建列族 `rtd`、`VERSIONS => 1`。
- 预分区、压缩、Bloom、Region 规划尚未与目标负载对齐（属于性能项，不是 API 正确性项）。
- HBase WAL 独立于对象目录；多节点 WAL 恢复、fencing、ZooKeeper 生产 HA 仍属于中间件生产验收，不在本专题的 API 兼容门禁内。

## 3. 必须补齐

### 3.1 版本门禁

建立机读的 conformance 锁，至少固定：

- 官方 `kvstore` 宽表客户端源码摘要（含 Bigtable 请求构造）；
- 官方 `kv-rpc` 数据面客户端摘要；
- 网关实现的 RPC 清单和 filter 清单；
- 14 张表名与列族配置摘要。

客户端源码摘要变化时，`verify-conformance.sh` 失败并提示复核；不允许“升级官方二进制后自动通过”。

### 3.2 回归矩阵

每次升级必须跑：

1. 14 条管线从 checkpoint 0 到有界高度的回填；
2. `GetObject`/`BatchGetObjects`/`GetTransaction`/`BatchGetTransactions`/`GetCheckpoint`/`GetEpoch`/包服务点查；
3. `ListCheckpoints`/`ListTransactions`/`ListEvents` 正反向、空区间、游标终止、组合过滤；
4. `ReadRows` 正反向、闭开边界、大值分块、列过滤、rows_limit；
5. `MutateRows` 部分失败逐项索引、原子行、`timestamp_micros=-1` 与显式历史时间；
6. `CheckAndMutateRow` 链 ID 绑定、水位单调推进、predicate false 写入；
7. 未知 RPC/过滤器/表/列族失败关闭；
8. HBase 连接失败、超时、取消与恢复。

### 3.3 表结构

- 继续由显式 DDL 创建 14 表，列族 `rtd`、`VERSIONS => 1`；
- 不设置 TTL；
- 预分区/压缩/Bloom/Region 只做性能实验，不改表名、列族、限定符、时间戳和行键编码；
- 任何表结构变更必须重新跑全矩阵。

## 4. 验收

1. conformance 锁校验通过。
2. 14 管线有界回填和 13 个查询方法样本通过（`GetEpoch` 同时覆盖公共入口）。
3. 未知 filter/RPC 返回明确失败，不返回空成功。
4. 表 DDL、网关 RPC 清单、客户端源码摘要三者一致。
5. 工作日志记录固定源码摘要、测试命令和未覆盖的多节点 WAL 项。

## 5. 非目标

- 不实现完整宽表 API；
- 不改官方客户端的业务查询语义；
- 不以关闭 WAL/放宽能力检查来换取“兼容”。
