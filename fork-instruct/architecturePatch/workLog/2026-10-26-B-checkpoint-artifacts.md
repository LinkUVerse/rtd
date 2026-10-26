# 工作日志 B：published checkpoint 官方产物

日期：2026-10-26

## 改动

- 新增 `rtd-archive-service/crates/rtd-archive-publication/src/artifacts.rs`
  - `parse_epochs`：只接受严格递增、< 2^53、受字节/条目预算约束的 `u64` 数组。
  - `visible_epochs`：只取 ≤ 本轮已认证 `through` 的已结束 epoch 前缀。
  - `require_append_only`：published `epochs.json` 只能追加，禁止回退/替换。
  - `PublisherStorage::publish_epochs`：固定版本写入、读回、latest 复核；相等时复用旧版本。
  - `PublisherStorage::publish_checkpoint_bcs`：stage 有 `.chk` 时复制固定版本，校验 SHA-256、published 内容和 latest。
  - `artifact_exists`：区分“不存在”和存储错误。
- `publisher_storage.rs`：发布内部字段改为 `pub(crate)`，供同一 crate 的 artifact 逻辑访问；不改变外部 API。
- `publisher.rs`
  - 每轮在 checkpoint 波次之后、`new_versions` 读回之前发布 `epochs.json`；
  - 对本轮新增 checkpoint 用首/中/尾有界样本预检 `<seq>.chk`：全无视为 BCS 管线未启用；样本不一致则冲突；样本均存在才逐个复制，复制中缺失同样失败关闭；
  - 所有 artifact 进入 `new_versions`，先过 source/replica 固定版本读回，再写 committed marker。
- `modules/checkpoint-blob/README.md`：声明三条标准管线与 `.chk` 全有/全无策略。
- `modules/indexer-alt/patches/README.md`：增加候选补丁不是当前运行构建的警示。

## 验证

```text
$ cargo check -p rtd-archive-publication
Finished `dev` profile

$ cargo test -p rtd-archive-publication --lib
running 42 tests
test result: ok. 41 passed; 0 failed; 1 ignored
```

覆盖：epochs 格式/边界、可见前缀、追加语义；原有发布、S3、visibility、bucket、schema 单测全部通过。

## 未完成/边界

- 未在真实 publisher + 隔离 S3 上执行完整发布；`epochs.json`/`.chk` 的实际复制、副本读回和恢复需要现场运行验证。
- 正常 indexer 仍通过 publication adapter 读取 `.binpb.zst`；`epochs.json`/`.chk` 的恢复消费者使用 published 桶只读 recovery 身份。adapter 的 artifact 白名单扩展未在本轮实现。
- `.chk` 采用“全有或全无”策略：若 stage 只有部分 `.chk`，发布冻结，需修 stage 或显式关闭 BCS 管线。
- committed marker 之前会完成 artifact 读回；但 published 直读仍可能看到尚未提交的对象，这和现有 checkpoint 复制语义一致，生产直读限制仍由发布准入承担。

## 追溯

- 设计：`../fork-instruct/architecturePatch/02-checkpoint-artifacts.md`
- 状态：`../fork-instruct/architecturePatch/08-implementation-status.md`
