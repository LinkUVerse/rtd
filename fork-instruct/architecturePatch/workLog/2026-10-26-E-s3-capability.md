# 工作日志 E：S3 能力预检

日期：2026-10-26

## 改动

- `rtd-archive-service/crates/rtd-archive-publication/src/publisher.rs`
  - 在版本化检查之后，对 staging/published 各执行一次单 key、单页的 `ListObjectVersions` 探测；
  - 不支持版本列举、超时或协议不一致时，发布在产生任何写入前失败；
  - 不自动降级到无版本模式。

## 验证

```text
$ cargo check -p rtd-archive-publication
Finished `dev` profile

$ cargo test -p rtd-archive-publication --lib
41 passed; 0 failed; 1 ignored
```

现有 S3/publisher 单测通过。版本化检查已有实现，新增的版本列举探针使用现有 `list_key_versions_page` 路径。

## 未完成/边界

- 未在目标 S3 兼容存储上执行“未开启版本化 / 不支持版本列举”的负例；需要在隔离环境补真实预检。
- `If-None-Match`、用户元数据、COPY 结果等能力由现有发布路径和单测覆盖，但未形成独立预检矩阵。
- HBase S3A 的 rename/list/multipart 兼容性不属于 publication 预检，仍是中间件独立验收。
- 文档已明确“任意 S3 兼容”应理解为支持版本化/版本列举等最小能力，而不是所有 S3 实现无条件可用。

## 追溯

- 设计：`../fork-instruct/architecturePatch/05-s3-capability.md`
- 状态：`../fork-instruct/architecturePatch/08-implementation-status.md`
