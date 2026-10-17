# 本地创世资金与部署账户校验

已有validator/fullnode运行配置的只读来源检查另见[本地节点配置](./LOCAL-RUNTIME-CONFIGURATION.md)；它复用本页的真实创世验证，不生成或迁移链。

该能力归属正式 `rtd` 的 `rtd-swarm-config`，复用链的地址、密钥、GasCoin、StakedRtd 和 Genesis 类型。部署工程只调用 CLI；不复制地址推导、私钥解析或资金分配逻辑。

```sh
rtd-genesis-inspect /absolute/rtd-conf.yaml
rtd-genesis-inspect /absolute/rtd-conf.yaml --verify-blob /absolute/genesis.blob
rtd-genesis-inspect /absolute/rtd-conf.yaml --verify-signer /absolute/client.yaml
rtd-genesis-inspect /absolute/rtd-conf.yaml --verify-signer /absolute/client.yaml --deploy-address 0x...
```

签名账户检查读取 client.yaml 中显式的 `active_address` 和 `keystore.File`，要求后者为规范绝对路径。显式给出 `--deploy-address` 时还必须与 active_address 一致。现阶段本地部署只支持 File keystore；外部签名器拒绝接入，不回退或初始化钱包。client 文件必须属于当前用户且不可被组/其他用户写入；keystore 必须是当前用户的私有普通文件。符号链接、重复字段、畸形密钥、重复账户、丢失账户和非规范路径均拒绝。

使用正式 RtdKeyPair 解析 Ed25519、Secp256k1、Secp256r1 密钥后推导公有地址。全部配置账户中属于部署地址的 gas_amounts 汇总为十进制字符串；允许多个币和多个同地址配置项，不写死单账户、单币或金额。总额必须大于零。仅作为 validator_token_recipient 接收默认验证者资金而没有显式业务配款，不满足部署准入。命令只检查本地密钥存在，不执行签名或提交交易。

创世反查分别输出：

- `allocations`：YAML 的业务配款，以及同一地址实际拥有的流动 GasCoin。
- `validatorAllocations`：每个验证者的原生默认流动资金、实际锁定质押和接收人。

未设置 validator_token_recipient 时保留原生验证者接收人；显式指定时，验证者流动资金和质押归该地址。检查组合后的币金额多重集，以及每个验证者池的 StakedRtd 接收人和本金。`actualTotalMist` 只计流动币，质押单独报告，不充当可支付交易费用的余额。

所有检查均只读，输入各有大小上限；不生成、导入、复制、导出或删除私钥，不更新 aliases。成功输出只包含公有地址、资金和输入创世摘要；错误输出不回显配置值或密钥。PATH、余额/地址环境变量不会覆盖配置。

相邻部署工程的 `localDeploy/verify_genesis_signer.sh` 校验已安装原生产物摘要并转发；debug/release 入口的 `--check-signer` 只执行该检查。完整 reset/start、链身份绑定与交易恢复仍由对应生命周期及部署工作包提供，不能从该命令成功推导出链服务就绪。
