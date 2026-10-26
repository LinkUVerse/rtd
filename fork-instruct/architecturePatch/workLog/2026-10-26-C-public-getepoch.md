# 工作日志 C：公共 gRPC 入口补齐 GetEpoch

日期：2026-10-26

## 改动

- `link-u-smart-contract/all-in-one-deploy/localDeploy/templates/middlewares/archive-bridge.conf`
  - 两个监听器（`archive_browser`、`archive_browser_loopback`）各增加 `LedgerService/GetEpoch` 的 POST 与 OPTIONS 路由。
  - 每监听器路由数 26 → 28，方法数 13 → 14。
- 新增 `link-u-smart-contract/all-in-one-deploy/localDeploy/verify-archive-bridge-methods.sh`
  - 校验公共 bridge 恰好包含 10 个 Ledger + 4 个 MovePackage 方法；
  - 校验每个方法都有 POST/OPTIONS；
  - 防止后续只更新内部 ingress 而漏掉公共入口。

## 验证

```text
$ all-in-one-deploy/localDeploy/verify-archive-bridge-methods.sh
archive bridge methods OK: 14 methods, 2 listeners
```

模板 JSON 解析通过；两个监听器均为 28 条路由（14 方法 × 2 HTTP 方法）。

## 未完成/边界

- 当前运行中的 `archive-bridge` 容器仍使用旧的生成配置；需要重新生成 middlewares 配置并重建该容器后，公共端口才会出现 `GetEpoch`。
- 本轮只做静态模板/方法一致性校验，未对公共 mTLS 端点发真实 `GetEpoch` 请求；激活后应补运行正/负例。
- 未改动内部 `query-ingress`；它本来就允许全部方法。

## 追溯

- 设计：`../fork-instruct/architecturePatch/03-query-api-surface.md`
- 状态：`../fork-instruct/architecturePatch/08-implementation-status.md`
