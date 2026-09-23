# Sui 区块链完整 Fork 指南

本文档详细说明如何完整 fork Sui 区块链项目及其所有依赖仓库，并确保 fork 后的项目能够顺利编译。

## 目录

1. [仓库清单概览](#仓库清单概览)
2. [Fork 前准备工作](#fork-前准备工作)
3. [第一阶段：Fork 所有仓库](#第一阶段fork-所有仓库)
4. [第二阶段：修改依赖配置](#第二阶段修改依赖配置)
5. [第三阶段：验证编译](#第三阶段验证编译)
6. [后续维护建议](#后续维护建议)

---

## 仓库清单概览

### 必须 Fork 的仓库（6 个核心仓库）

| # | 仓库名称 | 原始 URL | 用途 |
|---|---------|----------|------|
| 1 | **sui** (主仓库) | https://github.com/MystenLabs/sui | Sui 区块链主代码库 |
| 2 | **fastcrypto** | https://github.com/MystenLabs/fastcrypto | 高性能密码学库 |
| 3 | **mysten-sim** | https://github.com/MystenLabs/mysten-sim | 模拟器测试框架 |
| 4 | **sui-rust-sdk** | https://github.com/MystenLabs/sui-rust-sdk | Rust SDK 核心类型 |
| 5 | **anemo** | https://github.com/mystenlabs/anemo | P2P 网络通信框架 |
| 6 | **async-graphql** | https://github.com/amnn/async-graphql | GraphQL 自定义分支 |

### 可选 Fork 的仓库（6 个外部依赖）

| # | 仓库名称 | 原始 URL | 用途 |
|---|---------|----------|------|
| 7 | **axum-server** | https://github.com/bmwill/axum-server | TLS 服务器实现 |
| 8 | **datatest-stable** | https://github.com/nextest-rs/datatest-stable | 数据驱动测试框架 |
| 9 | **nexlint** | https://github.com/nextest-rs/nexlint | Linting 工具 |
| 10 | **tabled** | https://github.com/zhiburt/tabled | 表格格式化 |
| 11 | **prometheus-parser** | https://github.com/asonnino/prometheus-parser | Prometheus 解析器 |
| 12 | **tidehunter** | https://github.com/andll/tidehunter | RocksDB 优化（可选） |

### Git Submodule（1 个，仅文档用途）

| # | 仓库名称 | 原始 URL | 用途 |
|---|---------|----------|------|
| 13 | **awesome-sui** | https://github.com/sui-foundation/awesome-sui | Awesome Sui 资源集合 |

---

## Fork 前准备工作

### 步骤 0.1：确定你的 GitHub 账户类型

GitHub 有两种账户类型，fork 命令的使用方式不同：

| 账户类型 | 说明 | Fork 命令 |
|---------|------|----------|
| **个人账户 (User)** | 普通用户账户，如 `link-u-web3` | `gh repo fork REPO --clone=false` |
| **组织账户 (Organization)** | 团队/公司账户 | `gh repo fork REPO --org ORG_NAME --clone=false` |

**如何判断账户类型？**
- 访问 `https://github.com/YOUR_NAME`
- 如果页面显示 "People" 标签，则是**组织账户**
- 如果页面显示个人头像和 bio，则是**个人账户**

#### 方案 A：使用个人账户 Fork（推荐新手）

```bash
# 设置你的 GitHub 用户名
export MY_USERNAME="your-github-username"
```

#### 方案 B：使用组织账户 Fork（推荐团队协作）

```bash
# 设置你的组织名称
export MY_ORG="your-organization-name"
```

#### 方案 C：创建新的 GitHub 组织

如果你想使用组织来管理 fork 的仓库，可以创建一个新组织：

1. 访问 https://github.com/organizations/plan
2. 选择 "Create a free organization"
3. 填写组织名称（如 `my-sui-fork`）
4. 完成创建后设置环境变量：

```bash
export MY_ORG="my-sui-fork"
```

> **注意**：本文档后续使用 `$MY_ORG` 作为占位符。如果你使用个人账户，请将所有 `$MY_ORG` 替换为你的用户名 `$MY_USERNAME`。

### 步骤 0.2：安装必要工具

确保你的系统已安装以下工具：

```bash
# macOS
brew install gh git rust

# 或使用 rustup 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 验证安装
git --version
gh --version
cargo --version
```

### 步骤 0.3：配置 GitHub CLI

```bash
# 登录 GitHub CLI
gh auth login

# 验证登录状态
gh auth status
```

---

## 第一阶段：Fork 所有仓库

### 1.1 Fork 核心仓库（必须）

> **重要提示**：根据你的账户类型选择对应的命令！
> - **个人账户**：使用不带 `--org` 的命令
> - **组织账户**：使用带 `--org $MY_ORG` 的命令

#### 1.1.1 Fork Sui 主仓库

**个人账户方式：**
```bash
# Fork sui 主仓库到个人账户
gh repo fork MystenLabs/sui --clone=false

# 验证 fork 成功（替换 YOUR_USERNAME 为你的用户名）
gh repo view $YOUR_USERNAME/sui
```

**组织账户方式：**
```bash
# Fork sui 主仓库到组织
gh repo fork MystenLabs/sui --org $MY_ORG --clone=false

# 验证 fork 成功
gh repo view $MY_ORG/sui
```

#### 1.1.2 Fork fastcrypto

**个人账户方式：**
```bash
gh repo fork MystenLabs/fastcrypto --clone=false
```

**组织账户方式：**
```bash
gh repo fork MystenLabs/fastcrypto --org $MY_ORG --clone=false
```

#### 1.1.3 Fork mysten-sim

**个人账户方式：**
```bash
gh repo fork MystenLabs/mysten-sim --clone=false
```

**组织账户方式：**
```bash
gh repo fork MystenLabs/mysten-sim --org $MY_ORG --clone=false
```

#### 1.1.4 Fork sui-rust-sdk

**个人账户方式：**
```bash
gh repo fork MystenLabs/sui-rust-sdk --clone=false
```

**组织账户方式：**
```bash
gh repo fork MystenLabs/sui-rust-sdk --org $MY_ORG --clone=false
```

#### 1.1.5 Fork anemo

**个人账户方式：**
```bash
gh repo fork mystenlabs/anemo --clone=false
```

**组织账户方式：**
```bash
gh repo fork mystenlabs/anemo --org $MY_ORG --clone=false
```

#### 1.1.6 Fork async-graphql（从 amnn 的仓库）

**个人账户方式：**
```bash
gh repo fork amnn/async-graphql --clone=false
```

**组织账户方式：**
```bash
gh repo fork amnn/async-graphql --org $MY_ORG --clone=false
```

### 1.2 Fork 外部依赖仓库（推荐）

如果你希望完全控制所有依赖，请执行以下命令：

**个人账户方式：**
```bash
# axum-server
gh repo fork bmwill/axum-server --clone=false

# datatest-stable
gh repo fork nextest-rs/datatest-stable --clone=false

# nexlint
gh repo fork nextest-rs/nexlint --clone=false

# tabled
gh repo fork zhiburt/tabled --clone=false

# prometheus-parser
gh repo fork asonnino/prometheus-parser --clone=false

# tidehunter (可选，仅用于非 Windows 平台的性能优化)
gh repo fork andll/tidehunter --clone=false
```

**组织账户方式：**
```bash
# axum-server
gh repo fork bmwill/axum-server --org $MY_ORG --clone=false

# datatest-stable
gh repo fork nextest-rs/datatest-stable --org $MY_ORG --clone=false

# nexlint
gh repo fork nextest-rs/nexlint --org $MY_ORG --clone=false

# tabled
gh repo fork zhiburt/tabled --org $MY_ORG --clone=false

# prometheus-parser
gh repo fork asonnino/prometheus-parser --org $MY_ORG --clone=false

# tidehunter (可选，仅用于非 Windows 平台的性能优化)
gh repo fork andll/tidehunter --org $MY_ORG --clone=false
```

### 1.3 Fork Git Submodule（可选）

**个人账户方式：**
```bash
# awesome-sui（仅文档资源，可选）
gh repo fork sui-foundation/awesome-sui --clone=false
```

**组织账户方式：**
```bash
# awesome-sui（仅文档资源，可选）
gh repo fork sui-foundation/awesome-sui --org $MY_ORG --clone=false
```

---

## 第二阶段：修改依赖配置

### 2.1 克隆你 Fork 的 Sui 主仓库

```bash
# 克隆你的 sui fork
git clone https://github.com/$MY_ORG/sui.git
cd sui
```

### 2.2 修改根目录 Cargo.toml

打开 `Cargo.toml` 文件，修改所有 git 依赖的 URL：

#### 2.2.1 修改 MystenLabs 依赖

找到以下内容并修改（约第 442-677 行）：

**原始内容：**
```toml
msim = { git = "https://github.com/MystenLabs/mysten-sim.git", rev = "427147994705914a2f5afa42bc140794e31113b9", package = "msim" }
msim-macros = { git = "https://github.com/MystenLabs/mysten-sim.git", rev = "427147994705914a2f5afa42bc140794e31113b9", package = "msim-macros" }
```

**修改为：**
```toml
msim = { git = "https://github.com/YOUR_ORG/mysten-sim.git", rev = "427147994705914a2f5afa42bc140794e31113b9", package = "msim" }
msim-macros = { git = "https://github.com/YOUR_ORG/mysten-sim.git", rev = "427147994705914a2f5afa42bc140794e31113b9", package = "msim-macros" }
```

#### 2.2.2 修改 fastcrypto 依赖（第 650-655 行）

**原始内容：**
```toml
fastcrypto = { git = "https://github.com/MystenLabs/fastcrypto", rev = "4db0e90c732bbf7420ca20de808b698883148d9c" }
fastcrypto-tbls = { git = "https://github.com/MystenLabs/fastcrypto", rev = "4db0e90c732bbf7420ca20de808b698883148d9c" }
fastcrypto-zkp = { git = "https://github.com/MystenLabs/fastcrypto", rev = "4db0e90c732bbf7420ca20de808b698883148d9c", package = "fastcrypto-zkp" }
fastcrypto-vdf = { git = "https://github.com/MystenLabs/fastcrypto", rev = "4db0e90c732bbf7420ca20de808b698883148d9c", features = ["experimental"] }
```

**修改为：**
```toml
fastcrypto = { git = "https://github.com/YOUR_ORG/fastcrypto", rev = "4db0e90c732bbf7420ca20de808b698883148d9c" }
fastcrypto-tbls = { git = "https://github.com/YOUR_ORG/fastcrypto", rev = "4db0e90c732bbf7420ca20de808b698883148d9c" }
fastcrypto-zkp = { git = "https://github.com/YOUR_ORG/fastcrypto", rev = "4db0e90c732bbf7420ca20de808b698883148d9c", package = "fastcrypto-zkp" }
fastcrypto-vdf = { git = "https://github.com/YOUR_ORG/fastcrypto", rev = "4db0e90c732bbf7420ca20de808b698883148d9c", features = ["experimental"] }
```

#### 2.2.3 修改 anemo 依赖（第 669-672 行）

**原始内容：**
```toml
anemo = { git = "https://github.com/mystenlabs/anemo.git", rev = "4b5f0f1d06a31c8ef78ec2e5b446bc633e4e2f77" }
anemo-build = { git = "https://github.com/mystenlabs/anemo.git", rev = "4b5f0f1d06a31c8ef78ec2e5b446bc633e4e2f77" }
anemo-cli = { git = "https://github.com/mystenlabs/anemo.git", rev = "4b5f0f1d06a31c8ef78ec2e5b446bc633e4e2f77" }
anemo-tower = { git = "https://github.com/mystenlabs/anemo.git", rev = "4b5f0f1d06a31c8ef78ec2e5b446bc633e4e2f77" }
```

**修改为：**
```toml
anemo = { git = "https://github.com/YOUR_ORG/anemo.git", rev = "4b5f0f1d06a31c8ef78ec2e5b446bc633e4e2f77" }
anemo-build = { git = "https://github.com/YOUR_ORG/anemo.git", rev = "4b5f0f1d06a31c8ef78ec2e5b446bc633e4e2f77" }
anemo-cli = { git = "https://github.com/YOUR_ORG/anemo.git", rev = "4b5f0f1d06a31c8ef78ec2e5b446bc633e4e2f77" }
anemo-tower = { git = "https://github.com/YOUR_ORG/anemo.git", rev = "4b5f0f1d06a31c8ef78ec2e5b446bc633e4e2f77" }
```

#### 2.2.4 修改 sui-rust-sdk 依赖（第 675-677 行）

**原始内容：**
```toml
sui-sdk-types = { git = "https://github.com/MystenLabs/sui-rust-sdk.git", rev = "339c2272fd5b8fb4e1fa6662cfa9acdbb0d05704", features = [ "hash", "serde" ] }
sui-crypto = { git = "https://github.com/MystenLabs/sui-rust-sdk.git", rev = "339c2272fd5b8fb4e1fa6662cfa9acdbb0d05704", features = [ "ed25519", "secp256r1", "secp256k1", "passkey", "zklogin" ] }
sui-rpc = { git = "https://github.com/MystenLabs/sui-rust-sdk.git", rev = "339c2272fd5b8fb4e1fa6662cfa9acdbb0d05704" }
```

**修改为：**
```toml
sui-sdk-types = { git = "https://github.com/YOUR_ORG/sui-rust-sdk.git", rev = "339c2272fd5b8fb4e1fa6662cfa9acdbb0d05704", features = [ "hash", "serde" ] }
sui-crypto = { git = "https://github.com/YOUR_ORG/sui-rust-sdk.git", rev = "339c2272fd5b8fb4e1fa6662cfa9acdbb0d05704", features = [ "ed25519", "secp256r1", "secp256k1", "passkey", "zklogin" ] }
sui-rpc = { git = "https://github.com/YOUR_ORG/sui-rust-sdk.git", rev = "339c2272fd5b8fb4e1fa6662cfa9acdbb0d05704" }
```

#### 2.2.5 修改 patch.crates-io 中的 async-graphql（第 809-812 行）

**原始内容：**
```toml
[patch.crates-io]
async-graphql = { git = "https://github.com/amnn/async-graphql", branch = "v7.0.1-react-18-graphiql-4" }
async-graphql-axum = { git = "https://github.com/amnn/async-graphql", branch = "v7.0.1-react-18-graphiql-4" }
async-graphql-value = { git = "https://github.com/amnn/async-graphql", branch = "v7.0.1-react-18-graphiql-4" }
```

**修改为：**
```toml
[patch.crates-io]
async-graphql = { git = "https://github.com/YOUR_ORG/async-graphql", branch = "v7.0.1-react-18-graphiql-4" }
async-graphql-axum = { git = "https://github.com/YOUR_ORG/async-graphql", branch = "v7.0.1-react-18-graphiql-4" }
async-graphql-value = { git = "https://github.com/YOUR_ORG/async-graphql", branch = "v7.0.1-react-18-graphiql-4" }
```

#### 2.2.6 修改其他外部依赖（如果你 fork 了它们）

**axum-server（第 320-322 行）：**
```toml
# 原始
axum-server = { git = "https://github.com/bmwill/axum-server.git", rev = "f44323e271afdd1365fd0c8b0a4c0bbdf4956cb7", ... }
# 修改为
axum-server = { git = "https://github.com/YOUR_ORG/axum-server.git", rev = "f44323e271afdd1365fd0c8b0a4c0bbdf4956cb7", ... }
```

**datatest-stable（第 364 行）：**
```toml
# 原始
datatest-stable = { git = "https://github.com/nextest-rs/datatest-stable.git", rev = "72db7f6d1bbe36a5407e96b9488a581f763e106f" }
# 修改为
datatest-stable = { git = "https://github.com/YOUR_ORG/datatest-stable.git", rev = "72db7f6d1bbe36a5407e96b9488a581f763e106f" }
```

**nexlint（第 445-446 行）：**
```toml
# 原始
nexlint = { git = "https://github.com/nextest-rs/nexlint.git", rev = "7ce56bd591242a57660ed05f14ca2483c37d895b" }
nexlint-lints = { git = "https://github.com/nextest-rs/nexlint.git", rev = "7ce56bd591242a57660ed05f14ca2483c37d895b" }
# 修改为
nexlint = { git = "https://github.com/YOUR_ORG/nexlint.git", rev = "7ce56bd591242a57660ed05f14ca2483c37d895b" }
nexlint-lints = { git = "https://github.com/YOUR_ORG/nexlint.git", rev = "7ce56bd591242a57660ed05f14ca2483c37d895b" }
```

**json_to_table（第 430 行）：**
```toml
# 原始
json_to_table = { git = "https://github.com/zhiburt/tabled/", rev = "e449317a1c02eb6b29e409ad6617e5d9eb7b3bd4" }
# 修改为
json_to_table = { git = "https://github.com/YOUR_ORG/tabled/", rev = "e449317a1c02eb6b29e409ad6617e5d9eb7b3bd4" }
```

**prometheus-parse（第 475 行）：**
```toml
# 原始
prometheus-parse = { git = "https://github.com/asonnino/prometheus-parser.git", rev = "75334db" }
# 修改为
prometheus-parse = { git = "https://github.com/YOUR_ORG/prometheus-parser.git", rev = "75334db" }
```

### 2.3 修改 typed-store 中的 tidehunter 依赖

编辑文件 `crates/typed-store/Cargo.toml`（第 42 行）：

**原始内容：**
```toml
tidehunter = {git = "https://github.com/andll/tidehunter.git", rev = "dd686f055375aa8fa2145618301bdfd5170a2a6b", version = "0.1.0", optional = true}
```

**修改为：**
```toml
tidehunter = {git = "https://github.com/YOUR_ORG/tidehunter.git", rev = "dd686f055375aa8fa2145618301bdfd5170a2a6b", version = "0.1.0", optional = true}
```

### 2.4 修改 .gitmodules

编辑文件 `.gitmodules`：

**原始内容：**
```ini
[submodule "docs/submodules/awesome-sui"]
    path = docs/submodules/awesome-sui
    url = https://github.com/sui-foundation/awesome-sui.git
```

**修改为：**
```ini
[submodule "docs/submodules/awesome-sui"]
    path = docs/submodules/awesome-sui
    url = https://github.com/YOUR_ORG/awesome-sui.git
```

然后更新 submodule 配置：

```bash
# 同步 submodule 配置
git submodule sync

# 重新初始化 submodule
git submodule update --init --recursive
```

---

## 第三阶段：验证编译

### 3.1 清理 Cargo 缓存

```bash
# 删除旧的 git 依赖缓存，确保使用新的 fork 仓库
rm -rf ~/.cargo/git/checkouts/
rm -rf ~/.cargo/git/db/

# 清理本地构建缓存
cargo clean
```

### 3.2 验证依赖解析

```bash
# 更新依赖锁文件
cargo update

# 检查依赖是否能正确解析
cargo fetch
```

### 3.3 编译项目

```bash
# 首先进行快速检查（不生成二进制文件）
cargo check

# 构建整个项目
cargo build

# 或者只构建核心组件
cargo build -p sui-node
cargo build -p sui
```

### 3.4 运行测试（可选）

```bash
# 运行单元测试
SUI_SKIP_SIMTESTS=1 cargo nextest run -p sui-types -p sui-core

# 运行模拟测试
cargo simtest -p sui-e2e-tests
```

---

## 第四阶段：自动化脚本

为了简化上述流程，可以使用以下自动化脚本：

### 4.1 Fork 脚本 (fork_all.sh)

```bash
#!/bin/bash
set -e

# 配置你的组织名称
MY_ORG="${1:-YOUR_ORG}"

echo "=== Forking all Sui repositories to $MY_ORG ==="

# 核心仓库
CORE_REPOS=(
    "MystenLabs/sui"
    "MystenLabs/fastcrypto"
    "MystenLabs/mysten-sim"
    "MystenLabs/sui-rust-sdk"
    "mystenlabs/anemo"
    "amnn/async-graphql"
)

# 外部依赖仓库（可选）
EXTERNAL_REPOS=(
    "bmwill/axum-server"
    "nextest-rs/datatest-stable"
    "nextest-rs/nexlint"
    "zhiburt/tabled"
    "asonnino/prometheus-parser"
    "andll/tidehunter"
    "sui-foundation/awesome-sui"
)

echo "=== Forking core repositories ==="
for repo in "${CORE_REPOS[@]}"; do
    echo "Forking $repo..."
    gh repo fork "$repo" --org "$MY_ORG" --clone=false || echo "Already forked or error"
done

echo ""
read -p "Do you want to fork external dependencies too? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "=== Forking external repositories ==="
    for repo in "${EXTERNAL_REPOS[@]}"; do
        echo "Forking $repo..."
        gh repo fork "$repo" --org "$MY_ORG" --clone=false || echo "Already forked or error"
    done
fi

echo ""
echo "=== Fork complete! ==="
echo "Next steps:"
echo "1. Clone your sui fork: git clone https://github.com/$MY_ORG/sui.git"
echo "2. Run the update script to modify Cargo.toml"
```

### 4.2 依赖替换脚本 (update_deps.sh)

```bash
#!/bin/bash
set -e

# 配置
MY_ORG="${1:-YOUR_ORG}"
SUI_DIR="${2:-.}"

echo "=== Updating dependencies in $SUI_DIR to use $MY_ORG ==="

cd "$SUI_DIR"

# 替换 MystenLabs 为你的组织
declare -A REPLACEMENTS=(
    ["MystenLabs/fastcrypto"]="$MY_ORG/fastcrypto"
    ["MystenLabs/mysten-sim"]="$MY_ORG/mysten-sim"
    ["MystenLabs/sui-rust-sdk"]="$MY_ORG/sui-rust-sdk"
    ["mystenlabs/anemo"]="$MY_ORG/anemo"
    ["amnn/async-graphql"]="$MY_ORG/async-graphql"
    ["bmwill/axum-server"]="$MY_ORG/axum-server"
    ["nextest-rs/datatest-stable"]="$MY_ORG/datatest-stable"
    ["nextest-rs/nexlint"]="$MY_ORG/nexlint"
    ["zhiburt/tabled"]="$MY_ORG/tabled"
    ["asonnino/prometheus-parser"]="$MY_ORG/prometheus-parser"
    ["andll/tidehunter"]="$MY_ORG/tidehunter"
    ["sui-foundation/awesome-sui"]="$MY_ORG/awesome-sui"
)

# 更新根目录 Cargo.toml
echo "Updating Cargo.toml..."
for old in "${!REPLACEMENTS[@]}"; do
    new="${REPLACEMENTS[$old]}"
    sed -i.bak "s|github.com/$old|github.com/$new|g" Cargo.toml
done

# 更新 typed-store/Cargo.toml
echo "Updating crates/typed-store/Cargo.toml..."
for old in "${!REPLACEMENTS[@]}"; do
    new="${REPLACEMENTS[$old]}"
    sed -i.bak "s|github.com/$old|github.com/$new|g" crates/typed-store/Cargo.toml
done

# 更新 .gitmodules
echo "Updating .gitmodules..."
for old in "${!REPLACEMENTS[@]}"; do
    new="${REPLACEMENTS[$old]}"
    sed -i.bak "s|github.com/$old|github.com/$new|g" .gitmodules
done

# 删除备份文件
find . -name "*.bak" -delete

echo ""
echo "=== Update complete! ==="
echo "Next steps:"
echo "1. git submodule sync"
echo "2. cargo clean"
echo "3. cargo update"
echo "4. cargo check"
```

### 4.3 使用脚本

```bash
# 设置可执行权限
chmod +x fork_all.sh update_deps.sh

# 执行 fork
./fork_all.sh YOUR_ORG

# 克隆你的 sui fork
git clone https://github.com/YOUR_ORG/sui.git
cd sui

# 更新依赖
./update_deps.sh YOUR_ORG .

# 验证编译
cargo check
```

---

## 后续维护建议

### 5.1 设置上游同步

为每个 fork 的仓库设置上游远程，以便同步最新更新：

```bash
# 在 sui 仓库中
git remote add upstream https://github.com/MystenLabs/sui.git

# 同步上游更新
git fetch upstream
git merge upstream/main
```

### 5.2 定期同步依赖仓库

当上游更新了依赖版本（新的 rev）时，你需要：

1. 同步对应的 fork 仓库
2. 更新 `Cargo.toml` 中的 rev 值

### 5.3 CI/CD 配置

确保你的 CI/CD 系统有权限访问所有 fork 的仓库。如果使用私有仓库，可能需要配置 SSH 密钥或 PAT (Personal Access Token)。

---

## 快速参考：完整替换清单

### Cargo.toml 替换表

| 行号 | 原始 URL | 替换为 |
|------|----------|--------|
| 320 | `github.com/bmwill/axum-server` | `github.com/YOUR_ORG/axum-server` |
| 364 | `github.com/nextest-rs/datatest-stable` | `github.com/YOUR_ORG/datatest-stable` |
| 430 | `github.com/zhiburt/tabled` | `github.com/YOUR_ORG/tabled` |
| 442-443 | `github.com/MystenLabs/mysten-sim` | `github.com/YOUR_ORG/mysten-sim` |
| 445-446 | `github.com/nextest-rs/nexlint` | `github.com/YOUR_ORG/nexlint` |
| 475 | `github.com/asonnino/prometheus-parser` | `github.com/YOUR_ORG/prometheus-parser` |
| 650-655 | `github.com/MystenLabs/fastcrypto` | `github.com/YOUR_ORG/fastcrypto` |
| 669-672 | `github.com/mystenlabs/anemo` | `github.com/YOUR_ORG/anemo` |
| 675-677 | `github.com/MystenLabs/sui-rust-sdk` | `github.com/YOUR_ORG/sui-rust-sdk` |
| 810-812 | `github.com/amnn/async-graphql` | `github.com/YOUR_ORG/async-graphql` |

### crates/typed-store/Cargo.toml 替换表

| 行号 | 原始 URL | 替换为 |
|------|----------|--------|
| 42 | `github.com/andll/tidehunter` | `github.com/YOUR_ORG/tidehunter` |

### .gitmodules 替换表

| 原始 URL | 替换为 |
|----------|--------|
| `github.com/sui-foundation/awesome-sui` | `github.com/YOUR_ORG/awesome-sui` |

---

## 故障排除

### 问题 1：使用 --org 参数时提示 "is the login for a user account"

**错误信息：**
```
failed to fork: HTTP 422: 'your-username' is the login for a user account.
You must pass the login for an organization account.
Fork.organization is invalid
```

**原因：**
`--org` 参数只能用于 GitHub **组织账户**（Organization），不能用于**个人账户**（User Account）。

**解决方案：**

方案 A - 使用个人账户 fork（去掉 `--org` 参数）：
```bash
# 错误的命令（个人账户不能使用 --org）
gh repo fork MystenLabs/sui --org your-username --clone=false

# 正确的命令（个人账户直接 fork）
gh repo fork MystenLabs/sui --clone=false
```

方案 B - 创建一个 GitHub 组织：
1. 访问 https://github.com/organizations/plan
2. 选择 "Create a free organization"
3. 创建组织后，使用组织名称执行 fork：
```bash
gh repo fork MystenLabs/sui --org your-new-org --clone=false
```

### 问题 2：依赖解析失败

```
error: failed to get `xxx` as a dependency
```

**解决方案：**
1. 确保你已经 fork 了对应的仓库
2. 确保 fork 仓库是公开的或你有访问权限
3. 检查 URL 拼写是否正确

### 问题 2：rev 不存在

```
error: failed to find revision `xxx`
```

**解决方案：**
1. 确保你的 fork 包含了所有分支和标签
2. 同步上游仓库：`git fetch upstream && git push origin --all`

### 问题 3：分支不存在

```
error: failed to find branch `v7.0.1-react-18-graphiql-4`
```

**解决方案：**
对于 async-graphql，确保 fork 了 `amnn/async-graphql` 而不是原始的 `async-graphql/async-graphql`，因为需要特定的自定义分支。

---

## 总结

完整 Fork Sui 项目需要 fork **6 个核心仓库**（必须）和 **7 个外部依赖仓库**（推荐），共计 **13 个仓库**。

核心仓库是编译必需的，外部依赖仓库则根据你的需求决定是否 fork。如果你只是想进行简单的定制化开发，可以只 fork 核心仓库。

祝你 fork 顺利！
