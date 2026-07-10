# c69bda53：更新受影响的 qs 依赖

- 上游提交：`c69bda536263dce7fcfe18a32611fef2bebd4ed1`
- RTD 提交：`66dd0f93fe41125b152dac744780e291deb73f15`
- 上游标题：`chore(pnpm audit): bump qs >= 6.14.1 (#24814)`

## 基线关联

fork 基线的 trading API 通过 Express 4.21.2 解析到 `qs` 6.13.0，Move Analyzer 与 prettier extension 的 npm locks 分别解析到 6.11.2 和 6.13.1。这些版本处于上游审计记录的 DoS advisory 影响范围；RTD 保留相同版本。

## 移植内容

- trading API 将 Express 更新为 4.22.1，并同步 pnpm lock。
- Move Analyzer 和 prettier extension 运行 lockfile-only `npm update qs`。
- 上游当时解析到已修复的 6.14.1；当前 registry 和新增 advisory 要求更高版本，因此三个 lock 路径均解析到 `qs` 6.15.3，避免移植后仍被当前审计标记。
- trading API 继续使用 `@linku/rtd`，没有带入上游 `@mysten/sui` 品牌。

## 验证

修复前 lockfiles 分别包含 `qs` 6.11.2、6.13.0、6.13.1。修复后：

```text
Move Analyzer package-lock: qs 6.15.3
Prettier package-lock: qs 6.15.3
pnpm lock: qs 6.15.3

npm audit --json: 两个 npm workspace 均无 qs vulnerability
pnpm audit --json: qs advisories 0
```

仓库中仍有与本固定上游提交无关的其他 JavaScript advisories；本记录只声明 `qs` 路径已清除。
