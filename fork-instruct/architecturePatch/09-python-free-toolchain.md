# 零 Python 工具链与运行路径

## 1. 目标与约束

本专题把 RTD 归档服务及其本地部署实现中的 Python 依赖清零：

- 启动、构建、发布、验证、基准和本地开发脚本不再调用 `python`/`python3`/`pip`。
- 运行期间不启动任何 Python 服务或 sidecar；容器基础镜像不再使用 `python:*`。
- 遗留 Python 候选实现只在与 Rust 正式实现完成职责对照后删除，不保留“可执行但未使用”的 Python 副本。
- 按场景选择 shell 或 Rust：短小的文件/JSON/HTTP 门禁使用 shell + `jq`/`curl`；需要遍历 Cargo 依赖图的许可证清单使用 Rust。

不改变既有硬约束：官方 Rust 管线同源、HBase 3.0／S3A、任意 S3 兼容存储、自建 PostgreSQL、无上游品牌字样。

## 2. 范围分类

| 范围 | 处置 | 说明 |
| --- | --- | --- |
| `rtd-archive-service/scripts/**` | 全部迁移或删除 | 构建、发布、SBOM、源码锁、审计、基准脚本 |
| `rtd-archive-service/modules/**` | 删除 Python 实现，保留 Java/Rust/模板资产 | account/control/publication/route 已由 `crates/*` 承接 |
| `rtd-archive-service/deploy/kubernetes`、`deploy/grpc-web` | 整目录撤回 | 仅 Python 渲染器/测试与未部署候选；生产编排待 Rust/shell 重建 |
| `link-u-smart-contract/all-in-one-deploy/localDeploy` | 去除内嵌 Python | Nacos properties 合并、bridge 方法门禁 |
| `link-u-smart-contract/*_record/script`、`link_u_profile_center` | 去除 Python | 分片 JSON 组装、地址取模 |
| 历史 `docs/architecture/workLog`、`architecturePatch/workLog` | 保留文字记录 | 只读历史，不是可执行路径；本专题新增的结论写入本目录 |
| `../rtd` 上游源码 | 不改 | 锁定源码；本工程构建/启动不调用其中的 Python 工具 |
| SDK `fork-instruct*` 一次性迁移脚本、`rtd-apps/.agents` 技能资源 | 不在本轮运行路径内 | 非归档服务启动/构建/运行依赖；如后续纳入交付，须另行迁移 |

## 3. 迁移矩阵

| 原 Python 入口 | 新入口 | 选择 | 验证 |
| --- | --- | --- | --- |
| `scripts/verify-source.py` | `scripts/verify-source.sh` | shell + git/jq/shasum | 当前锁 `39a9b622…` 通过 |
| `scripts/generate-sboms.sh` 内嵌 Python | 同名 shell + jq | shell | 假 syft/docker 测试 6 镜像清单 |
| `scripts/prepare-source-overlay.py` | `scripts/prepare-source-overlay.sh` | shell + 临时 Git index | 7 类锁篡改拒绝 + 产出目录不覆盖 |
| `scripts/prepare-private-image-release.py` | `scripts/prepare-private-image-release.sh` | shell + docker/jq | 预检、发布回读、7 类仓库负例 |
| `scripts/audit-source-licenses.py` | `scripts/audit-source-licenses.sh` + `rtd-archive-admin license-inventory` Rust 子命令 | Rust | 6 二进制 `unresolved=0` |
| `scripts/benchmark-query-plane.py` | `scripts/benchmark-query-plane.sh` | shell + curl/jq/awk | 成功/连接失败两条路径 |
| `modules/hbase-compat-gateway/verify-conformance.py` | `verify-conformance.sh` | shell + jq/git/shasum | 4 RPC/14 表通过 |
| `localDeploy/verify-archive-bridge-methods.py` | `verify-archive-bridge-methods.sh` | shell + jq | 14 方法/2 监听器通过 |
| `localDeploy/lib/linku_sync.sh` 内嵌 Python | 纯 Bash 行合并 | shell | 假 Nacos 全流程；其余行字节不变；尾换行保持 |
| 分片脚本内嵌 Python | `jq -Rn … from_entries` | shell | 与旧输出按键数值排序一致 |
| `get_my_shard.sh` 的十六进制取模 | Bash `16#${HEX: -3} % 1024` | shell | 多组地址与 Python 结果一致 |
| 遗留 account/control/fence/guard/route/retention Python 模块 | 删除；Rust 已承接 | Rust | 账号 API、控制协议、发布可见性定向测试编译/通过 |
| K8s/grpc-web Python 渲染器与 Python sidecar | 撤回候选 | 待 Rust/shell 重建 | 本地开发与业务查询不依赖该目录 |

