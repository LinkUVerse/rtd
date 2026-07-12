# RTD 固定总供应量的源码位置与 Genesis 约束

本文说明 RTD 固定总供应量 `10B RTD` 在哪里定义、Genesis 如何铸造和校验，以及修改总供应量时需要同步调整哪些位置。

## 结论

RTD 的 `10B` 固定总供应量不在 `rtd-conf.yaml` 中配置，而是同时硬编码在 Move 和 Rust 源码中：

- Move 侧在 Genesis 中实际铸造 `10B RTD`；
- Rust 侧以同样的总量构建并校验 Token Distribution Schedule；
- 两侧的 `TOTAL_SUPPLY_MIST` 必须一致；
- `rtd-conf.yaml` 只能决定固定总量如何分配，不能改变固定总量本身；
- 已生成的 `genesis.blob` 不会因修改源码而变化，调整总量后必须重新构建 Framework 并重新生成 Genesis。

## 单位换算

RTD 使用 9 位小数：

```text
1 RTD = 1,000,000,000 MIST
```

固定总量为：

```text
10,000,000,000 RTD
× 1,000,000,000 MIST/RTD
= 10,000,000,000,000,000,000 MIST
```

## Move 侧：实际铸造总量

文件：

```text
crates/rtd-framework/packages/rtd-framework/sources/rtd.move
```

总量常量为：

```move
const MIST_PER_RTD: u64 = 1_000_000_000;
const TOTAL_SUPPLY_RTD: u64 = 10_000_000_000;
const TOTAL_SUPPLY_MIST: u64 = 10_000_000_000_000_000_000;
```

Genesis 调用 `rtd::rtd::new` 时，真正执行铸造的是：

```move
let total_rtd = supply.increase_supply(TOTAL_SUPPLY_MIST);
```

因此，Move 侧 `TOTAL_SUPPLY_MIST` 是链上实际铸造量的直接来源。

`new` 同时限制：

- 调用者必须是系统地址 `@0x0`；
- 当前必须是 Epoch 0；
- 铸造完成后销毁 `Supply` 能力。

这使正常链运行期间无法再次通过该能力增发 RTD。

## Rust 侧：Genesis 分配与总量校验

文件：

```text
crates/rtd-types/src/gas_coin.rs
```

Rust 常量为：

```rust
pub const MIST_PER_RTD: u64 = 1_000_000_000;
pub const TOTAL_SUPPLY_RTD: u64 = 10_000_000_000;
pub const TOTAL_SUPPLY_MIST: u64 = TOTAL_SUPPLY_RTD * MIST_PER_RTD;
```

Rust 常量不直接铸币，而是用于创建 Token Distribution Schedule。构建器从全部固定供应量开始：

```rust
TokenDistributionScheduleBuilder {
    pool: TOTAL_SUPPLY_MIST,
    allocations: vec![],
}
```

随后每添加一笔创世分配，就从 `pool` 中扣除：

```rust
self.pool = self.pool.checked_sub(allocation.amount_mist).unwrap();
```

最终余额成为：

```text
stake_subsidy_fund_mist
```

Token Distribution Schedule 必须满足：

```text
stake_subsidy_fund_mist
+ sum(allocation.amount_mist)
= TOTAL_SUPPLY_MIST
```

如果不相等，Genesis 构建会失败。

相关文件：

```text
crates/rtd-config/src/genesis.rs
```

## Genesis 执行流程

Genesis Builder 在：

```text
crates/rtd-genesis-builder/src/lib.rs
```

构造以下调用链：

```text
rtd::rtd::new()
        │
        ├── 按 Move TOTAL_SUPPLY_MIST 铸造 10B RTD
        │
        ▼
rtd_system::genesis::create(...)
        │
        ├── 分出 Stake Subsidy Fund
        ├── 创建普通账户 Gas Coin
        ├── 创建 Validator Gas Coin
        └── 创建 Validator Genesis Stake
```

Move Genesis 收到完整 `Balance<RTD>` 后，先按 Schedule 分出 `stake_subsidy_fund_mist`，再处理全部 allocations。最后要求原始 `rtd_supply` 余额恰好为 0。

## `rtd-conf.yaml` 能控制什么

`rtd-conf.yaml` 可以控制：

- 普通账户地址和 `gas_amounts`；
- Validator 配置及其 `stake`；
- Stake Subsidy 的开始 Epoch、初始单次发放额、周期和衰减比例；
- Epoch 时长和协议版本。

它不能直接控制：

- `TOTAL_SUPPLY_RTD`；
- `TOTAL_SUPPLY_MIST`；
- 当前实现中每个 Validator 自动获得的默认 Gas Coin 数量。

所以调整账户余额或 Validator Stake 只会改变固定 10B 在以下去向之间的比例：

```text
普通账户 Coin
+ Validator Gas Coin
+ Validator Genesis Stake
+ Stake Subsidy Fund
= 10B RTD
```

## 修改固定总供应量的方法

如果确实要修改 RTD 固定总量，至少需要同步修改两个位置。

### 1. 修改 Rust 常量

```text
crates/rtd-types/src/gas_coin.rs
```

```rust
pub const TOTAL_SUPPLY_RTD: u64 = <新的 RTD 总量>;
```

`TOTAL_SUPPLY_MIST` 会根据 `MIST_PER_RTD` 计算。

### 2. 修改 Move 常量

```text
crates/rtd-framework/packages/rtd-framework/sources/rtd.move
```

同时更新：

```move
const TOTAL_SUPPLY_RTD: u64 = <新的 RTD 总量>;
const TOTAL_SUPPLY_MIST: u64 = <新的 MIST 总量>;
```

其中必须满足：

```text
TOTAL_SUPPLY_MIST = TOTAL_SUPPLY_RTD × 1,000,000,000
```

### 3. 重建并重新生成链

修改后需要：

1. 重新构建 RTD Framework；
2. 更新 Framework snapshots；
3. 重新编译 RTD；
4. 使用 `rtd genesis --from-config ... --force` 重新生成 `genesis.blob` 和节点配置；
5. 删除旧链数据库，从新 Genesis 启动。

仅修改源码但继续复用旧 `genesis.blob`，不会改变既有链的总供应量。

## 一致性要求

Move 和 Rust 两处常量表达的是同一个协议事实，但目前没有从单一配置文件自动生成，因此修改时必须人工保持一致。

若 Move 侧和 Rust 侧不一致，可能出现：

- Move 铸造量与 Rust Schedule 要求的分配总量不同；
- Genesis 交易执行失败；
- Token Distribution Schedule 校验失败；
- Framework snapshot 或 Genesis digest 改变。

因此，总供应量调整属于协议和 Genesis 级变更，不是普通运行参数调整。
