# 工作日志 G：零 Python 工具链与运行路径

日期：2026-10-26

## 背景与目标

在 A～F 整改中新增了多个 Python 门禁/基准脚本，同时仓库内仍保留早期 Python 候选实现和 K8s/grpc-web 渲染器。按“启动、构建、发布、运行路径不使用 Python；按场景选择 shell 或 Rust”的要求，本轮完成全量清理：

- 不保留实现目录中的 `*.py`；
- 不在 shell/YAML/Dockerfile 中调用 `python`/`python3`/`pip`；
- 不启动 Python 服务或 sidecar；
- 遗留 Python 实现只在 Rust 正式实现已存在并用测试证明后删除。

设计说明见 [`../09-python-free-toolchain.md`](../09-python-free-toolchain.md)。

## 改动

### 1. 新增门禁脚本改为 shell

- `link-u-smart-contract/all-in-one-deploy/localDeploy/verify-archive-bridge-methods.sh`
  - 用 `jq` 精确集合比较 14 个方法与 2 个监听器，逐监听器要求 POST/OPTIONS。
- `rtd-archive-service/modules/hbase-compat-gateway/verify-conformance.sh`
  - 用 `jq`/`git`/`shasum` 核对源码提交、5 个输入摘要、4 个 Java handler、5 类 filter、14 表及列族/版本。
- `rtd-archive-service/scripts/benchmark-query-plane.sh`
  - `curl` + `jq` + `awk` 保持原查询集、warmup、迭代、p50/p95/p99、mean/max、错误字段和退出码语义。

### 2. 构建、发布、SBOM、源码锁迁移

- `scripts/verify-source.sh`：提交、脏树、Cargo.lock、toolchain、六二进制源码入口检查。
- `scripts/generate-sboms.sh`：去除内嵌 Python，`syft` 扫描后由 `jq` 组装 SPDX 摘要与镜像 manifest。
- `scripts/prepare-source-overlay.sh`：临时 Git index + cached apply，保持补丁顺序、tree、Cargo.lock、changedFiles 和输出目录拒绝语义。
- `scripts/prepare-private-image-release.sh`：仓库主机校验、六镜像平台/OCI revision 预检、可选推送与远端 manifest 回读。
- `scripts/audit-source-licenses.sh` + `crates/rtd-archive-admin/src/license_inventory.rs` 的 `rtd-archive-admin license-inventory` 子命令：Rust 执行 Cargo metadata 非 dev 可达闭包、许可证表达式/许可证文件清单。
- 删除已退役的 `build-*.py`、`test_prepare_*.py`、`audit-runtime-deps.py`、`verify-image-policy.py`。
- 新增 shell 回归：`test-prepare-source-overlay.sh`、`test-prepare-private-image-release.sh`、`test-generate-sboms.sh`。

### 3. 启动与业务脚本去除内嵌 Python

- `localDeploy/lib/linku_sync.sh`：Nacos properties 合并改为纯 Bash；只替换目标 7 个键，缺失键追加，其余行字节不变，源文件无尾换行时输出同样无尾换行。
- 三个业务合约分片脚本：`create_all_shards*.sh` 的 JSON 组装改为 `jq -Rn … from_entries`，按 `shard_index` 数值排序。
- `get_my_shard.sh`：`int(hex,16) % 1024` 改为 Bash `16#${HEX: -3} % 1024`（模 1024 只依赖最后 3 个十六进制位）。

### 4. 遗留 Python 实现删除

删除 160 个遗留模块 Python 文件，删除前确认对应 Rust 实现/入口存在：

- `modules/account-activity` → `crates/rtd-archive-account`
- `modules/control-api` → `crates/rtd-archive-control`
- `modules/publication-fence`、`publication-guard`、`consistent-route` → `crates/rtd-archive-publication`、`crates/rtd-archive-query-gateway`
- `modules/object-retention` → `crates/rtd-archive-publication` 的 `rtd-archive-retention`
- `modules/indexer-alt/history_policy.py` → 原生 retention/history 入口

Rust 测试所需资产迁移：

- `modules/account-activity/schema.graphql` → `crates/rtd-archive-account/tests/fixtures/account-schema.graphql`
- `modules/control-api/control_bootstrap.sql` → `crates/rtd-archive-control/tests/fixtures/control_bootstrap.sql`
- 更新 `api.rs`、`protocol_contract.rs`、`verification_jobs_isolated.rs`、`backup_manifest_isolated.rs` 的 `include_*` 路径。

