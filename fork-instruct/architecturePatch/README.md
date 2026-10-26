# RTD 归档服务对标整改（architecturePatch）

## 1. 目标

在保留以下硬约束的前提下，使 RTD 归档服务的查询能力、归档产物完整性和运行健壮性对标上游官方归档栈：

- 中间件替换不变：HBase 3.0（对象目录使用 S3A，WAL 独立）、任意 S3 兼容对象存储、自建 PostgreSQL。
- 不改写官方核心管线：checkpoint blob、PostgreSQL indexer、consistent store、KV store、KV RPC、GraphQL 继续复用同源构建。
- 不破坏现有业务：钱包账户历史、发布认证、只读查询、gRPC/GraphQL 客户端。
- 新增代码和注释只使用 RTD 命名，不出现上游品牌字样；遵循 `COMPLETE_FORK_GUIDE.md`。

## 2. 已确认的必须补齐项

| 编号 | 专题 | 目标 |
| --- | --- | --- |
| A | 源码基线 | 运行态、source lock、SBOM、文档使用同一份可复现源码 |
| B | checkpoint 产物 | published 桶补齐官方恢复所需的 `epochs.json`；`.chk` 按管线启用情况处理 |
| C | 公共查询入口 | 公共 gRPC 补齐 `GetEpoch`；明确 GraphQL 公共 surface |
| D | HBase 适配 | 锁定客户端版本与 RPC/过滤器子集，升级时必须回归 |
| E | S3 能力 | 明确并预检 publication 所需的 S3 能力，不把“任意 S3”说成无条件兼容 |
| F | 性能基线 | 对非中间件开销建立可复核基准，区分官方核心与 RTD 发布/账户层 |

**附加硬约束（G）**：启动、构建、发布、验证和运行路径不得使用 Python；按场景使用 shell 或 Rust。详见 [`09-python-free-toolchain.md`](09-python-free-toolchain.md) 和 [G 工作日志](workLog/2026-10-26-G-python-free-toolchain.md)。

## 3. 明确不作为必须项

以下可以保留现状或在业务确认后再做：

- 自定义 `/account/graphql`：作为钱包业务扩展保留，但不得混称官方 GraphQL。
- `hist_displays` 与 DisplayHistory 字段：RTD 业务扩展，保留但单独登记。
- GraphQL 订阅、GET IDE、`/health`、`/metrics`、Web CORS：业务需要时再补。
- Consistent Store B/C 高可用与自定义 route：官方基线本身是单一 consistent store URL；除非业务要求 HA。
- 上游默认修剪策略：RTD 采用无限保留属于业务策略，明确文档即可。
- 上游最新版本新增的类型（如新的签名算法、订阅边类型）：以固定目标版本为准，不追求随时追 main。

## 4. 工作流

1. 每个专题先在本目录写设计文档（输入、约束、改动点、拒绝路径、验收）。
2. 再在对应仓库实现，所有改动必须可回滚、可失败关闭。
3. 每个专题在 `workLog/` 写独立工作日志，记录：
   - 改动文件与关键行为；
   - 测试命令、运行结果；
   - 未覆盖项和已知边界；
   - 与上游契约的对应关系（不写品牌字样）。
4. 最后按 `07-implementation-order.md` 的顺序做统一回归。

## 5. 验收原则

- **失败关闭**：无法证明版本、链身份、连续前缀、摘要或能力时，返回错误，不返回空数据。
- **不引入新的 LIST 热路径**：发布/查询热路径不得按归档规模做对象列举。
- **契约优先**：先满足官方对象路径、字段、方法、分页和错误语义，再做 RTD 扩展。
- **可审计**：每个发布、重建、修复动作都有可复核的输入摘要和输出回执。
- **性能区分**：中间件替换导致的性能差异单列；RTD 自建发布层、账户层的开销必须单独度量。
