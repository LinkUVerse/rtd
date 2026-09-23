# 2026-09-23 上游刷新记录

本次主链基于上游 `MystenLabs/sui` 提交
`ea36d0cbc33376eb33b8083c1e27cb3e7bb9f368`。RTD 从全新 genesis 启动，
链上模块、原生币、JSON-RPC 方法、私钥文本格式和 protobuf 服务名称均按 RTD
规则重新命名。此文件记录旧 `COMPLETE_FORK_GUIDE.md` 对当前上游缺少的步骤。

## 已确认的依赖版本

| 用途 | 本次使用方式 |
| --- | --- |
| Rust SDK | `LinkUVerse/rtd-rust-sdk`，固定 `fd95c4566e88cda3c4e5e590deda4e6aff864700` |
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
品牌更名后必须运行 `cargo fmt --all`：Rust import 排序会因 crate 名变化而
重新分组，源码即使能编译也未必通过 `.github/workflows/rust.yml` 的
`cargo fmt --check` 门禁。品牌脚本 phase14 已加入这个步骤。
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
本轮重新生成 `crates/rtd-open-rpc/spec/openrpc.json` 和测试快照，确认当前
服务端规范有 56 个方法、154 个 schema，协议版本为 138；
`generate-spec` 测试通过。生成规范原先继承的四处未部署文档域名已改成
类型本身的说明，不再对外给出虚构链接。
今后刷新上游时，应检查 `rtd-json-rpc-api`、`rtd-json-rpc-types` 与
`rtd-indexer-alt-graphql` 的公开接口注释，随后同步 OpenRPC 的
`spec/openrpc.json`、测试快照和 GraphQL 的两个 SDL/快照；用
`cargo test -p rtd-open-rpc --test generate-spec` 与
`cargo test -p rtd-indexer-alt-graphql --lib test_schema_sdl_export`
验证生成文件一致，不能只更新静态 JSON 或 SDL。

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

本次首次提交采用两阶段绑定：先生成五个 138 包并以临时 revision 写入
manifest，提交完整 fork 源码为 `7098077d8d82e7b6f145faf7b694219e7b99f566`，
再把 manifest 的 `git_revision` 改成这个 40 位源码提交并单独提交。
兼容性测试从 `https://github.com/LinkUVerse/rtd.git` 获取该 revision；
在尚未推送的本地审核阶段，用 Git 的一次性 URL rewrite 指向当前工作树，
并确保 `git cat-file -e <revision>^{commit}` 能在本地解析。不要把临时
revision 留作最终版本。

本次新链仅支持从版本 138 建立 genesis。清理脚本会先确认 138 的五个包
与 manifest 一致、BCS 中无旧品牌，然后同时删除 3–137 的旧链 BCS 和
manifest 条目。加载器对旧版本、缺失版本和超过当前最大版本的请求明确
失败；genesis builder 不再在快照缺失时回退到当前内置框架。兼容性测试中
对旧版本快照的假设也已调整。快照兼容性测试 5 项通过，本地 genesis 测试确认
原生币类型为 `Coin<rtd::rtd::RTD>`。历史 light-client 二进制证明已换为 Simulacrum
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

## 未部署网络与测试夹具

从上游机械改写出的 `*.rtd.io` 不等于已经部署的 RTD 端点。
`scripts/compatibility/fullnode-sync.sh` 现在要求预先提供经过核对的
`genesis.blob` 与 `fullnode.yaml`，不再自动下载所谓正式网络 genesis 或
附加上游 seed peer ID。未显式传 `--end-epoch` 时，
`monitor_synced.py` 还要求 `RTD_REFERENCE_RPC_URL` 指向已核对的 RTD 节点。
协议兼容检查同样需要 `RTD_PROMETHEUS_URL`；不能把旧监控域名的机械改写
结果当作正式服务。`rtd-config/src/test_gateway.yml` 中的旧验证者域名已经
替换为保留的 `.example.invalid` 测试地址，此文件仅用于解析样例。

Rust SDK 的 `build_devnet`、`build_testnet`、`build_mainnet` 现在分别要求
`RTD_DEVNET_RPC_URL`、`RTD_TESTNET_RPC_URL`、`RTD_MAINNET_RPC_URL`，
或由调用方使用 `build(url)`；新钱包环境不再填假公网地址。`rtd-data-store`
和 `rtd-fork` 的命名公网网络在无部署配置时明确报错，自定义 URL 可用。
远端 cluster-test 也须显式传 fullnode/faucet 地址。fullnode 的交易 KV
HTTP 后备默认关闭，使用本地 DB；正式 archive 部署后再填真实地址。
light-client 的主网/测试网 YAML 只是空模板，不是可运行网络配置。

`rtd-tool` 正式快照下载要求提供对应 bucket 和
`RTD_CHECKPOINT_INGESTION_URL`，不再假定上游归档已搬到 RTD 域名。
`http_kv_tool` 的 `--base-url`、日志拉取工具的 `GRAFANA_LOGS_URL` 也须
显式提供。MVR 解析需要对应网络的 `RTD_MVR_*_URL`；zkLogin 测试工具需要
真实 salt、prover、faucet 和 fullnode URL，不能把上游对象或密钥服务
机械换域名后视为已部署。

