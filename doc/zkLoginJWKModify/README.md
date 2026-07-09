# RTD zkLogin JWK Provider 演化方案

记录日期：2026-06-05

## 结论

RTD 未来如果要支持 zkLogin，不建议删除 JWK 机制。JWK updater 是 zkLogin 的链上认证状态维护路径：

- 节点从认可的 OIDC provider 拉取 JWK。
- validator 通过 consensus 提交 `NewJWKFetched`。
- 链上 `authenticator_state` 被更新。
- zkLogin 交易验签使用链上 active JWK 和 provider/issuer 信息。

当前需要做的不是删掉 JWK updater，而是把 Sui 生态默认 provider 替换为 RTD 自己认可的 provider。

## 当前代码结构

RTD 默认 provider 列表在：

```text
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/rtd/crates/rtd-config/src/node.rs
```

关键函数：

```rust
default_zklogin_oauth_providers()
```

当前列表仍包含 Sui 生态 provider，例如 `FanTV`、`Arden`、`Onefc`、`EveFrontier`、`TestIssuer`、Sui partner AWS tenant 等。

JWK updater 启动逻辑在：

```text
/Users/changzechuan/WenchuanProjects/RTD-Blockchain/rtd/crates/rtd-node/src/lib.rs
```

关键函数：

```rust
RtdNode::start_jwk_updater()
```

启动条件是：

```rust
epoch_store.authenticator_state_enabled()
```

该判断最终依赖 protocol config 的 `enable_jwk_consensus_updates()` 和 genesis/epoch 中是否存在 authenticator state object。

zkLogin provider 的解析、issuer 映射和 JWK URL 不是定义在 RTD 仓库里，而是在 `fastcrypto-zkp` 中：

```text
fastcrypto-zkp/src/bn254/zk_login.rs
```

RTD 当前依赖：

```toml
fastcrypto = { git = "https://github.com/link-u-web3/fastcrypto", rev = "4db0e90c732bbf7420ca20de808b698883148d9c" }
fastcrypto-zkp = { git = "https://github.com/link-u-web3/fastcrypto", rev = "4db0e90c732bbf7420ca20de808b698883148d9c", package = "fastcrypto-zkp" }
```

## 改造路径

### 第一阶段：收窄 RTD 默认 provider 白名单

只改 RTD 仓库。

修改：

```text
crates/rtd-config/src/node.rs
```

目标：

- `Mainnet` 只保留 RTD 正式允许的 provider。
- `Testnet` 只保留 RTD 测试网允许的 provider。
- `Unknown` 用于本地链，不再默认包含 Sui partner provider；可以为空，或只保留 RTD 测试 issuer。
- 移除 `FanTV`、`Arden`、`Onefc`、`EveFrontier`、`TestEveFrontier`、`TestIssuer`、Sui partner AWS tenant 等。

如果 RTD 短期只接受 fastcrypto 已支持的通用 provider，例如 Google、Apple、Microsoft、RTD 自己的 AWS Cognito tenant，那么只改这一步即可。

注意：源码默认值只影响新生成的配置。已经生成的 `~/.rtd/rtd_config/network.yaml` 不会自动改变，需要重新生成 genesis/config，或显式修改配置文件。

### 第二阶段：新增 RTD 自有 OIDC provider

需要改 `link-u-web3/fastcrypto` fork。

修改 `fastcrypto-zkp/src/bn254/zk_login.rs`：

- 在 `OIDCProvider` 增加 RTD provider，例如 `Rtd` 或更具体的 provider 名称。
- 在 `FromStr` 中支持配置字符串，例如 `"Rtd"`。
- 在 `Display` 中输出相同 provider 字符串。
- 在 `get_config()` 中配置 RTD issuer 和 JWK endpoint。
- 在 `from_iss()` 中识别 RTD issuer。

要求 RTD OIDC provider 满足现有 zkLogin 约束：

- JWT `iss` 必须稳定，且和 `from_iss()`、`get_config().iss` 完全一致。
- JWK endpoint 必须返回标准 `{ "keys": [...] }`。
- 当前代码只接受 RSA/RS256 JWK：`kty == "RSA"`，`alg == "RS256"` 或缺省，`e == "AQAB"`。
- 如果要支持 ES256、EdDSA 等，需要改 JWK 解析、证明输入和验证逻辑，成本明显更高。

### 第三阶段：更新 RTD 依赖

fastcrypto fork 改完并提交后，在 RTD 仓库更新：

```text
Cargo.toml
Cargo.lock
```

把 `fastcrypto`、`fastcrypto-zkp` 等依赖的 `rev` 更新到包含 RTD provider 的 commit。

### 第四阶段：更新网络配置和部署流程

本地开发链：

- 重新生成 genesis/config，确保新生成的 `network.yaml` 使用 RTD provider 列表。
- 或在本地部署脚本中增加配置生成/覆盖步骤，但不要直接依赖旧 `~/.rtd/rtd_config/network.yaml`。

测试网/正式网：

- 发布新的 node config 模板。
- validator 和 fullnode 使用一致的 provider 白名单。
- 变更前要确认 provider 列表变更不会造成已有 zkLogin 交易验签语义不一致。

### 第五阶段：验证清单

建议至少覆盖：

- `cargo test -p fastcrypto-zkp`，覆盖 provider parse、`from_iss()`、`fetch_jwks()`。
- `cargo test -p rtd-config` 或新增测试确认默认 provider 列表不含 Sui provider。
- 本地 `rtd start` 日志中只出现 RTD 认可 provider。
- `rtd_getTotalTransactionBlocks` 正常返回。
- zkLogin 正向交易：RTD provider JWT + proof + JWK 上链后可以验签。
- zkLogin 负向交易：Sui provider 或未授权 issuer 的 zkLogin 签名应被拒绝。

## 推荐实施顺序

1. 先收窄 `default_zklogin_oauth_providers()`，解决 RTD 节点默认访问 Sui 外部 provider 的问题。
2. 明确 RTD 自有 issuer、JWK endpoint、JWT 算法和生产/测试 provider 列表。
3. 修改 `link-u-web3/fastcrypto`，新增 RTD provider。
4. 更新 RTD 的 fastcrypto 依赖 rev。
5. 重新生成本地链配置并完成 zkLogin 正/负向验证。