## 4. 遗留模块删除与资产迁移

删除前确认 Rust 侧已存在正式实现或等效入口：

- `modules/account-activity` → `crates/rtd-archive-account`；`schema.graphql` 迁入 `crates/rtd-archive-account/tests/fixtures/account-schema.graphql`。
- `modules/control-api` → `crates/rtd-archive-control`；`control_bootstrap.sql` 迁入 `crates/rtd-archive-control/tests/fixtures/control_bootstrap.sql`；`change_*.sql` 已有 `crates/rtd-archive-control/migrations/legacy` 同内容副本。
- `modules/publication-fence`、`publication-guard`、`consistent-route` → `crates/rtd-archive-publication` 与 `crates/rtd-archive-query-gateway`。
- `modules/object-retention` → `crates/rtd-archive-publication` 的 `rtd-archive-retention`。
- `modules/indexer-alt/history_policy.py` → `crates/rtd-archive-publication` 的 retention/history 策略入口。

删除的测试资产只在 Rust 测试中作为 fixture 使用，不执行 Python。

## 5. 零 Python 审计门禁

新增 `rtd-archive-service/scripts/audit-no-python.sh`，覆盖：

- `rtd-archive-service`（排除 `target`/`dist`/`docs`/`.git`）；
- `link-u-smart-contract/all-in-one-deploy/localDeploy`；
- 三个业务合约的分片脚本目录。

门禁失败条件：

1. 任一实现目录出现 `*.py`；
2. 任一 shell/YAML/Dockerfile/Makefile 以命令位置调用 `python`/`python3`/`pip`；
3. Dockerfile 使用 `python` 基础镜像。

```sh
rtd-archive-service/scripts/audit-no-python.sh
```

`scripts/verify-source.sh` 在通过源码锁检查后调用该门禁，因此正常构建/发布前的来源核验也会拒绝重新引入的 Python。

## 6. 验证结果

```text
archive bridge methods OK: 14 methods, 2 listeners
source baseline OK: RTD 39a9b622c2
wide-table conformance OK: commit 39a9b622c2e6, 4 RPCs, 14 tables
zero-Python audit OK: 5 roots, no .py sources and no runtime python/pip calls
source overlay contract OK: valid lock, 7 rejection cases, existing output preserved
private image release contract OK: preflight, publish readback, 7 registry cases, 4 negative paths
SPDX manifest assembly OK: 6 images, scanner/revision/count fields checked
Open-source runtime policy scan OK: 82 deployment artifacts
```

Rust 定向测试：

- `cargo test -p rtd-archive-control --test protocol_contract`：2 通过。
- `cargo test -p rtd-archive-account --test api`：11 通过。
- `cargo check --tests -p rtd-archive-account -p rtd-archive-control`：通过。
- `cargo check -p rtd-archive-admin --bin rtd-archive-admin`：通过。
- `scripts/audit-source-licenses.sh`：6 二进制 `unresolved=0`。

## 7. 边界与后续

- 生产 Kubernetes 编排、HBase/PG HA 清单和 grpc-web 渲染器原为未部署候选，已撤回；生产化时必须用 Rust/shell 重建并重新做静态/集群验收，不能把本专题当作生产部署完成。
- 本次未迁移 SDK `fork-instruct` 一次性上游刷新脚本与 `rtd-apps/.agents` 技能资源；它们不属于归档服务启动/构建/运行路径。若要求“整个工作区零 Python”，需另立任务逐项迁移。
- `../rtd` 上游锁定源码保持原样；本工程不调用其 Python 工具，也不修改其构建输入。`source.lock.json` 的 commit 仍是构建基线；`fork-instruct/` 文档提交可以作为其后代存在，`verify-source.sh` 只在这种文档-only 后代场景输出独立的 `docs HEAD`。
- 性能基线数字仍是原 Python 脚本采集的历史 dev 样本；shell 版本保持相同查询、warmup、迭代和统计字段，后续生产基线应使用 shell 版本重采。
- HBase 的 `init-tables.rb`/`wal-fs-capability.rb` 属于 HBase JRuby 环境，不是 Python；本次未改变。
- 历史 `docs/architecture/workLog/evidence/*.json` 中记录的 Python 文件哈希对应已删除的历史快照，仅用于追溯当时证据，不代表当前路径仍存在；当前门禁以 `audit-no-python.sh` 和本专题证据为准。