### 5. 撤回 Python-only 部署候选

`deploy/kubernetes`（39 个 Python 渲染/测试文件及 Python `consumer_proxy` sidecar 清单）和 `deploy/grpc-web`（4 个 Python 文件）没有已部署目标，且完全依赖 Python 渲染/运行。为避免保留不可执行的 Python 候选，本轮整目录撤回；生产编排和 grpc-web 入口须在真正生产化时用 Rust/shell 重建。相关当前文档已改为“已撤回候选”，历史 workLog 保留原记录。

### 6. 零 Python 审计门禁

新增 `scripts/audit-no-python.sh`，扫描 5 个实现根：

- `rtd-archive-service`（排除 `target`/`dist`/`docs`/`.git`）
- `localDeploy`
- 三个业务合约分片脚本目录

失败条件：出现 `*.py`、命令位置调用 `python`/`python3`/`pip`、Dockerfile 使用 Python 基础镜像。

## 验证

```text
$ all-in-one-deploy/localDeploy/verify-archive-bridge-methods.sh
archive bridge methods OK: 14 methods, 2 listeners

$ rtd-archive-service/scripts/verify-source.sh
source baseline OK: RTD 39a9b622c2

$ modules/hbase-compat-gateway/verify-conformance.sh
wide-table conformance OK: commit 39a9b622c2e6, 4 RPCs, 14 tables

$ scripts/audit-no-python.sh
zero-Python audit OK: 5 roots, no .py sources and no runtime python/pip calls

$ scripts/test-prepare-source-overlay.sh
source overlay contract OK: valid lock, 7 rejection cases, existing output preserved

$ scripts/test-prepare-private-image-release.sh
private image release contract OK: preflight, publish readback, 7 registry cases, 4 negative paths

$ scripts/test-generate-sboms.sh
SPDX manifest assembly OK: 6 images, scanner/revision/count fields checked

$ scripts/audit-cloud-policy.sh --strict
Open-source runtime policy scan OK: 82 deployment artifacts

$ scripts/audit-source-licenses.sh
...: 6 binaries; unresolved=0
```

Rust：

```text
cargo test -p rtd-archive-control --test protocol_contract   -> 2 passed
cargo test -p rtd-archive-account --test api                 -> 11 passed
cargo check --tests -p rtd-archive-account -p rtd-archive-control -> success
cargo check -p rtd-archive-admin --bin rtd-archive-admin         -> success
scripts/audit-source-licenses.sh                               -> 6 binaries; unresolved=0
```

启动脚本：

- 假 Nacos 全流程：7 个键更新/追加，其余行保持不变；源文件无尾换行时输出同样无尾换行。
- 分片脚本：jq 输出与旧 Python 输出按键数值排序一致。
- `get_my_shard.sh`：4 组地址的模 1024 结果与 Python 一致（495/1023/1023/700）。

性能脚本：

- 本地 mock GraphQL：3 个查询组、3 次迭代、错误 0，p50/p95/mean 字段正常。
- 连接失败路径：3 个查询组 `errors=1`、`first_error` 字符串正确、进程退出 1。

证据 JSON：[`evidence/2026-10-26-python-free-audit.json`](evidence/2026-10-26-python-free-audit.json)。

## 未完成/边界

- 生产 Kubernetes 编排、HBase/PG HA 清单、grpc-web 渲染器已撤回；生产化时须用 Rust/shell 重建并重新验收。本轮的“零 Python”不等于生产部署完成。
- SDK `fork-instruct` 一次性上游刷新脚本、`rtd-apps/.agents` 技能资源不在归档服务启动/构建/运行路径内，未纳入本轮；若要求整个工作区零 Python，需另立任务。
- `../rtd` 上游锁定源码保持原样；本工程不调用其 Python 工具。
- HBase 的 Ruby/JRuby 探针与初始化脚本不属于 Python，未改动。
- 性能基线数字仍是原 Python 脚本采集的历史 dev 样本；shell 版本保持相同查询与统计字段，后续生产基线应用 shell 版本重采。
- 激活项（发布器重建、bridge 容器重生成、真实 S3/HBase/性能矩阵）仍按 A～F 日志的待激活清单执行，不因本专题完成而关闭。

## 追溯

- 设计：`../fork-instruct/architecturePatch/09-python-free-toolchain.md`
- 状态：`../fork-instruct/architecturePatch/08-implementation-status.md`
- 证据：`workLog/evidence/2026-10-26-python-free-audit.json`
