# Sui 区块链完整 Fork 指南（终极版）

本文档详细说明如何从 Sui 区块链源码完整 fork 出一条全新品牌的区块链，确保与原 Sui 网络完全不兼容。

## 目录

1. [概述](#概述)
2. [第一部分：环境准备](#第一部分环境准备)
3. [第二部分：Fork 所有依赖仓库](#第二部分fork-所有依赖仓库)
4. [第三部分：品牌重命名](#第三部分品牌重命名)
5. [第四部分：重编译 Move 字节码](#第四部分重编译-move-字节码)
6. [第五部分：构建和测试](#第五部分构建和测试)
7. [第六部分：Genesis 创建](#第六部分genesis-创建)
8. [常见问题排查](#常见问题排查)
9. [附录](#附录)

---

## 概述

### 本指南目标

将 Sui 区块链完整 fork 为一条新品牌区块链（以 RTD 为例），实现：

- **完全独立的品牌标识**：所有 "Sui/SUI/sui" 替换为新品牌名
- **与 Sui 网络完全不兼容**：账户、智能合约、代币均不互通
- **独立的 Genesis**：创建全新的创世区块

### 品牌替换规则

| 原始 | 替换为 | 说明 |
|------|--------|------|
| MystenLabs | LinkUVerse | 组织名称 |
| Mysten | LinkU | 品牌名称（首字母大写） |
| mysten | linku | 品牌名称（纯小写） |
| SUI | RTD | 代币符号（纯大写） |
| Sui | Rtd | 混合大小写 |
| sui | rtd | 纯小写 |

### Fork 后的不兼容性

| 方面 | Sui | RTD (Fork) | 兼容性 |
|------|-----|------------|--------|
| 原生代币 | `Coin<sui::sui::SUI>` | `Coin<rtd::rtd::RTD>` | ❌ 不兼容 |
| 系统模块 | `sui::*`, `sui_system::*` | `rtd::*`, `rtd_system::*` | ❌ 不兼容 |
| Chain ID | Sui 主网/测试网 ID | 新 genesis 产生新 ID | ❌ 不兼容 |
| 网络节点 | Sui 节点 | RTD 节点 | ❌ 无法互连 |

---

## 第一部分：环境准备

### 1.1 必要工具安装

```bash
# macOS
brew install gh git rust

# 或使用 rustup 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 验证安装
git --version    # >= 2.30
gh --version     # >= 2.0
cargo --version  # >= 1.75
```

### 1.2 配置 GitHub CLI

```bash
# 登录 GitHub CLI
gh auth login

# 验证登录状态
gh auth status
```

### 1.3 确定 GitHub 账户类型

| 账户类型 | 说明 | Fork 命令 |
|---------|------|----------|
| 个人账户 | 普通用户 | `gh repo fork REPO --clone=false` |
| 组织账户 | 团队账户 | `gh repo fork REPO --org ORG_NAME --clone=false` |

```bash
# 设置你的账户/组织名称
export MY_ORG="your-organization-name"
# 或
export MY_USERNAME="your-github-username"
```

---

## 第二部分：Fork 所有依赖仓库

### 2.1 必须 Fork 的仓库（6 个核心）

| # | 仓库名称 | 原始 URL | 用途 |
|---|---------|----------|------|
| 1 | **sui** | https://github.com/MystenLabs/sui | 主代码库 |
| 2 | **fastcrypto** | https://github.com/MystenLabs/fastcrypto | 密码学库 |
| 3 | **mysten-sim** | https://github.com/MystenLabs/mysten-sim | 模拟器框架 |
| 4 | **sui-rust-sdk** | https://github.com/MystenLabs/sui-rust-sdk | Rust SDK |
| 5 | **anemo** | https://github.com/mystenlabs/anemo | P2P 网络框架 |
| 6 | **async-graphql** | https://github.com/amnn/async-graphql | GraphQL 分支 |

### 2.2 Fork 命令

```bash
# Fork 所有核心仓库到你的组织
gh repo fork MystenLabs/sui --org $MY_ORG --clone=false
gh repo fork MystenLabs/fastcrypto --org $MY_ORG --clone=false
gh repo fork MystenLabs/mysten-sim --org $MY_ORG --clone=false
gh repo fork MystenLabs/sui-rust-sdk --org $MY_ORG --clone=false
gh repo fork mystenlabs/anemo --org $MY_ORG --clone=false
gh repo fork amnn/async-graphql --org $MY_ORG --clone=false
```

### 2.3 重命名 Fork 的仓库（可选）

```bash
# 重命名仓库以匹配新品牌
gh repo rename rtd --repo $MY_ORG/sui
gh repo rename rtd-rust-sdk --repo $MY_ORG/sui-rust-sdk
gh repo rename linku-sim --repo $MY_ORG/mysten-sim
```

### 2.4 Clone 主仓库

```bash
git clone https://github.com/$MY_ORG/sui.git rtd
cd rtd
```

---

## 第三部分：品牌重命名

### 3.1 使用一键式脚本

将 `rtd-brand-rename.sh` 脚本放入项目的 `fork-instruct/final/` 目录，然后执行：

```bash
# 完整执行所有阶段
./fork-instruct/final/rtd-brand-rename.sh all

# 或单独执行某个阶段
./fork-instruct/final/rtd-brand-rename.sh phase01  # 文本替换
./fork-instruct/final/rtd-brand-rename.sh phase03  # 重命名 crates
```

### 3.2 脚本执行的阶段

| 阶段 | 功能 | 说明 |
|------|------|------|
| phase00 | 准备工作 | 创建备份分支和标签 |
| phase01 | 文本替换 | 替换所有文件中的品牌标识 |
| phase02 | 更新 SDK 依赖 | 更新 Cargo.toml 中的依赖 |
| phase03 | 重命名 crates | `crates/sui-*` → `crates/rtd-*` |
| phase04 | 重命名 execution | `sui-execution` → `rtd-execution` |
| phase05 | 重命名 framework | 框架包目录重命名 |
| phase06 | 重命名 docker | Docker 相关目录 |
| phase07 | 重命名 ansible | Ansible 配置目录 |
| phase08 | 重命名 docs | 文档目录 |
| phase09 | 重命名 move | Move 编译器目录 |
| phase10 | 重命名其他 | 其他 sui* 目录 |
| phase11 | 重命名文件 | sui*.rs 等文件 |
| phase12 | 修复特殊情况 | 常量冲突等问题 |
| phase13 | 重编译字节码 | 重新编译 Move 包 |
| phase14 | 清理验证 | 检查遗漏和统计 |

### 3.3 处理编译错误

执行脚本后，尝试编译：

```bash
cargo build --workspace
```

如果遇到错误，检查：
1. 是否有遗漏的 `sui/Sui/SUI` 引用
2. 是否有目录/文件未重命名
3. 是否有常量/枚举冲突

---

## 第四部分：重编译 Move 字节码

### 4.1 为什么需要重编译

品牌重命名只修改了 Move 源代码，但预编译的字节码文件仍包含原始的 `sui::SUI`。需要：

1. 重新编译 `packages_compiled/` 目录
2. 删除或重新生成 `bytecode_snapshot/` 目录

### 4.2 重编译 packages_compiled

```bash
# 从 Move 源代码重新编译系统包字节码
UPDATE=1 cargo test -p rtd-framework --test build-system-packages
```

此命令会：
- 从 `crates/rtd-framework/packages/` 中的 Move 源代码重新编译
- 生成新的 `packages_compiled/` 字节码文件
- 新字节码将包含正确的 `rtd::RTD` 名称

### 4.3 删除旧的字节码快照

```bash
# 删除所有历史版本的字节码快照
rm -rf crates/rtd-framework-snapshot/bytecode_snapshot/*
```

删除后，genesis 过程会自动使用 `BuiltInFramework`（新编译的字节码）。

### 4.4 验证重编译结果

```bash
# 检查是否还有 sui 引用（应无输出）
strings crates/rtd-framework/packages_compiled/rtd-framework | grep -E '::sui::'

# 检查是否包含正确的 rtd 引用（应有输出）
strings crates/rtd-framework/packages_compiled/rtd-framework | grep -E '::rtd::'
```

---

## 第五部分：构建和测试

### 5.1 完整构建

```bash
# 构建整个工作空间
cargo build --workspace

# 或构建 release 版本
cargo build --release -p rtd
```

### 5.2 运行测试

```bash
# 运行单元测试（跳过模拟测试）
RTD_SKIP_SIMTESTS=1 cargo nextest run

# 运行特定 crate 的测试
cargo nextest run -p rtd-types -p rtd-core
```

### 5.3 代码检查

```bash
# 格式化代码
cargo fmt --all

# Lint 检查
cargo xclippy
```

---

## 第六部分：Genesis 创建

### 6.1 创建 Genesis 配置文件

创建 `genesis-config.yaml`：

```yaml
ssfn_config_info: ~
validator_config_info: ~
parameters:
  chain_start_timestamp_ms: 1759857384897
  protocol_version: 105  # 使用最新协议版本
  allow_insertion_of_extra_objects: true
  epoch_duration_ms: 86400000
  stake_subsidy_start_epoch: 0
  stake_subsidy_initial_distribution_amount: 1000000000000000
  stake_subsidy_period_length: 10
  stake_subsidy_decrease_rate: 1000
accounts:
  - address: "0x..."  # 你的地址
    gas_amounts:
      - 5200000000000000
```

### 6.2 执行 Genesis

```bash
rtd genesis --force --from-config ./genesis-config.yaml
```

### 6.3 验证 Genesis

```bash
# 查看生成的 genesis 文件
ls -la ~/.rtd/rtd_config/
```

---

## 常见问题排查

### Q1: `sui::SUI` 错误

**错误信息**：
```
one-time witness type 0x2::sui::SUI is instantiated in the 0x2::sui::new function
```

**原因**：预编译的字节码没有重新生成

**解决方案**：
```bash
UPDATE=1 cargo test -p rtd-framework --test build-system-packages
rm -rf crates/rtd-framework-snapshot/bytecode_snapshot/*
```

### Q2: `MISSING_DEPENDENCY` 错误

**错误信息**：
```
VMError with status MISSING_DEPENDENCY at location Module ModuleId { address: 0x1, name: "type_name" }
```

**原因**：协议版本与字节码不兼容

**解决方案**：
1. 检查 `genesis-config.yaml` 中的 `protocol_version`
2. 使用支持的协议版本（1-105）

### Q3: 编译错误 - 常量冲突

**错误信息**：
```
error: the name `RTD` is defined multiple times
```

**原因**：常量和枚举变体同名

**解决方案**：
检查 `external-crates/move/crates/move-compiler/src/editions/mod.rs`，确保常量正确命名。

### Q4: 目录/文件未重命名

**检查命令**：
```bash
# 查找遗漏的 sui 目录
find . -type d -name "sui-*" ! -path "./.git/*" ! -path "./target/*"

# 查找遗漏的 sui 文件
find . -type f -name "sui_*.rs" ! -path "./.git/*" ! -path "./target/*"
```

---

## 附录

### A. 完整目录重命名对照表

| 原始目录 | 新目录 |
|---------|--------|
| `crates/sui-*` | `crates/rtd-*` |
| `crates/mysten-*` | `crates/linku-*` |
| `sui-execution/` | `rtd-execution/` |
| `crates/rtd-framework/packages/sui-framework` | `crates/rtd-framework/packages/rtd-framework` |
| `crates/rtd-framework/packages/sui-system` | `crates/rtd-framework/packages/rtd-system` |

### B. 回滚方法

如果需要回滚所有更改：

```bash
# 回滚到备份标签
git checkout pre-rename-backup-v2
git reset --hard pre-rename-backup-v2

# 或完全重新克隆
rm -rf rtd
git clone https://github.com/$MY_ORG/sui.git rtd
```

### C. 自定义品牌配置

如果要使用自定义品牌名，修改 `brand-config.sh`：

```bash
# 品牌配置
OLD_ORG="MystenLabs"    NEW_ORG="YourOrg"
OLD_BRAND="Mysten"      NEW_BRAND="YourBrand"
OLD_UPPER="SUI"         NEW_UPPER="YOUR"
OLD_MIXED="Sui"         NEW_MIXED="Your"
OLD_LOWER="sui"         NEW_LOWER="your"
```

然后执行：
```bash
./rtd-brand-rename.sh --config ./brand-config.sh all
```

### D. 文件类型覆盖范围

脚本处理的文件类型：
- 代码文件：`*.rs`, `*.move`, `*.ts`, `*.js`, `*.tsx`, `*.jsx`
- 配置文件：`*.toml`, `*.yaml`, `*.yml`, `*.json`, `*.conf`, `*.cfg`, `*.ini`
- 文档文件：`*.md`, `*.mdx`, `*.txt`
- 脚本文件：`*.sh`, `*.py`
- 其他文件：`*.graphql`, `*.proto`, `*.sql`, `*.ptb`, `*.snap`, `*.lock`

---

## 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.0 | 2024-12-21 | 初始版本，整合 v2 脚本和所有补丁 |

## 贡献者

本文档基于以下脚本整合：
- `fork-instruct/v2/run-rename-v2.sh`
- `fork-instruct/v2/run-rename-v2-patch1.sh` ~ `patch7.sh`
