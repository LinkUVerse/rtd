# 原生修剪与启动凭据

修剪继续由本 crate 的 handler 与 `rtd-indexer-alt-framework` 执行。Shell 不计算业务删除集合、不写 `reader_lo`/`pruner_hi`，不执行代替 handler 的 DELETE。现行开发配置的全局 `[pruner]` 只是默认值；没有逐 pipeline 的 `pruner` 节时不会启用修剪。

明确启用某条受支持 pipeline 时，使用原生 TOML 配置，例如：

```toml
[pipeline.kv_transactions.pruner]
interval-ms = 1000
delay-ms = 120000
retention = 4000000
max-chunk-size = 2000
prune-concurrency = 1
```

`retention` 的单位为检查点，读取下界为 `(checkpoint_hi_inclusive + 1).saturating_sub(retention)`。Reader 先看到新下界；修剪器等待数据库时间计算的 `delay-ms` 后才删除，并持久推进自己的水位。重启复用原水位与宽限期，不重新解释一个较早的边界。必须按业务历史查询要求确定保留策略；此示例不授权对当前环境启用修剪。

本地维护及普通启动可使用 `indexer --database-url-file /absolute/private/writer.url`，替代在进程参数中传密码。文件要求当前用户所有、无组/其他人权限、普通文件、末级无符号链接且不超过 4096 字节；内容为单个 PostgreSQL URL，可带一个结尾换行。错误不回显内容。与显式 `--database-url` 互斥；未指定文件时旧 URL 参数行为保持。运行中不会重新载入凭据，轮换按停止和重启入口执行。

对持续发布源仍保留 `--require-published-watermark`；保留数据的写入仍保留 `--require-exclusive-writer` 和数据库 generation fence。修剪不是旁路 writer。`obj_versions` 保留每个对象在下界前的最后版本及删除标记，并在一个原子 SQL 语句中修剪对应内容。不能把对象版本索引简单按检查点截断。

2026-10-15 原生验收见正式归档工程的 `crates/rtd-archive-query-gateway/tests/native_pruning_isolated.rs`：真实 36 个检查点、22 条管线、三种原生 handler、数据库时间宽限期、SIGKILL 续跑、独占写者拒绝、双表删除故障回滚、逐字节保留区间与锚点、生产 IndexReader 下界、pg_dump/pg_restore 同链恢复。部署工程的 `tests/native_pruning_test.sh` 只提供 Shell 夹具编排。

HBase/KV 目前没有已支持的物理修剪 handler。`rtd-kvstore-alt --validate-config` 可在访问中间件前检查配置；不受支持的全局/逐 pipeline `pruner` 与未知 pipeline 均拒绝。它不表示已经完成 KV 删除协议。S3 published/replica/backup 永久保留及 quarantine 的拒删门独立保留，不能借 PostgreSQL 修剪通过来授权对象存储删除。
