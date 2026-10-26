# 专题 F：非中间件性能基线

## 1. 口径

性能对标分两层：

- **中间件替换开销**：HBase 网关翻译、S3A、对象存储 LIST/COPY、网络；单独度量，不与官方核心直接比较。
- **RTD 业务层开销**：publication 认证/复制/读回、account API 扫描/索引、RTD 额外管线；必须与官方核心性能一起评估。

不能用“中间件不同”解释 publication 或 account 层的额外延迟。

## 2. 必须建立的指标

| 层 | 指标 | 采集方式 |
| --- | --- | --- |
| checkpoint 发布 | 每轮认证耗时、copy/readback 耗时、发布延迟、committed 高度 | publisher 日志/回执、metrics |
| indexer | checkpoint ingest 速率、各管线 lag、commit 延迟 | indexer metrics、PG watermarks |
| KV store | 14 管线写入速率、HBase 网关延迟/错误 | kvstore metrics、网关指标 |
| KV RPC | 点查/批量/List p50/p95/p99、错误率 | gRPC 基准脚本 |
| GraphQL | 地址/交易/事件/对象/检查点查询 p50/p95/p99、错误率 | HTTP 基准脚本 |
| Consistent | 余额/持有对象读取延迟、副本追平时间 | consistent metrics |
| account | 账户历史首页/续页延迟、扫描预算、失败率 | account metrics |

## 3. 基准方法

- 固定链高度样本：创世、早期、修剪边界两侧、当前头。
- 固定查询集：地址余额/交易、交易详情、事件过滤、对象/对象版本、检查点列表、包、serviceConfig。
- 记录结果大小、分页参数、并发度、后端配置。
- publication 基准必须区分“认证/复制”和“中间件网络”耗时。
- 不把 dev 单机低并发配置当作生产容量结论；生产配置需单独跑。

## 4. 验收

1. 有可重复执行的基准脚本和固定样本。
2. 每类查询有 p50/p95/p99、错误率、返回大小、并发度记录。
3. publication 额外延迟可分解，且不随归档规模线性增长。
4. account 首页/续页有预算和超限拒绝路径。
5. 与官方基线的差异按“中间件/RTD 业务层”分类说明；不能只给总时延。
6. 性能回归可作为发布门禁，至少覆盖 publication lag、GraphQL p99、KV RPC p99。

## 5. 非目标

- 不为了跑分改变官方查询语义；
- 不把 HBase/RustFS 的调优结果写成官方核心性能；
- 不在未测生产配置前宣称容量。
