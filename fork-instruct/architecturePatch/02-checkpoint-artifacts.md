# 专题 B：checkpoint 发布产物完整性

## 1. 官方对象布局

官方 checkpoint blob 写入器在对象存储中产生：

- `<sequence>.binpb.zst`：压缩 protobuf checkpoint；
- `<sequence>.chk`：checkpoint BCS（由 `checkpoint_bcs` 管线产生）；
- `epochs.json`：每个已结束纪元的最后一个 checkpoint 序号数组；
- 框架水位/链身份元数据。

官方读取器在恢复和 epoch 边界处理时会读取 `epochs.json`；`checkpoint_bcs` 管线被启用时，`.chk` 也是正式产物。

## 2. 当前缺陷

RTD publisher 当前只复制：

- `<sequence>.binpb.zst`
- `_metadata/chain_id/checkpoint_blob`
- `_metadata/watermarks/checkpoint_blob.json`
- `_metadata/watermark/checkpoint_blob.json`
- `_metadata/publication/committed.json`

没有复制 `epochs.json`，也没有复制 `.chk`。published 审计白名单不接受这两类键，consumer adapter 也明确拒绝 `epochs.json`。

影响：

- 官方 consistent-store formal snapshot restore 无法从 published 桶读取 `epochs.json`；
- 依赖 BCS checkpoint 的恢复/工具链无法从 published 桶读取 `.chk`；
- RTD 只能从 checkpoint 0 冷重放，恢复时间和对官方快照的兼容性不达标；
- 文档声称“官方对象布局”与实际产物不一致。

另：`modules/checkpoint-blob/README.md` 声称只运行 `checkpoint_blob`、`epochs` 两条管线，但运行时测试期望 `checkpoint_blob`、`checkpoint_bcs`、`epochs` 三条，需要统一。

## 3. 设计

### 3.1 `epochs.json`

**来源**：读取 stage 最新 `epochs.json`，解析为严格递增的 `u64` 数组，过滤掉 > `through` 的未来项。stage 缺失时视为尚无可发布的 epoch 边界，不创建空文件。

**写入规则**：

1. 读取 published 最新 `epochs.json`（若存在），解析旧数组；
2. 新数组必须是旧数组的前缀扩展（允许相等，禁止回退/替换）；
3. 相等时复用旧固定版本，不产生新版本；
4. 否则用 fence writer 写入新版本，读回固定版/摘要，验证 latest，再纳入 replica 读回；
5. 任何格式错误、乱序、未来项被截断但 published 已有更高项、旧数组不是新数组前缀、版本漂移，均失败关闭。

**原子性**：在 checkpoint 波次提交和水位/committed marker 之前完成；committed marker 只在本轮所有 artifact 读回通过后写入。

**兼容性**：内容格式与官方 `epochs.json` 一致（JSON 数组，无额外包装）。RTD 不向官方 reader 暴露额外字段。

### 3.2 `<sequence>.chk`

**策略**：若 stage 存在对应 `.chk`，复制到 published，并与 stage 固定版本内容做 SHA-256 对照；若 stage 不存在，视为该管线未启用，跳过并记录（不把缺失当作成功）。

**一致性要求**：

- 先用有界样本（首/中/尾）判断该范围是否启用 BCS 管线；样本不一致时拒绝并冻结该轮；
- 选定启用后逐个复制，任何对象在复制时消失都失败关闭，避免“半套官方产物”；
- published 已有 `.chk` 时，必须与 stage 同 key 内容一致，否则冲突；
- 复制后固定版本读回、replica 读回、latest 复核。

### 3.3 消费路径

- 正常 indexer/GraphQL 继续通过 publication adapter 读取 `.binpb.zst`；本次不改其热路径。
- 官方快照恢复等需要 `epochs.json`/`.chk` 的消费者使用 **只读 recovery 身份直接读取 published 桶**（或后续为 adapter 增加显式 artifact 白名单）。文档必须写清这是恢复路径，不是普通 ingestion 旁路。
- 不在 published 桶中写入任何 RTD 私有的 artifact 索引对象；版本信息由 journal/发布回执承载。

### 3.4 审计

- 更新离线发布审计的白名单：允许 `epochs.json`、`.chk` 和现有元数据键；
- 审计检查：epochs 单调、最大项 ≤ committed 高度、`.chk` 与 checkpoint 一一对应（管线启用时）、固定版本与摘要；
- 不在每个 follow tick 做全桶 LIST。

## 4. 验收

1. 新发布一轮后，published 桶包含正确 `epochs.json`，内容与 stage 截止 `through` 的已结束 epoch 一致。
2. `epochs.json` 从旧到新只追加，不出现回退/重写。
3. stage 启用 BCS 时，published 对每个新 checkpoint 都有 `.chk`；禁用时文档/配置/测试一致。
4. 官方 consistent-store formal snapshot 恢复在隔离环境可从 published 桶成功启动（或明确记录不支持项）。
5. 部分 `.chk`、epochs 冲突、版本漂移均能被负例测试拒绝。
6. 发布延迟与对象写入预算有基线，不能因为 artifact 复制阻塞正常 checkpoint 发布。

## 5. 回滚

- 新增 artifact 复制失败时，不推进 committed marker；保留已写入的固定版本，下一轮幂等重试。
- 若需要临时关闭 BCS artifact，必须在配置中显式关闭对应管线，不能仅停止复制。
