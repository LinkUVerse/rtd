# 本地 Genesis 中 Validator 的创建与 RTD 分配

本文结合以下内容说明本地开发链中的 `validator-0` 从何而来，以及 Explorer 显示约 `26M RTD` 质押的原因：

- `/Users/changzechuan/WenchuanProjects/RTD-Blockchain/rtd-conf.yaml`
- `/Users/changzechuan/WenchuanProjects/RTD-Blockchain/smartContract-rtd/all-in-one-deploy/localDeploy/deploy_local_all.sh`
- RTD 的 Genesis、Swarm 配置和系统质押源码
- `/Users/changzechuan/.rtd/rtd_config/` 中当前生成的配置

## 结论

1. `validator-0` 不是一键部署脚本显式创建的普通账户。它是在生成本地 Genesis 时，由 `ConfigBuilder` 根据默认 `committee_size = 1` 自动创建的唯一 Validator。
2. Validator 地址 `0xe2708b0a161340feb5777d76c40a5fb2a527d5e11d1e22a22c11af2c57ab06b8` 由 Validator 的 `account-key-pair` 公钥派生。该密钥保存在 `network.yaml` 和对应 Validator YAML 中。
3. `rtd-conf.yaml` 中的 `accounts` 只定义普通账户的 Gas Coin 分配，不定义 Validator 的账户、Gas Coin 或质押。网络构建器会另外为每个 Validator 自动增加：
   - `30,000,000 RTD` 的普通 Gas Coin；
   - `20,000,000 RTD` 的创世自质押。
4. `20M RTD` 不是从 `0xc535...0df9` 的 `5.2M RTD` 中扣除，也不是一键部署脚本中的某个用户后来发起的委托。它是 Genesis Token Distribution Schedule 从固定总供应量中划给 Validator 地址的独立分配。
5. 该链的 Genesis 总供应量不是 `25.2M RTD`，也不是只看创世时生成的可流通 Coin 和 StakedRtd 对象之和。RTD 的固定总供应量是 `10,000,000,000 RTD`，Genesis 必须把全部供应量分配完；未分配给账户和 Validator 的部分进入 `StakeSubsidy` 基金。
6. 按当前配置和一个 Validator 计算，Genesis 分配为：
   - 配置账户 Gas Coin：`5.2M RTD`；
   - Validator Gas Coin：`30M RTD`；
   - Validator 创世自质押：`20M RTD`；
   - Stake Subsidy Fund：`9,944.8M RTD`；
   - 合计：`10,000M RTD = 10B RTD`。
7. Explorer 当前显示约 `26M RTD`，与 `20M RTD` 创世自质押加上约六次 `1M RTD` 质押补贴相符；是否恰好发生六次补贴分配，需要用 distribution counter 和原始池余额核实。Explorer 的 `26M` 是缩写显示，不应理解为精确原始数值。

## `--write-config` 和 `--from-config` 的区别

需要先澄清命令语义。

```bash
rtd genesis --write-config ./rtd-conf.yaml
```

这个命令的作用是生成一份 `GenesisConfig` YAML 并退出。若工作目录已有兼容 keystore，生成配置时会使用其中已有的账户地址；否则会生成本地测试账户配置。无论哪种情况，源码在处理 `write_config` 时保存 YAML 后都会立即 `return`，不会继续创建：

- `genesis.blob`
- `network.yaml`
- Validator 节点配置
- Fullnode 配置

真正用该 YAML 生成本地 Genesis 和节点配置的命令应是类似：

```bash
rtd genesis \
  --from-config /Users/changzechuan/WenchuanProjects/RTD-Blockchain/rtd-conf.yaml \
  --working-dir /Users/changzechuan/.rtd/rtd_config \
  --force
```

因此，准确的数据流是：

```text
rtd genesis --write-config ./rtd-conf.yaml
                    │
                    └── 只生成 GenesisConfig 模板

rtd genesis --from-config ./rtd-conf.yaml
                    │
                    ├── 读取 accounts 和 parameters
                    ├── 自动创建 Validator 配置
                    ├── 补充 Validator Gas Coin 和创世质押
                    ├── 计算 Stake Subsidy Fund 余额
                    └── 生成 genesis.blob、network.yaml 和节点 YAML
```

如果实际自动化流程封装了第二条命令，最终结果仍然遵循上述 `--from-config` 路径。

## 一键部署脚本如何使用已有 Genesis

