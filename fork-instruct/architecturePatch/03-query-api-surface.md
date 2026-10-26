# 专题 C：公共查询入口与 GraphQL surface

## 1. 官方查询面

官方 GraphQL schema 包含：

- Query 37 个根字段：地址、资产、对象、交易、事件、检查点、纪元、包、类型、元数据、配置、模拟与验签；
- Mutation 1 个：`executeTransaction`；
- Subscription 3 个：checkpoints、events、transactions。

官方端点：

- `POST /graphql`（查询/变更）；
- `GET /graphql`（Web IDE，可由 `--no-ide` 关闭）；
- `/graphql/subscriptions`（SSE，需要 Fullnode checkpoint stream）；
- `/graphql/health`；
- `/metrics`。

## 2. 当前 RTD 状态

- 主 schema 与当前 fork HEAD 基本一致；37 个 Query 根全部存在，仅 deprecated 字段反射可见性差异；另有 RTD 的 DisplayHistory 扩展。
- 只读查询后端（PostgreSQL、Ledger gRPC、Consistent Store）已接入；地址、对象、交易、事件、检查点、纪元、包、余额、serviceConfig 等主路径可用。
- 公共 gRPC `archive-bridge` 只路由 9/10 个 Ledger 方法，缺 `GetEpoch`；内部 ingress 允许全部方法，两个入口不一致。
- 公共 GraphQL ingress 只有 POST `/graphql`，没有 `/graphql/subscriptions`、GET IDE、health、metrics。
- 本地 GraphQL 进程未带 `--checkpoint-stream-url`，订阅未启用；K8s 清单也未配置订阅。
- dev 进程未带 `--fullnode-rpc-url`；生产 K8s 清单有该参数。因此执行/模拟在 dev 不可用，生产需单独验收。
- `/account/graphql` 是自定义业务 API，不属于官方 GraphQL。

## 3. 必须项

### 3.1 公共 gRPC 补 `GetEpoch`

- 在 archive bridge 的显式路由中加入 `LedgerService/GetEpoch` 的 POST/OPTIONS；
- 入口方法清单与官方 proto 的 14 个 Ledger/MovePackage 方法一致；
- 增加静态/运行合同测试，防止后续新增方法只进入内部 ingress。

### 3.2 GraphQL 能力声明

- 明确 RTD 的公共 GraphQL surface：当前是“只读查询 GraphQL”，不是完整官方服务；
- 若业务需要执行/模拟，在生产配置 `--fullnode-rpc-url` 并验收；
- 若业务需要订阅，配置 `--checkpoint-stream-url`、增加 `/graphql/subscriptions` 路由，并完成实际流/断线/权限验收；否则文档不得宣称可用。

## 4. 可选项

- GET IDE、`/graphql/health`、`/metrics`：按运维需要补，不影响查询 schema。
- CORS：生产 Web 客户端需要时增加受控 origin。
- `/account/graphql`：保留为 RTD 扩展；若严格官方 API 口径，必须移出官方命名空间或标记为 BFF。

## 5. 验收

1. 公共 gRPC 反射/路由方法数 = 官方 14 个方法。
2. `GetEpoch` 在公共入口返回与内部入口相同的错误/数据语义。
3. GraphQL 37 个 Query 根中，依赖已配置后端的读字段返回数据；Fullnode 依赖字段在未配置时明确 `FEATURE_UNAVAILABLE`，不返回伪数据。
4. 订阅未配置时 schema 存在但明确不可用；配置后完成流与准入回归。
5. 自定义账户 API 与官方 `/graphql` 在文档、入口、错误码上明确分离。
