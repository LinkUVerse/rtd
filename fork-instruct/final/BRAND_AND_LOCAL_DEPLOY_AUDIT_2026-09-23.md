# RTD 主链品牌与本地部署审核记录（2026-09-23）

## 范围与结论

本次检查的是 `temp/rtd` 主链 fork 的运行源码、构建配置、CI、文档站点，
以及 `temp/link-u-smart-contract` 的复制工程。主链运行路径没有发现仍把
Sui/Mysten 服务当作 RTD 服务调用的遗漏；旧品牌的剩余命中须按下面的
来源类别处理，不能机械改写为不存在的 LinkUVerse 仓库或 RTD 公网服务。

## 有意保留的上游标识

- `rtd-http` 的原作者版权、许可与作者邮箱，以及原始论文和历史资料的
  Sui 标题，保留真实来源。原论文 PDF 已以 `sui` 文件名保存，避免将其
  错称 RTD 原创文件。
- `tidehunter`、`tokio-msim-fork`、`mystenmark/async-task` 仍使用真实
  上游 Git 来源；若要求依赖源码和来源也完全无旧品牌，须另外 fork 并
  固定可用提交。仅更改 Git URL 不能证明 fork 已存在或能编译。
- 固定 JWK、旧链负向断言和历史协议测试夹具包含旧标识，用于检查边界，
  不是运行网络配置。
- `examples/` 中若干未迁移的第三方示例及假设仓库地址仍作为参考资料
  隔离在主链 pnpm workspace 外，不能宣称示例可构建或已在 RTD 链部署。

## 已清理的发布配置

继承的 AWS 账号和 KMS 测试密钥配置、无法使用的 TS SDK e2e Action、
机械生成的 RTD 公网域名与不存在的官方服务入口已从活跃 CI/站点配置
移除或改为明确的用户配置占位符。文档首页与导航将源码能力和已上线
服务分开表述。`docs/RTD_PUBLICATION_GATE.md` 仍阻止发布未经审核的
文档站点；旧链第三方集成、对外经济政策与正式网络端点须另行审核。
RTD 尚无主网；未来主网不提供 JSON-RPC，本地开发链仍使用该接口。

## 验证证据

- `cargo check --workspace`、`cargo build -p rtd -p rtd-node`、
  `cargo fmt --all -- --check`、`cargo metadata --no-deps --format-version 1`
  和 `git diff --check` 通过。
- 根目录 `pnpm install --frozen-lockfile` 与
  `pnpm turbo lint build test` 通过；Move formatter 60/60 测试通过。
- 文档站点依赖安装、Docusaurus 构建和 372 页 frontmatter 校验通过。
  构建仍有非致命的旧 SVG 读取、生成框架页断锚警告；这些不构成
  公开发布验收。
- 指定 `rtd-conf.yaml` 的实际解析、隔离创世演练与全新本地部署通过。
  配置仅含地址
  `0xc535a846ad8aecf2c353c12b557612f0f1ae3bb09ba7cd2c6c8fa6fa56bf0df9`
  的一笔 `5200000000000000` MIST 分配；默认五组匿名测试账户已删除。
  完整本地 dev Chain ID 为
  `2msPrVdGQLEzZeTM2cMdhdrgEjUNgmaS7bXWbnqSChJx`。
  三份合约均发布、初始化成功，各有 1024 个分片；部署元数据与
  checkpoint 0 的链 ID 一致，三个包对象及初始化交易在链上可查。
  `/health` 返回 HTTP 200；`rtd-indexer-alt`、
  `rtd-indexer-alt-jsonrpc` 和对应守护进程已停止。

复制工程中的创世配置为
`temp/link-u-smart-contract/all-in-one-deploy/localDeploy/rtd-conf.yaml`，
部署元数据为相邻 `deployResults/deployment_metadata.json`。该 Chain ID
仅属于本次本地 dev 链，不是未来 RTD 测试网或主网的固定身份。