`deploy_local_all.sh` 默认使用：

```text
RTD_CONFIG_DIR=~/.rtd/rtd_config
FULLNODE_RPC_PORT=9000
```

启动命令为：

```bash
target/debug/rtd start --fullnode-rpc-port 9000
```

脚本启动前只删除 `RTD_CONFIG_DIR` 第一层中名称匹配 `*_db` 的数据库目录，没有删除：

- `genesis.blob`
- `network.yaml`
- `fullnode.yaml`
- Validator YAML
- `rtd.keystore`

所以该脚本的“从 Genesis 启动”是保留原 Genesis 和密钥、删除运行数据库，然后从同一个 `genesis.blob` 恢复链。它不会重新创建另一套 Validator 密钥，也不会改变 `validator-0` 的地址。

这只适用于复用现有配置的一键部署流程。如果重新执行：

```bash
rtd genesis --from-config ./rtd-conf.yaml --force
```

`rtd-conf.yaml` 的 `validator_config_info: ~` 只表示“由构建器自动创建 Validator”，没有固定 Validator 的账户密钥。`ConfigBuilder` 默认使用操作系统随机数生成器，因此新生成的 Validator `account-key-pair` 和地址通常会变化。当前 `0xe270...ab06b8` 不在 `rtd-conf.yaml` 中，也不是由该文件内容确定的可复现地址；它之所以在重复运行一键脚本后保持不变，是因为脚本保留并复用了现有 `network.yaml`、Validator YAML 和 `genesis.blob`。

脚本后续只发布三个 Move Package、创建业务对象和写入 `.env`，没有调用 `request_add_stake`，因此业务合约部署不是 `26M RTD` 质押的来源。

## `validator-0` 在哪里创建

### 1. 默认创建一个 Validator

`rtd-conf.yaml` 中有：

```yaml
validator_config_info: ~
```

这表示没有提供完整的自定义 Validator 配置。Genesis 命令因此走 `ConfigBuilder.committee_size(...)` 分支。

未传 `--committee-size` 时，默认值为 1，所以构建器创建一个 Validator。

### 2. 自动生成 Validator 身份

`ValidatorGenesisConfigBuilder` 为 Validator 生成四类身份密钥：

- `protocol-key-pair`：协议和共识签名；
- `worker-key-pair`：共识 Worker；
- `network-key-pair`：网络身份；
- `account-key-pair`：链上 Validator 账户身份。

链上 `rtdAddress` 来自 `account-key-pair` 的公钥。当前文件中可以看到相同的账户密钥：

- `~/.rtd/rtd_config/network.yaml`
- `~/.rtd/rtd_config/127.0.0.1-55178.yaml`

它派生出的地址是：

```text
0xe2708b0a161340feb5777d76c40a5fb2a527d5e11d1e22a22c11af2c57ab06b8
```

### 3. 自动命名

若 Validator 配置没有设置 `name`，网络构建器使用：

```rust
format!("validator-{i}")
```

唯一 Validator 的索引是 0，因此名称为 `validator-0`。

## `rtd-conf.yaml` 的 5.2M 只代表什么

当前配置为：

```yaml
accounts:
  - address: "0xc535a846ad8aecf2c353c12b557612f0f1ae3bb09ba7cd2c6c8fa6fa56bf0df9"
    gas_amounts:
      - 5200000000000000
```

RTD 使用 9 位小数：

```text
1 RTD = 1,000,000,000 MIST
```

所以该对象金额为：

```text
5,200,000,000,000,000 MIST
÷ 1,000,000,000
= 5,200,000 RTD
```

`GenesisConfig::generate_accounts` 只把 `accounts[].gas_amounts[]` 转换成普通 Token Allocation，并设置：

```text
recipient_address = 0xc535...0df9
staked_with_validator = None
```

因此，这 `5.2M RTD` 是一个属于配置账户的普通 Gas Coin。它与 Validator 的创世资金分配相互独立。

另外，显式填写 `accounts[].address` 只指定 Coin 的接收地址，不会创建或导入该地址的私钥。要花费这笔 `5.2M RTD`，操作者必须另外持有并导入与 `0xc535...0df9` 对应的私钥。

## Validator 的 20M RTD 从哪里来

Validator Genesis 配置的默认质押是：

```rust
20_000_000_000_000_000 MIST
```

换算后为：

```text
20,000,000 RTD
```

