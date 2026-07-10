# 上游补丁合并记录

本目录审计 RTD fork 点 `35d0fac50adb310ca9049af5a933336adeaac468` 之后的 Sui 上游提交，只移植直接修复 fork 基线既有代码缺陷的 bugfix/patch。上游 feature、非缺陷重构、以及只修复 fork 后新增功能的提交均排除。

## 固定范围

- RTD fork 基线：`35d0fac50adb310ca9049af5a933336adeaac468`
- Sui 审计仓库：`/Users/changzechuan/WenchuanProjects/RTD-Blockchain/temp/sui`
- 本轮固定上游头：`94c67f729f870593244b368203f8456ae8246e7a`
- 审计提交范围：`35d0fac...94c67f7`，共 1819 个提交
- RTD 工作分支：`codex/merge-upstream-patches`

如果 Sui `main` 在审计期间继续前进，应先完成上述固定范围，再把新增范围作为后续批次追加，不能静默改变本轮边界。

## 审计入口

- [实现计划](implementation-plan.md)
- `upstream-commit-inventory.tsv`：全部 1819 个提交的唯一总账
- `review-batches/`：逐批提交判定和证据
- `merged-patches/`：实际移植补丁的详细记录

## 判定原则

1. 标题中的 `fix`、`bug`、`patch` 只用于发现候选，不能直接决定合并。
2. 必须阅读完整 diff，并确认被修复的缺陷代码在 fork 基线已经存在。
3. 若缺陷代码由 fork 后 feature 引入，即使上游标题是 bugfix，也标记为 `EXCLUDE_POST_FORK`。
4. 混合提交只提取能独立证明适用于基线的最小修复 hunk，不连带移植 feature 或重构。
5. RTD 已有等价修复标记为 `ALREADY_PRESENT`，记录对应本地提交或代码证据。
6. 移植代码必须遵守仓库品牌映射，不保留上游产品/组织标识符。

## 当前进度

- 已固定审计范围并确认共有 1819 个上游提交。
- 总账已生成并通过 1819 行、SHA 无重复检查。
- 已人工审查第 1-50 个提交：16 个 `MERGE` 已全部移植；其余 34 个已按逐提交证据排除。
- 已移植上游 `a817cc2d`、`0662e882`、`34f9a3e3`、`0b0dc2a8`、`101d3b79`、`da34bd87`、`d3466348`、`c748f4b4`、`dbd8cfa7`、`f90d706a`、`ebc792b4`、`2b0a76ef`、`d1182c00`、`a342a9ed`、`c3a01cac`、`c69bda53`，详细记录见 `merged-patches/`。
