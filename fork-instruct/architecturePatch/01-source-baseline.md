# 专题 A：源码基线与可复现构建

## 1. 当前问题

运行态、锁文件和文档存在三套源码标识：

| 来源 | 当前值 | 说明 |
| --- | --- | --- |
| 正在运行的官方二进制 | `rtd/dist/native/releases/.../manifest.json`：head `39a9b622…`，working diff 为空，patches 为空 | 实际运行的是未打补丁的源码构建 |
| `rtd-archive-service/source.lock.json` | `f4cac7d…`，路径 `../temp/rtd` | 该路径不存在；锁与运行态不一致 |
| `docs/architecture/08-源码与状态对照.md` | `9a2c188…` | 第三套标识 |
| 候选补丁锁 | 多数以 `f4cac7d` 为 base | 当前未应用于运行态 |

这导致：`verify-source.sh` 无法通过；SBOM 不能覆盖运行二进制；任何“已通过 parity”的结论不能自动适用于当前运行态。

## 2. 目标

- 唯一源码基线：选择运行态实际使用的源码提交，或一个明确的上游已验证提交。
- `source.lock.json`、active manifest、SBOM、架构文档和候选锁都引用同一个提交。
- 候选补丁明确标注“生产启用 / 仅候选”，不再混用。
- 所有官方二进制可以从锁定源码和工具链重现。

## 3. 实施方案

### 3.1 锁定运行态

将 `source.lock.json` 更新为当前实际运行的源码：

- path 指向真实源码目录（本机为 `../rtd`，CI 使用受控 checkout）。
- commit 使用运行态 manifest 的 `sourceSnapshot.head`。
- `cargoLockSha256` 使用该提交的 `Cargo.lock` 摘要。
- `rustToolchain` 使用该提交的 `rust-toolchain.toml` channel。
- `rtdApis` 保持与 GraphQL/gRPC 生成物一致的提交。

### 3.2 生成/更新证据

- 用同一源码快照重建六个官方二进制，记录 SHA-256、rustc、Cargo.lock 摘要。
- 更新 active manifest 和 SBOM，禁止用历史镜像标签代替当前构建证据。
- 在 `docs/architecture/08-源码与状态对照.md` 记录当前基线、构建命令和证据路径。

### 3.3 候选补丁登记

- 保留 `modules/indexer-alt/patches/*` 作为候选资产，不删除。
- 在补丁 README 和本目录文档中列出：哪些补丁已经并入目标源码、哪些只用于候选镜、哪些已废弃。
- 生产构建只能从“已启用补丁集合”生成；不能在部署时临时 apply 未登记补丁。

### 3.4 `fork-instruct/` 文档提交

`source.lock.json` 的 commit 表示**构建输入基线**，不要求等于 `rtd` HEAD。允许锁定提交之后存在仅修改 `fork-instruct/` 的后代提交（本目录的架构文档和工作日志），因为该目录不是构建输入，现有脏树检查也一直排除它。`verify-source.sh` 仅在后代提交的全部变更路径都位于 `fork-instruct/` 时接受，并输出“docs HEAD”提示；任何构建输入路径的变更仍然失败。SBOM 和 active manifest 继续绑定锁定构建提交。

## 4. 验收

1. `scripts/verify-source.sh --rtd-root <clean checkout>` 退出码 0。
2. active manifest 的 `sourceSnapshot.head` 与 `source.lock.json` 完全一致。
3. 六个二进制的重建 SHA 与 active manifest 一致（允许构建可复现性差异时，必须记录差异原因）。
4. `docs/architecture/08-源码与状态对照.md` 不再出现三套 commit。
5. 候选补丁状态可机读：是/否进入目标构建。

## 5. 边界

- 不修改官方核心源码行为来“凑”基线；若需要改，先形成显式补丁并进入补丁清单。
- 不把旧候选锁删除作为解决方式；保留为历史证据，但不得被当前部署引用。
