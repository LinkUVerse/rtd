# 工作日志 F：非中间件查询平面基线

日期：2026-10-26

## 改动

- 新增 `rtd-archive-service/scripts/benchmark-query-plane.sh`
  - 固定 checkpoints / transactions / serviceConfig / 地址余额 / 地址交易查询；
  - 支持 warmup、迭代次数、超时、额外 header、JSON 输出；
  - 输出 p50/p95/p99、mean/max、最大响应字节、错误数；
  - 明确只测查询平面，不混入发布写入和中间件开销。

## 首轮结果（本机 dev，5 次迭代）

样本：`http://127.0.0.1:43412/graphql`，地址 `0xc535…0df9`。

| 查询 | p50 ms | p95 ms | 错误 |
| --- | ---: | ---: | ---: |
| checkpoints | 394.33 | 747.24 | 0 |
| transactions | 487.43 | 608.70 | 0 |
| serviceConfig | 194.65 | 557.86 | 0 |
| address balances | 364.57 | 594.84 | 0 |
| address transactions | 504.84 | 674.25 | 0 |

完整 JSON：`workLog/evidence/2026-10-26-query-plane-baseline.json`。

## 未完成/边界

- 这只是 dev 单机低并发样本，不是生产容量结论，也不与上游官方生产环境直接可比。
- 未采集 gRPC KV RPC、publication 认证/复制、indexer lag、account 首页/续页；这些仍需按设计扩展到各自指标。
- 时间包含本机 GraphQL ingress、PG、Ledger/Consistent 访问和 dev 网络；未拆分中间件/业务开销。
- 下一次需要固定生产配置、固定链高度样本和更高并发，形成发布门禁阈值。

## 追溯

- 设计：`../fork-instruct/architecturePatch/06-performance-baseline.md`
- 状态：`../fork-instruct/architecturePatch/08-implementation-status.md`
