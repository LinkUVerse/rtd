# 2026-09-23 上游刷新记录

本次主链基于上游 `MystenLabs/sui` 提交
`ea36d0cbc33376eb33b8083c1e27cb3e7bb9f368`。RTD 从全新 genesis 启动，
链上模块、原生币、JSON-RPC 方法、私钥文本格式和 protobuf 服务名称均按 RTD
规则重新命名。此文件记录旧 `COMPLETE_FORK_GUIDE.md` 对当前上游缺少的步骤。

## 已确认的依赖版本

| 用途 | 本次使用方式 |
| --- | --- |
| Rust SDK | `LinkUVerse/rtd-rust-sdk`，固定 `1aed5776c9d957f08d06bde07664993d70825c93` |
| fastcrypto、linku-sim、anemo | 使用 LinkUVerse 仓库中与上游相同的固定提交 |
| HTTP 服务器 | `crates/rtd-http`，基于 `sui-http` 0.3.1 的完整源码及测试，本地包名为 `rtd-http` |

`rtd-http` 不能从旧 RTD 仓库的 0.0.0 版本整包复制：新上游依赖 0.3.1
新增的回调、gRPC 超时中间件及连接防护。对新克隆的源码，执行完品牌脚本
phase01–phase12 后，phase12 会调用 `vendor-rtd-http.py`：它校验原始
crates.io 发布包的 SHA-256，复制完整源代码，改包名并设置本地 path 依赖。
原作者与 Apache-2.0 许可保留在本地 crate 的版权信息和 README 中。

`tidehunter`、`tokio-msim-fork`、`mystenmark/async-task` 当前没有可访问且
含本次固定提交的 LinkUVerse fork。它们的 Cargo URL 保留真实上游地址；
仅替换锁文件 URL 会使构建失败。若要求**依赖源码与来源也完全无旧品牌**，
需另建真实可访问的 fork，修改 `linku-sim` 的传递依赖，再固定新提交并重新
生成锁文件。不能凭字符串替换虚构仓库。

## 协议字面量

普通单词（如 `suitable`、`suite`）不能做无边界的 `sui` 子串替换；
`brand-replace.py` 先改品牌词与源码标识。`brand-fix-protocol-literals.py`
随后处理单词内的协议标识：`suix_*` → `rtdx_*`、
`suiprivkey` → `rtdprivkey`、`SuinsRegistration` → `RtdnsRegistration`。
Bech32 示例私钥必须重新计算校验和，不能只替换前缀；脚本有自检入口：

```bash
python3 fork-instruct/final/brand-fix-protocol-literals.py --self-test
```

Rust 主链的 `rtdx_*` 必须与 TS SDK 的 JSON-RPC client 方法匹配；
Rust/TS 两边的私钥编码前缀也必须一致。生成当前 OpenRPC 文档后，
逐项核对方法清单。

本次对照 OpenRPC 后，TS SDK 静态 JSON-RPC 方法 39 项均有服务端路由，
`rpc.discover` 由服务器另行注册。`crates/rtd-json-rpc-api/src/extended.rs`
虽然声明 `getEpochs`、`getCurrentEpoch`，当前服务端未注册 ExtendedApi；
不要把 trait 声明当作可用接口。已从 TS SDK 删除这两项及另外五项没有
路由的 analytics 方法，后续若服务端正式实现，应同时更新 OpenRPC 和 SDK。

## 生成产物

新上游含旧链字节码和 protobuf 二进制描述符。编译通过不会刷新它们。

```bash
UPDATE=1 cargo test -p rtd-framework --test build-system-packages
cargo test -p rtd-framework --test build-system-packages

rustup toolchain install nightly --profile minimal
cargo +nightly -Zscript crates/rtd-fork/build-protos
cargo +nightly -Zscript crates/rtd-rpc-store/codegen.rs
cargo +nightly -Zscript crates/rtd-consistent-store/codegen.rs
cargo test -p rtd-rpc-cursor --test bootstrap
cargo test -p rtd-indexer-alt-consistent-api --test bootstrap
python3 fork-instruct/final/verify-proto-descriptors.py .
```

