# Debug 重启 checkpoint 延迟修复设计

## 目标

保留持久化 fullnode、权威 readiness 与 pending transaction WAL 恢复，同时让 `target/debug/rtd` 和 `target/release/rtd` 在停机后重启时尽快恢复到 live checkpoint；只有 validator 的恢复 backlog 已处理且 fullnode 真正追平后，交易相关 RPC 和 `/health` 才允许对外 ready。

## 根因与排除过程

历史修复 `ca5301c164` 为取得 validator 的启动 checkpoint，把上游的全节点并发启动改成了 validator 完全启动后再启动 fullnode。未优化 debug 冷启动时 fullnode 会晚约 70 秒打开 DB，validator 在此期间新增约 260 个 checkpoint；fullnode 的执行速率约 5 checkpoint/s，而 validator 持续生成约 4.1 checkpoint/s，因此只能以很小净速率追赶。

恢复上游并发启动并不能解决 RTD debug 环境：现场 consensus replay 在 fullnode 冷 DB I/O 竞争下约 94 秒才完成，期间 CheckpointBuilder 未启动，但旧 checkpoint target 已满足，节点会提前 ready，首笔交易 60 秒超时。

继续向下定位发现 consensus store 与 RTD consumer 在重启时存在正常恢复窗口。样本中 consensus store 已到 commit `568466`，consumer 处理到 `567090`，需要回放约 1,386 个 commit。未优化 debug 回放相近规模耗时 39.25 秒，并在 CheckpointBuilder 启动后形成集中 checkpoint backlog。CPU 采样热点跨越 `rtd-core` execution、Move VM/package load、object digest、BCS 与 Blake2；单纯调整 checkpoint executor 并发没有稳定收益。

最新上游仍采用相同的 consensus replay/CheckpointBuilder 生命周期以及 swarm 并发启动。为避免引入上游新 feature 或改变多 validator 共识恢复语义，本修复不改 consensus 协议逻辑。

## 最终方案

1. Cargo dev profile 使用 `opt-level = 1`。产物仍位于 `target/debug`，保留 debug assertions 与 debug symbols，但执行、Move VM、序列化和 digest 等跨 crate 热点不再使用完全未优化机器码。仓库现有 simulator profile 也采用同级优化，并记录有 5 倍以上运行提速。
2. `Swarm::launch` 在节点启动前只读校验 validator checkpoint stores，取得持久化 `HighestExecuted` 最大值，提前注入 fullnode readiness。
3. fullnode 先完成 DB 打开，再启动 validator，避免 fullnode 冷 DB I/O 饿死 validator replay，也避免 validator 在 fullnode 尚未打开时持续制造 checkpoint backlog。
4. CheckpointBuilder 在 consensus replay 后完成第一轮 `maybe_build_checkpoints`、清空启动 backlog 时发布 recovery checkpoint。
5. swarm 只等待正常交易 fullnode（`run_with_range = None`）至少追到该 recovery checkpoint，并继续与当前 active validator 的 `HighestExecuted` 对齐；validator-only 和 range-only swarm 保持原启动语义。
6. 对齐后才解除 `/health`、coin read、transaction builder 和 transaction execute 的 network-startup gate。该 gate 由 `Node::spawn` 一次性消费，之后单独重启 fullnode 不会永久卡住。
7. builder channel 的唯一 Sender 随 builder 生命周期结束；epoch 切换时重新订阅新 receiver。恢复等待同时检查节点存活并受 300 秒总超时约束，失败时不会错误解除 readiness。
8. 外部部署 helper 保留原有 checkpoint 防线，并要求 `/health` 为 200，避免脚本比内部权威状态提前报告 ready；当 `RUST_LOG=warn` 导致 INFO target 未记录时，以新二进制内部 `/health` gate 作为权威回退。debug-first 二进制选择保持不变。

## 上游兼容边界

- 没有改变 consensus commit replay target、提议/投票、checkpoint 构造内容、交易执行或对象版本校验。
- CheckpointBuilder 的正常循环与上游一致，只新增一次性的启动进度通知及可检测的 sender 生命周期。
- swarm 启动顺序是 RTD embedded local network 的编排修复；fresh network 集成测试覆盖 fullnode 先启动且最终 ready。
- 已退出当前 validator set 的 configured validator 会被跳过；epoch hand-off 会重新采样 active set，而不是误判恢复成功或永久等待。
- 不引入最新 Sui 的新 RPC store、feature flag 或协议功能。

## 现场验收结果

- 同量级 consensus recovery：未优化 debug 39.25 秒，optimized debug 4.24 秒。
- 带此前实验 backlog 的 debug 重启：CheckpointBuilder 清空 backlog 后，fullnode 0.41 秒内追到 validator；内部 ready 总计约 29 秒。
- 随后正常 debug 重启：内部 ready 8 秒。
- optimized debug 转账共 10 笔，耗时 0.49–0.710 秒，全部 effects success、`confirmedLocalExecution=true`。
- 稳态 15 次采样中 validator/fullnode checkpoint lag 为 0 或 1，`/health` 始终 200。
- 最终 debug 在 `RUST_LOG=warn` 下同 DB 重启约 8 秒 ready；脚本通过内部 health 回退正确放行。
- 最终 release 同 DB 重启约 10 秒 ready；累计 4 笔转账 0.488–0.54 秒，全部成功。
