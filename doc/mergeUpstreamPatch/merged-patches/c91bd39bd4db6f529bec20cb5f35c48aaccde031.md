# c91bd39b：同步验证块后更新 peer round tracker

- 上游提交：`c91bd39bd4db6f529bec20cb5f35c48aaccde031`
- RTD 提交：`4700771ba46c9ffb9de2b66694a12cc8cc921f21`
- RTD 测试补充提交：`6ae8cf0ce9f236dec93a1c0a99b32b39e42a62c2`

## 基线关联

fork 基线只在实时 push 和本地提议路径更新 `PeerRoundTracker`。block sync 与 commit sync 拉取的块即使验证成功也绕过 tracker，恢复节点会保留陈旧 accepted quorum round、虚高 propagation delay，并可能停止提案。

## 移植与验证

把 authority node 已有的同一个 tracker 注入 `Synchronizer` 与 `CommitSyncer`，在块验证成功后用空 `excluded_ancestors` 更新接受轮次。回归测试用合法的 round 61-91 peer blocks 引用 round 60 ancestor，构造三方接受证据并断言 propagation delay 从 60 降为 0。

```text
cargo test -p consensus-core synchronizer --lib
7 passed; 0 failed

cargo test -p consensus-core commit_syncer_start_and_pause_scheduling --lib
1 passed; 0 failed

cargo check -p consensus-core
Finished successfully
```
