# 工作日志 D：宽表适配 conformance 门禁

日期：2026-10-26

## 改动

- 新增 `rtd-archive-service/modules/hbase-compat-gateway/conformance.json`
  - 固定源码提交 `39a9b622…`；
  - 固定宽表客户端、KV RPC 客户端、表常量、网关 PROTOCOL、HBase DDL 的 SHA-256；
  - 固定 4 个 RPC、5 类 filter、14 张表。
- 新增 `rtd-archive-service/modules/hbase-compat-gateway/verify-conformance.sh`
  - 校验 `source.lock` 与当前 `rtd` HEAD；
  - 校验输入文件摘要；
  - 校验网关 Java 仍实现 4 个 RPC 对应 handler、filter 处理仍存在；
  - 校验 DDL 仍使用 `rtd` 列族、`VERSIONS => 1` 和 14 张表。

## 验证

```text
$ modules/hbase-compat-gateway/verify-conformance.sh
wide-table conformance OK: commit 39a9b622c2e6, 4 RPCs, 14 tables
```

## 未完成/边界

- 这是静态门禁，不替代 14 管线回填、三类 List、CAS、正反向扫描、大值分块、部分失败和 HBase 故障回归。
- HBase WAL 多节点持久/恢复、ZooKeeper 生产 HA、S3A 目标存储兼容仍是中间件生产验收项。
- 预分区/压缩/Bloom/Region 规划仍未固化；属于性能和容量项，不改变行键/列族/时间戳语义。

## 追溯

- 设计：`../fork-instruct/architecturePatch/04-hbase-conformance.md`
- 状态：`../fork-instruct/architecturePatch/08-implementation-status.md`
