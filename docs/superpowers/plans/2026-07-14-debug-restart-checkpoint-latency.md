# Debug 重启 checkpoint 延迟修复实现计划

**目标：** debug/release 在持久化重启后均先完成 validator recovery 与 fullnode live catch-up，再对外 ready，并保持秒级以内的本地转账确认。

## 已完成

- [x] 回退部署脚本的 release 优先实验，保持 debug-first。
- [x] 量化原 validator-first 启动造成的冷启动 checkpoint backlog。
- [x] 验证上游并发启动候选；现场否决（debug consensus replay 约 94 秒且 readiness 提前）。
- [x] 对比最新上游 consensus、checkpoint 与 swarm 生命周期，确认不修改共识协议语义。
- [x] profile/metrics 定位 WaitForTransactions、Move execution、digest/BCS 等 debug 热点。
- [x] TDD 添加启动前 validator checkpoint store 读取测试（先红后绿）。
- [x] TDD 添加 embedded validator network readiness 测试（先红后绿）。
- [x] 实现 fullnode-first、CheckpointBuilder 启动 backlog 通知、live catch-up 与内部 readiness gate。
- [x] fresh swarm、单 fullnode 重启、builder channel 关闭和 run-with-range 边界测试；`rtd-swarm` 全部 8 个 lib tests 通过。
- [x] 增加 `[profile.dev] opt-level = 1`，生成保留 debuginfo/debug assertions 的 `target/debug/rtd`。
- [x] debug 同 DB 重启、9 笔转账和稳态 lag 实测通过。
- [x] release build、同 DB 重启和 3 笔转账实测通过。
- [x] TDD 更新外部 readiness helper，在 checkpoint 达标后等待内部 `/health`；INFO target 因 `RUST_LOG=warn` 缺失时可使用权威内部 health 回退；shell tests 与 `bash -n` 通过。
- [x] 独立代码审查并修复 JSON-RPC 两处 readiness 构造参数遗漏；`rtd-json-rpc` 53 个 lib tests 通过。
- [x] 修复 one-shot gate、builder sender 生命周期、inactive validator、epoch hand-off、节点死亡、总超时与 run-with-range 边界。

## 最终验证清单

- [x] 运行 `rtd-core` readiness/transaction recovery/checkpoint 定向测试（8 + 1 + 1 + 1）。
- [x] 运行 `cargo test -p rtd-swarm --lib`（8/8）。
- [x] 运行局部 rustfmt check、`git diff --check`、Cargo metadata 与最终 diff 审计。
- [x] 将本机 dev 环境切回 debug binary；在 `RUST_LOG=warn` 下确认脚本等待内部 health 后返回并保留运行。