`rtd-oracle` 测试中的 `SUIUSD` 字符串已变为 `RTDUSD`；OpenRPC 和类型解析
示例中的旧链社群包名只作合成示例，不得指向旧链包。可选第三方示例应在
对应服务确实部署到 RTD 后才配置真实对象 ID、RPC 或浏览器 URL。
`examples/RTD_FORK_STATUS.md` 记录各示例的部署前置条件。继承的 TS 示例
仍有 `@linku/rtd` / `@linku/dapp-kit` 旧包 API，而此次独立 TS SDK 实际
包名为 `rtd-typescript` / `rtd-dapp-kit-react`；这些非主链示例需在后续
移植 API 和安装包后才可声明完整构建通过。本轮 27 个 TS/JS 示例文件的
语法检查与 oracle 配置的类型检查通过；继承的 `@linku/rtd@1.18.0` 无法
从 npm 获取，离线环境也缺少 React Query Devtools，故没有完整 TS 示例
构建通过的证据。

## 文档发布门槛

`docs/content` 仍包含上游独有的扫描器、钱包、SDK、游戏设备及历史
交易链接。机械替换正文中的品牌并不会让 `suiscan.xyz`、`suivision.xyz`
等站点支持 RTD。尤其不要把这套文档直接作为 RTD 正式站点发布；
需先按实际已部署的 RTD 服务建立文档发布清单，裁掉旧链第三方页面和
入站导航、修正外链，再通过 Docusaurus 的断链检查。本轮未把文档站点的
构建、链接可达性或外部服务接入记为通过。`doc/` 中保留的
历史 fork/修复资料是内部参考，不代表当前网络的可用服务。
根目录 `README.md` 已去掉虚构的官方服务与性能宣称，`SECURITY.md`
已移除没有依据的赏金金额、第三方报告入口和联系方式；发布生产网络前
须由维护者配置并核实正式的安全报告渠道与政策。
每次重新 fork 均须审查 `README.md`、`SECURITY.md`、
`CONTRIBUTING.md` 和 `CODE_OF_CONDUCT.MD` 的外链与政策，不能只做
品牌字符串替换后沿用上游承诺。

## 旧 RTD 运行修复的移植范围

旧 `doc/devRestartBugs` 的持久 fullnode 配置路径已被新上游覆盖。
已按新版架构移植：同链且可打开的旧 fullnode DB 候选选择；交易提交未完成
时保留 pending WAL、释放内存中的 inflight 标记以允许同 digest 重试，并在
重启时恢复；fullnode 同步追赶期间的启动就绪门槛与 RPC 拒绝；checkpoint
builder 的恢复等待、失败重试和后台任务 fail-stop；consensus 的持久回放窗口
背压、恢复后重新触发本地工作，以及回放时的空批次和队列处理。

移植采用当前 Tidehunter、embedded RPC store 和新版 transaction driver
接口。RocksDB 路径会只读验证 checkpoint 与对象库确实可打开，Tidehunter
目前没有对应的只读打开能力；其 swarm 启动目标改从 checkpoint builder
恢复通知获取，不能把这称为旧 DB 候选检查的等价实现。已有针对 WAL、
背压、就绪和快照的单元及集成测试；真实进程 SIGKILL 后的持久恢复和
强制 checkpoint builder panic 后的端到端故障演练仍需单独执行。

旧 `doc/fullNodePerfomance/P0-fullnode-runtime-stall-fix-plan.md` 仍是待审
计划，旧源码未实施。其多线程 swarm/fullnode 调度与同步 checkpoint 写入
建议属于新增优化，不能列为已经保留的旧修复。

## 本轮验证记录

在 Rust SDK 固定到 `fd95c4566e88cda3c4e5e590deda4e6aff864700` 后，
`cargo check --workspace`、`cargo build -p rtd -p rtd-node`、
`cargo fmt --all -- --check` 和 `git diff --check` 均通过；`rtd --version`
可执行。两个独立 `codegen.rs` 用 nightly 重新生成后没有产物差异，
protobuf 描述符检查通过。

OpenRPC `generate-spec` 1/1、GraphQL SDL 导出默认与 staging 配置各
1/1、框架快照兼容性 5/5 通过。未部署网络拒绝测试 2/2、SDK Chain ID
测试 5/5、核心链标识测试 6/6、序列化测试 2/2、Swarm 库测试 5/5
通过。`rtd-core` 的 node readiness 测试 11/11、light-client 测试
28/28，以及共识相关的持久化/活性目标测试也已通过。

上述结果是本地构建及目标测试。正式网络 genesis 与 Chain ID、外部服务、
真实进程强杀恢复、强制 checkpoint builder panic 的端到端演练，以及
第三方 TS 示例的完整构建均不在本轮通过范围内。