网络构建器遍历每个 Validator，并在读取 `rtd-conf.yaml` 所产生的普通账户分配之后，自动追加两笔 Token Allocation。

第一笔是 Validator Gas Coin：

```rust
TokenAllocation {
    recipient_address: validator_address,
    amount_mist: DEFAULT_GAS_AMOUNT,
    staked_with_validator: None,
}
```

其中 `DEFAULT_GAS_AMOUNT` 为：

```text
30,000,000,000,000,000 MIST = 30,000,000 RTD
```

第二笔是 Validator 创世质押：

```rust
TokenAllocation {
    recipient_address: validator_address,
    amount_mist: validator.stake,
    staked_with_validator: Some(validator_address),
}
```

两个地址相同意味着 Validator 自己是质押凭证的所有者，同时也把质押投入自己的 Staking Pool。

这笔资金不是先铸造给 `0xc535...0df9` 再转账，而是在 Genesis 构建期间直接从固定总供应量的尚未分配余额中划出。

Move Genesis 执行 `allocate_tokens` 时看到 `staked_with_validator`，会调用：

```move
validator.request_add_stake_at_genesis(
    allocation_balance,
    recipient_address,
    ctx,
);
```

该函数立即把金额计入 Staking Pool，并创建一个 `StakedRtd` 凭证转给 `recipient_address`。Genesis Builder 的一致性检查还会验证：

- `StakedRtd` 的所有者等于 `recipient_address`；
- principal 等于创世分配金额；
- pool ID 对应指定 Validator；
- activation epoch 等于 0。

因此，20M 的经济含义是 `validator-0` 的创世自质押。

## Genesis 到底发行了多少 RTD

RTD 源码把总供应量固定为：

```text
TOTAL_SUPPLY_RTD  = 10,000,000,000 RTD
MIST_PER_RTD      = 1,000,000,000
TOTAL_SUPPLY_MIST = 10,000,000,000,000,000,000 MIST
```

`TokenDistributionScheduleBuilder` 从 `TOTAL_SUPPLY_MIST` 开始维护一个余额池。每增加一笔账户、Gas 或质押分配，就从余额池扣除；最终剩余金额全部成为 `stake_subsidy_fund_mist`。Schedule 的校验要求：

```text
所有 allocations 金额 + stake_subsidy_fund_mist
= TOTAL_SUPPLY_MIST
```

否则 Genesis 会直接失败。

按当前 `rtd-conf.yaml`、一个 Validator、未添加 Faucet 账户计算：

| Genesis 去向 | MIST | RTD |
| --- | ---: | ---: |
| `0xc535...0df9` 普通 Gas Coin | 5,200,000,000,000,000 | 5,200,000 |
| `validator-0` 普通 Gas Coin | 30,000,000,000,000,000 | 30,000,000 |
| `validator-0` 创世自质押 | 20,000,000,000,000,000 | 20,000,000 |
| Stake Subsidy Fund | 9,944,800,000,000,000,000 | 9,944,800,000 |
| 合计 | 10,000,000,000,000,000,000 | 10,000,000,000 |

所以以下两种理解都不准确：

```text
总供应量 = 5.2M RTD
总供应量 = 5.2M + 20M = 25.2M RTD
```

即使只统计创世时直接归地址所有的普通 Coin 和 StakedRtd，也还漏掉了 Validator 自动获得的 `30M RTD` Gas Coin；该口径应为：

```text
5.2M + 30M + 20M = 55.2M RTD
```

但 `55.2M RTD` 仍然只是创世时直接分配给地址的部分，不是链的总供应量。其余 `9,944.8M RTD` 已经存在于系统 `StakeSubsidy` 基金中，并会按协议规则在后续 Epoch 中逐步进入质押奖励分配。

这不是后续增发。Stake Subsidy 只是把 Genesis 时已经包含在固定 10B 总供应量中的系统余额转移给 Validator 和质押者。

## Explorer 为什么显示约 26M RTD

Explorer 的 Validator 页面读取：

```text
validatorData.stakingPoolRtdBalance
```

该字段对应系统状态中的 `StakingPool.rtd_balance`，表示整个 Validator 质押池的总 RTD，不等于“某一个外部委托人的本金”。它可以包含：

- Validator 的创世自质押；
- 外部账户后续增加的质押；
- 已经加入池中的质押奖励；
- 减去已经生效的退出质押。

当前链的 Genesis 参数为：