两个 `codegen.rs` 的 `proto-build` 固定到上述 Rust SDK 提交，避免从可能
较旧的 `master` 分支生成不一致的 API。生成后需要反序列化检查
FileDescriptorSet 的 package 与文件路径均为 `rtd.*` / `rtd/`。
`git diff --exit-code` 不会检查未跟踪的新目录；首次生成时须额外列出文件并
检查二进制内容，提交后重跑 bootstrap 测试。

## 历史字节码快照

上游 `rtd-framework-snapshot/bytecode_snapshot` 的 3–137 版本是旧链
历史产物，当前源码只能生成 `ProtocolVersion::MAX`（此提交为 138）。
绝不能只清空快照目录而保留 `manifest.json`；加载器及 genesis builder
仍会按 manifest 查找，且 genesis builder 存在失败后回退最新框架的行为。

在当前 Move 包字节码与 protobuf 产物经过审核并形成干净提交后，生成
本链 138 快照：

```bash
GIT_REVISION="$(git rev-parse HEAD)" cargo run -p rtd-framework-snapshot --bin rtd-framework-snapshot
python3 fork-instruct/final/finalize-rtd-snapshots.py
```

本次新链仅支持从版本 138 建立 genesis。清理脚本会先确认 138 的五个包
与 manifest 一致、BCS 中无旧品牌，然后同时删除 3–137 的旧链 BCS 和
manifest 条目。加载器对旧版本、缺失版本和超过当前最大版本的请求明确
失败；genesis builder 不再在快照缺失时回退到当前内置框架。还需检查基准测试中假定存在
`<MAX` 快照的断言。运行兼容性测试与本地 genesis 烟测，验证新币类型
为 `Coin<rtd::rtd::RTD>`。历史 light-client 二进制证明已换为 Simulacrum
运行时生成的 RTD 测试数据；Move decompiler 的测试字节码用等长品牌
替换并重新生成对应快照，`brand-fix-move-decompiler-bytecode.py` 可重复执行。

## 新链协议配置

此 fork 的版本 138 是全新 RTD genesis，未有已上线的 RTD 版本 138。
上游主网与测试网预置的第三方稳定币 gasless 白名单和 package linkage
amendment 表只对应上游链的 package ID，不能在 RTD 上照用。保留 gasless
机制，但新链默认白名单为空；移除上游专用 amendment JSON。若将来发行
RTD 链上的稳定币，须使用本链 package ID 经新协议版本增加白名单。
`rtd-protocol-config` 的旧版本快照仅用于源码回归测试，不是可启动的
RTD 历史版本。

RTD 的正式 mainnet/testnet genesis 尚未提供，因而 `rtd-types` 的两个
固定 Chain ID 当前为 `None`。已知的上游 genesis digest 即使设置了
测试用链类型 override 也只能识别为 Unknown。需要在各网络真实 genesis
确定并独立验证后填入其完整 digest；在此之前，桥接等依赖固定正式网络
身份的功能会显式报未配置，客户端不应缓存虚构的默认 Chain ID。
源码/测试中的短 ID 示例改用合成值；请勿将其当作 RTD 网络标识。

## 文档发布门槛

`docs/content` 仍包含上游独有的扫描器、钱包、SDK、游戏设备及历史
交易链接。机械替换正文中的品牌并不会让 `suiscan.xyz`、`suivision.xyz`
等站点支持 RTD。尤其不要把这套文档直接作为 RTD 正式站点发布；
需先按实际已部署的 RTD 服务建立文档发布清单，裁掉旧链第三方页面和
入站导航、修正外链，再通过 Docusaurus 的断链检查。`doc/` 中保留的
历史 fork/修复资料是内部参考，不代表当前网络的可用服务。

## 旧 RTD 运行修复的移植范围

旧 `doc/devRestartBugs` 的持久 fullnode 配置路径已被新上游覆盖。
本次仍需检查并移植：同链可打开 DB 候选选择、pending WAL 可重试保留及
恢复、fullnode 启动就绪门槛、checkpoint 等待重试与后台任务 fail-stop、
consensus 持久背压、回放期间的空批次/队列性能。新上游使用 Tidehunter、
embedded RPC store 和新版 transaction driver，不能覆盖整份旧文件。

旧 `doc/fullNodePerfomance/P0-fullnode-runtime-stall-fix-plan.md` 仍是待审
计划，旧源码未实施。其多线程 swarm/fullnode 调度与同步 checkpoint 写入
建议属于新增优化，不能列为已经保留的旧修复。
