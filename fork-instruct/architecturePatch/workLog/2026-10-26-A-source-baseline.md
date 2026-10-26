# 工作日志 A：源码基线与可复现构建

日期：2026-10-26

## 改动

- `rtd-archive-service/source.lock.json`
  - path：`../temp/rtd` → `../rtd`
  - commit：`39a9b622c2e6e464ed7b70b254d2f15089e1a0f5`
  - Cargo.lock SHA-256：`1b546482bc61f5a3e6f041d0b91ee029eb510b6bc459b5ef0a420ae40b97c4f7`
  - toolchain：`1.96.1`
- `rtd-archive-service/scripts/verify-source.sh`
  - 来源树脏检查忽略 `fork-instruct/`：该目录只承载本次审计文档与工作日志，不属于构建输入。
- `rtd-archive-service/docs/architecture/08-源码与状态对照.md`
  - 构建基线更新为当前运行态提交；明确历史候选锁不作为当前构建输入；PG 管线描述更新为 22 条官方 + 1 条 RTD Display 扩展。
- `rtd-archive-service/ARCHITECTURE.md`
  - 基线路径更新为 `../rtd`。

## 验证

```text
$ scripts/verify-source.sh
source baseline OK: RTD 39a9b622c2
```

后续补充：`verify-source.sh` 允许锁定构建提交之后仅修改 `fork-instruct/` 的后代提交，并在输出中区分 `RTD build <锁定提交>` 与 `docs HEAD <后代提交>`；SBOM/active manifest 仍绑定锁定构建提交。这样架构文档可以随同一分支提交，不会把文档提交误当成新的构建基线。

运行态依据：`rtd/dist/native/active.json` 的 `sourceSnapshot.head` 为 `39a9b622…`，`workingDiffSha256` 为空，`patches` 为空。

## 未完成/边界

- `dist/sbom/f4cac7d…` 为历史 SBOM，未重新生成；下一次正式发布构建必须重建 SBOM 并绑定当前 active manifest。
- 50+ 候选补丁仍保留为历史/候选资产；当前运行未应用，不得被默认生产构建引用。
- 本次只锁定源码基线，不改变运行中的二进制；要激活任何补丁，必须重新构建并重跑 API parity。

## 追溯

- 设计：`../fork-instruct/architecturePatch/01-source-baseline.md`
- 状态：`../fork-instruct/architecturePatch/08-implementation-status.md`