```yaml
stake_subsidy_start_epoch: 0
stake_subsidy_initial_distribution_amount: 1000000000000000
stake_subsidy_period_length: 10
stake_subsidy_decrease_rate: 1000
```

初始每次分配金额为：

```text
1,000,000,000,000,000 MIST = 1,000,000 RTD
```

前 10 次分配金额不变，之后按 10% 衰减。只有 Epoch 实际达到完整的 `epoch_duration_ms` 时，系统才提取当次 Stake Subsidy。

本地链只有一个 Validator，它拥有全部投票权。在没有惩罚的情况下，绝大部分补贴会分配到它的质押池。如果 Epoch 0 到 Epoch 6 的六次切换都满足完整 `epoch_duration_ms`，并且六次都实际调用了 Stake Subsidy 分配，近似关系为：

```text
20M 创世自质押
+ 6 × 1M Epoch 质押补贴
+ 少量可能的交易费/存储奖励
≈ 26M RTD
```

仅看到当前 `epoch = 6` 不能严格证明补贴分配计数一定为 6，因为不足完整 Epoch 时长的切换不会提取补贴。精确核对应同时读取：

- 系统状态中的 Stake Subsidy distribution counter；
- `stakingPoolRtdBalance` 的原始 MIST 值；
- 各 Epoch 的 Validator reward events。

默认 Validator commission 为 200 basis points，即 2%。每个 Epoch 的奖励会拆成：

- Validator 佣金：生成新的 `StakedRtd` 并转给 Validator 地址；
- 其余质押者奖励：进入 Staking Pool 的 rewards pool 并自动复利。

由于创世质押凭证本身也属于 `validator-0`，在没有其他委托人的情况下，两部分的最终经济权益都属于 `validator-0`。

Explorer 会将 `stakingPoolRtdBalance` 转成 JavaScript `Number`，再按 RTD 的 9 位精度和缩写规则显示，因此页面上的 `26M` 只适合表达数量级。需要精确金额时，应读取 RPC 返回的原始十进制字符串，并使用 bigint 或任意精度十进制处理。

## 关键源码位置

- `crates/rtd/src/rtd_commands.rs`
  - `--write-config` 与 `--from-config` 的命令定义；
  - 默认 `committee_size = 1`；
  - 读取自定义 GenesisConfig 并生成本地配置。
- `crates/rtd-swarm-config/src/genesis_config.rs`
  - Validator 密钥和地址生成；
  - 默认 Validator stake 为 `20M RTD`；
  - `DEFAULT_GAS_AMOUNT = 30M RTD`；
  - `accounts[].gas_amounts[]` 到普通分配的转换。
- `crates/rtd-swarm-config/src/network_config_builder.rs`
  - 为每个 Validator 自动添加 Gas Coin 和自质押；
  - 默认命名 `validator-{i}`。
- `crates/rtd-config/src/genesis.rs`
  - 固定总供应量校验；
  - Token Distribution Schedule；
  - 默认每 Epoch `1M RTD` Stake Subsidy。
- `crates/rtd-framework/packages/rtd-system/sources/genesis.move`
  - Genesis 中创建 Validator；
  - 将普通分配转成 Coin，将质押分配转成 StakedRtd。
- `crates/rtd-framework/packages/rtd-system/sources/validator.move`
  - 创世质押、自质押凭证所有权和奖励入池。
- `crates/rtd-framework/packages/rtd-system/sources/stake_subsidy.move`
  - Stake Subsidy 的逐 Epoch 提取和衰减。
- `crates/rtd-framework/packages/rtd-system/sources/validator_set.move`
  - 奖励按投票权分配、Validator 佣金和质押池自动复利。

## 最终回答

`validator-0` 的初始 `20M RTD` 来自 Genesis Builder 对每个 Validator 自动追加的创世质押分配。它不属于 `rtd-conf.yaml` 中 `0xc535...0df9` 的 `5.2M RTD`，也不会从该账户余额中扣除。

当前链的 Genesis 固定总供应量是 `10B RTD`。其中 `55.2M RTD` 在创世时直接以普通 Coin 或 StakedRtd 形式分配给地址，剩余 `9,944.8M RTD` 进入 Stake Subsidy Fund。Explorer 后来显示的约 `26M RTD` 与 `20M` 创世自质押加上约 `6M` 已释放补贴相符；精确拆分需进一步读取 distribution counter、原始池余额和 Epoch 奖励事件。
