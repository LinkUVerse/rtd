# 专题实施状态

更新日期：2026-10-26。

| 专题 | 文档 | 代码/配置 | 验证 | 状态 |
| --- | --- | --- | --- | --- |
| A 源码基线 | `01-source-baseline.md` | `source.lock.json`、`scripts/verify-source.sh`、架构文档 | `verify-source.sh` 通过 | 完成（SBOM 重建待发布构建） |
| B checkpoint 产物 | `02-checkpoint-artifacts.md` | `crates/rtd-archive-publication/src/artifacts.rs`、publisher、checkpoint README | `cargo test -p rtd-archive-publication --lib` 41 通过 | 代码完成；需在启用 publisher 的隔离环境做真实发布验收 |
| C 公共查询面 | `03-query-api-surface.md` | `templates/middlewares/archive-bridge.conf`、`verify-archive-bridge-methods.sh` | 14 方法/2 监听器校验通过 | 模板完成；现行容器需重建入口后生效 |
| D HBase conformance | `04-hbase-conformance.md` | `conformance.json`、`verify-conformance.sh` | 4 RPC/14 表校验通过 | 完成静态门禁；运行回归按矩阵执行 |
| E S3 能力 | `05-s3-capability.md` | `publisher.rs` 版本列举预检、文档 | `cargo check` 通过 | 代码完成；真实存储负例待隔离验收 |
| F 性能基线 | `06-performance-baseline.md` | `scripts/benchmark-query-plane.sh` | 首轮 dev 样本已记录 | 脚本完成；生产配置基线待采集 |
| G 零 Python 工具链 | `09-python-free-toolchain.md` | shell/Rust 替换、遗留 Python 移除、零 Python 审计 | 全部门禁与定向测试通过 | 完成；生产 K8s 编排待 Rust/shell 重建 |

## 实施原则

- 代码改动必须保持失败关闭，不以空结果或旧缓存掩盖依赖/版本问题。
- 文档中的“完成”只代表代码/静态检查完成；涉及运行中的发布器、入口容器和真实对象存储的项目，在 workLog 中单独标记“待激活/待现场验收”。
