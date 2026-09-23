# Sui 上游补丁移植实现计划

> **面向 AI 代理的工作者：** 使用 `executing-plans` 逐批执行本计划。步骤使用复选框（`- [ ]`）跟踪进度。

**目标：** 完整审查 Sui `35d0fac50adb310ca9049af5a933336adeaac468..94c67f729f870593244b368203f8456ae8246e7a` 的 1819 个提交，只把直接修复 fork 基线既有代码缺陷的补丁移植到 RTD，并保留可复核的逐提交证据。

**架构：** 用一份包含全部上游提交的 TSV 总账证明筛查覆盖度；每个审查批次记录提交级结论和依据；每个实际移植的补丁单独记录上游 diff、RTD 路径/品牌映射、测试和本地提交。候选判定不能只依赖提交标题，必须检查 diff 及缺陷代码相对 fork 基线的来源。

**技术栈：** Git、Rust/Cargo、Move、Markdown、TSV。

---

## 文件结构

- 创建：`doc/mergeUpstreamPatch/README.md`，说明范围、判定规则、当前进度和审计入口。
- 创建：`doc/mergeUpstreamPatch/upstream-commit-inventory.tsv`，逐行覆盖 1819 个上游提交。
- 创建：`doc/mergeUpstreamPatch/review-batches/*.md`，记录每批逐提交人工结论。
- 创建：`doc/mergeUpstreamPatch/merged-patches/*.md`，记录每个已移植补丁的代码和验证细节。
- 修改：与适用补丁对应的 RTD 源码和测试；路径按 `sui` 到 `rtd`、`mysten` 到 `linku` 的仓库规则映射。

### 任务 1：建立完整上游提交总账

- [x] **步骤 1：固定审计边界**

运行：

```bash
git -C /Users/changzechuan/WenchuanProjects/RTD-Blockchain/temp/sui rev-list --count \
  35d0fac50adb310ca9049af5a933336adeaac468..94c67f729f870593244b368203f8456ae8246e7a
```

预期：输出 `1819`。

- [x] **步骤 2：生成逐提交 TSV 总账**

总账按提交拓扑的时间顺序列出序号、SHA、提交日期、标题、审查状态、结论、RTD 提交和备注。初始审查状态为 `UNREVIEWED`，不得把标题关键词预判当作最终结论。

- [x] **步骤 3：验证总账完整性**

运行：

```bash
test "$(($(wc -l < doc/mergeUpstreamPatch/upstream-commit-inventory.tsv) - 1))" -eq 1819
cut -f2 doc/mergeUpstreamPatch/upstream-commit-inventory.tsv | tail -n +2 | sort | uniq -d
```

预期：第一条命令成功，第二条命令无输出。

### 任务 2：逐批判定上游提交

- [ ] **步骤 1：按每批 50 个提交读取元数据和 diff**

对每个提交运行：

```bash
git -C /Users/changzechuan/WenchuanProjects/RTD-Blockchain/temp/sui show \
  --stat --summary --find-renames <UPSTREAM_SHA>
git -C /Users/changzechuan/WenchuanProjects/RTD-Blockchain/temp/sui show \
  --format=fuller --find-renames <UPSTREAM_SHA>
```

- [ ] **步骤 2：检查缺陷代码是否属于 fork 基线**

对疑似 bugfix 的关键文件/行使用父提交 diff、`git blame` 和基线文件内容确认：缺陷逻辑必须已存在于 `35d0fac...`，或是对该基线既有逻辑直接适用且不依赖基线后的 feature。

- [ ] **步骤 3：写入提交级结论**

允许的最终结论只有：

- `MERGE`：直接修复 fork 基线既有代码，必须移植。
- `EXCLUDE_FEATURE`：feature、重构、性能优化或行为扩展。
- `EXCLUDE_POST_FORK`：修复的代码/功能在 fork 点之后才引入。
- `EXCLUDE_NON_CODE`：仅文档、发布、CI、依赖例行更新或无关测试维护。
- `ALREADY_PRESENT`：RTD 已通过独立修改包含等价修复，需记录证据。

每个结论都在 `review-batches/<range>.md` 中记录依据，并同步更新 TSV。

- [ ] **步骤 4：每批完成性检查**

运行该批 TSV 行的状态查询，预期不存在 `UNREVIEWED`，且 Markdown 中的 SHA 数与批次提交数一致。

### 任务 3：移植适用补丁

- [ ] **步骤 1：先移植或补充能够复现缺陷的上游测试**

路径和标识符按 RTD 品牌映射。若上游提交没有测试，使用其缺陷条件在对应 RTD crate 增加最小回归测试。

- [ ] **步骤 2：运行回归测试确认移植前失败**

使用对应 crate 的 `CLAUDE.md` 指定命令；Rust 单元测试优先：

```bash
RTD_SKIP_SIMTESTS=1 cargo nextest run -p <rtd-crate> <test-filter>
```

记录实际失败输出；若当前 RTD 已通过，转为核验是否 `ALREADY_PRESENT`，不得重复修改。

- [ ] **步骤 3：移植最小修复代码**

仅移植 bugfix 所需 hunk。不要连带移植上游 feature、重构或品牌名；执行 `Sui/SUI/sui -> Rtd/RTD/rtd` 和 `MystenLabs/Mysten/mysten -> LinkUVerse/LinkU/linku` 映射。

- [ ] **步骤 4：运行针对性测试和格式检查**

```bash
RTD_SKIP_SIMTESTS=1 cargo nextest run -p <rtd-crate> <test-filter>
cargo fmt --all -- --check
```

预期：相关测试通过，格式检查无差异。

- [ ] **步骤 5：记录并提交单个补丁**

在 `merged-patches/<upstream-sha>.md` 写明上游 SHA、缺陷、基线关联证据、RTD 文件、品牌映射、测试和本地提交。提交信息包含上游 SHA。

### 任务 4：全范围完成性核验

- [ ] **步骤 1：确认总账没有未审查提交**

运行：

```bash
awk -F '\t' 'NR > 1 && $5 == "UNREVIEWED" { print; found=1 } END { exit found }' \
  doc/mergeUpstreamPatch/upstream-commit-inventory.tsv
```

预期：无输出且退出码为 0。

- [ ] **步骤 2：核对所有 `MERGE` 结论都有实现记录**

逐项确认 TSV 的 RTD 提交存在，`merged-patches/<sha>.md` 存在，代码中不残留不应出现的 Sui 品牌标识符。

- [ ] **步骤 3：运行受影响 crate 测试和全仓 lint**

```bash
RTD_SKIP_SIMTESTS=1 cargo nextest run -p <all-affected-crates>
cargo fmt --all -- --check
cargo xclippy
```

预期：全部通过；任何无法运行或环境失败必须在总记录中明确列出，不能据此宣称完成。
