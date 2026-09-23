# RTD Kubernetes 测试网部署手册

本文档说明如何把本仓库的 debug 构建切换为正式 release 产物，打包 Linux 容器镜像，并在 Kubernetes 中部署一条由多个创世验证者节点和全节点组成的 RTD 测试网。

RTD 是 Sui 的 fork，节点部署模型仍然沿用 Sui/Rtd 的验证者、全节点、genesis blob、validator key-pair 和 P2P state-sync 机制。本手册只覆盖一条新测试网从零启动的流程，不覆盖主网级别的密钥托管、HSM、多地域 DNS、监控告警和链上治理流程。

## 0. 关键结论

1. Kubernetes 运行环境通常是 `linux/amd64` 或 `linux/arm64`，不要把 macOS 下的 `target/release/rtd-node` 直接复制进 Kubernetes 镜像。建议使用本仓库现有的 Dockerfile 在 Linux builder 镜像里编译 release 产物。
2. 验证者节点的 `network-address`、`p2p-address`、Narwhal 地址和公钥会进入 `genesis.blob`。创世验证者的稳定 DNS 必须在 genesis 生成前规划好。
3. 每个验证者必须使用独立的 `protocol.key`、`account.key`、`network.key`、`worker.key`。这些密钥放在 Kubernetes Secret 中，不能放进镜像。
4. 每个节点都需要持久化 RocksDB。验证者和全节点建议使用 StatefulSet + PVC，测试网可以先用较小磁盘，公开测试网应按业务规模扩容。
5. 全节点通过 `p2p-config.seed-peers` 连接验证者，验证者如配置了 `anemo-config.max-concurrent-connections: 0`，需要把允许同步的全节点加入验证者的 `seed-peers` 白名单。

## 1. 目录和变量

以下命令默认在仓库根目录执行：

```bash
cd /Users/changzechuan/WenchuanProjects/RTD-Blockchain/rtd
```

建议先定义这些变量：

```bash
export NAMESPACE=rtd-testnet
export REGISTRY=registry.example.com/rtd
export IMAGE_TAG=$(git rev-parse --short=12 HEAD)
export RTD_NODE_IMAGE=${REGISTRY}/rtd-node:${IMAGE_TAG}
export RTD_TOOLS_IMAGE=${REGISTRY}/rtd-tools:${IMAGE_TAG}
export GENESIS_WORKDIR=$PWD/deploy-artifacts/testnet-genesis
export RTD_BIN=$PWD/target/release/rtd
export VALIDATOR_COUNT=4
export FULLNODE_COUNT=2
```

变量含义：

- `NAMESPACE`：Kubernetes 命名空间。
- `REGISTRY`：你的镜像仓库地址，例如 ECR、GCR、Harbor、Docker Hub。
- `IMAGE_TAG`：镜像 tag，建议使用 git commit 或 release tag，不要使用裸 `latest`。
- `GENESIS_WORKDIR`：本地生成 genesis、密钥和 Kubernetes YAML 的临时目录。该目录会包含私钥，必须纳入 `.gitignore` 或放在仓库外。
- `VALIDATOR_COUNT`：创世验证者数量。
- `FULLNODE_COUNT`：全节点数量。

创建工作目录：

```bash
mkdir -p "$GENESIS_WORKDIR"/{keys,configs,k8s}
chmod 700 "$GENESIS_WORKDIR" "$GENESIS_WORKDIR/keys"
```

## 2. 编译 release 产物

### 2.1 本机只做可用性检查

如果本机不是 Linux，只建议用下面命令确认 Rust 代码能编译，不建议把产物用于 Kubernetes：

```bash
cargo build --release --bin rtd-node --bin rtd
```

产物位置：

```bash
ls -lh target/release/rtd-node target/release/rtd
```

说明：

- `rtd-node` 是节点进程，验证者和全节点都运行这个二进制。
- `rtd` 是 CLI，用于 genesis、keytool、链上操作和客户端验证。
- debug 产物在 `target/debug`，正式部署必须使用 `--release` 或 Dockerfile 的 `PROFILE=release`。

### 2.2 使用 Docker 构建 Linux release 镜像

本仓库已经有 `docker/rtd-node/Dockerfile`，会在 `rust:1.90-bullseye` builder 阶段执行：

```bash
cargo build --profile release --bin rtd-node
```

构建 `rtd-node` runtime 镜像：

```bash
docker build \
  --platform linux/amd64 \
  -f docker/rtd-node/Dockerfile \
  --build-arg PROFILE=release \
  --build-arg GIT_REVISION="$(git describe --always --abbrev=12 --dirty --exclude '*')" \
  --build-arg BUILD_DATE="$(date -u +'%Y-%m-%d')" \
  -t "$RTD_NODE_IMAGE" \
  .
```

构建包含 `rtd` CLI 的工具镜像。这个镜像主要用于 CI、Job、调试和 genesis 操作，不建议作为节点 runtime 镜像：

```bash
docker build \
  --platform linux/amd64 \
  -f docker/rtd-tools/Dockerfile \
  --build-arg PROFILE=release \
  --build-arg GIT_REVISION="$(git describe --always --abbrev=12 --dirty --exclude '*')" \
  --build-arg BUILD_DATE="$(date -u +'%Y-%m-%d')" \
  -t "$RTD_TOOLS_IMAGE" \
  .
```

推送镜像：

```bash
docker push "$RTD_NODE_IMAGE"
docker push "$RTD_TOOLS_IMAGE"
```

验证镜像内二进制：

```bash
docker run --rm "$RTD_NODE_IMAGE" /opt/rtd/bin/rtd-node --help
docker run --rm "$RTD_TOOLS_IMAGE" rtd --help
```

如果你的 Kubernetes 集群是 `linux/arm64`，把 `--platform linux/amd64` 改为 `linux/arm64`，并确保 Rust 依赖和 RocksDB 在 arm64 builder 中能正常编译。

## 3. 规划 Kubernetes DNS 和端口

创世验证者建议使用每个验证者独立 Headless Service 的稳定 DNS：

```text
rtd-validator-0.${NAMESPACE}.svc.cluster.local
rtd-validator-1.${NAMESPACE}.svc.cluster.local
rtd-validator-2.${NAMESPACE}.svc.cluster.local
rtd-validator-3.${NAMESPACE}.svc.cluster.local
```

本手册采用“每个验证者一个 Headless Service + 一个单副本 StatefulSet”的方式。这样每个验证者 Pod 只挂载自己的 Secret，不会在同一个 StatefulSet 中暴露其他验证者私钥。

每个验证者需要以下端口：

| 端口 | 协议 | 用途 |
| --- | --- | --- |
| 8080 | TCP | validator protocol / transaction interface |
| 8081 | TCP | consensus interface |
| 8081 | UDP | Narwhal primary interface |
| 8082 | UDP | Narwhal worker interface |
| 8084 | UDP | P2P state sync |
| 9000 | TCP | JSON-RPC，可选，验证者通常不对公网暴露 |
| 9184 | TCP | Prometheus metrics |
| 1337 | TCP | admin interface |

全节点建议也使用每个全节点独立 Headless Service 的稳定 DNS：

```text
rtd-fullnode-0.${NAMESPACE}.svc.cluster.local
rtd-fullnode-1.${NAMESPACE}.svc.cluster.local
```

全节点常用端口：

| 端口 | 协议 | 用途 |
| --- | --- | --- |
| 8080 | TCP | fullnode network address |
| 8084 | UDP | P2P state sync |
| 9000 | TCP | JSON-RPC，对应用或 Ingress 暴露 |
| 9184 | TCP | Prometheus metrics |

公网测试网注意事项：

- 如果多个验证者跨 Kubernetes 集群、跨云或跨 VPC，StatefulSet 内部 DNS 无法跨集群解析，需要为每个验证者准备公网 DNS，并在 genesis 中写入公网 DNS。
- Kubernetes `Service` 同时暴露 TCP/UDP 时，不同云厂商 LoadBalancer 行为不一致。生产化部署建议给验证者使用固定公网 DNS + 固定 LoadBalancer / NodePort / hostNetwork 方案。
- 单集群测试网最简单，所有节点都使用 `*.svc.cluster.local` 互相发现。

## 4. 准备 Namespace 和 StorageClass

创建命名空间：

```bash
kubectl create namespace "$NAMESPACE"
```

确认集群默认 StorageClass：

```bash
kubectl get storageclass
```

如果没有默认 StorageClass，需要创建云厂商对应的 StorageClass。下面是一个 GKE `pd-ssd` 示例，其他云厂商需要替换 `provisioner` 和参数：

```bash
cat > "$GENESIS_WORKDIR/k8s/storageclass.yaml" <<'YAML'
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: rtd-ssd
allowVolumeExpansion: true
provisioner: pd.csi.storage.gke.io
parameters:
  type: pd-ssd
reclaimPolicy: Retain
volumeBindingMode: WaitForFirstConsumer
YAML

kubectl apply -f "$GENESIS_WORKDIR/k8s/storageclass.yaml"
```

说明：

- `Retain` 可以降低误删 StatefulSet 导致数据卷被删除的风险。
- 测试网初期可以从 `500Gi` 或 `1Ti` 起步，公开压测和长期运行应提前评估增长速度。

## 5. 生成验证者密钥和 genesis

有两种方式可以生成 genesis：

- 方式 A：`rtd genesis --benchmark-ips`。适合单人快速启动一组固定 IPv4 地址的验证者，命令会直接输出 `genesis.blob`、validator 配置和 fullnode 配置。
- 方式 B：`rtd genesis-ceremony`。适合多个独立验证者各自贡献公钥并签名的公开测试网。

公开测试网建议使用方式 B；单团队内部测试可以使用方式 A。

### 5.1 方式 A：快速生成 IP 地址测试网 genesis

先生成一个 genesis config 模板：

```bash
./target/release/rtd genesis \
  --write-config "$GENESIS_WORKDIR/genesis-config.yaml"
```

编辑 `"$GENESIS_WORKDIR/genesis-config.yaml"`，至少确认：

- `parameters.protocol_version` 与当前代码支持的协议版本一致。
- `parameters.chain_start_timestamp_ms` 是计划开网时间的毫秒时间戳。
- `parameters.epoch_duration_ms` 是 epoch 时长，测试网常用 `86400000`。
- `accounts` 包含初始 gas 接收地址和金额。

如果只需要快速生成 4 个创世验证者，可以使用 benchmark IP 功能，它会按固定端口生成验证者配置。这个参数按源码会生成 `/ip4/<addr>/...` multiaddr，所以只适合传 IPv4 地址，不适合传 Kubernetes DNS。

```bash
VALIDATOR_IPS=()
for i in $(seq 0 $((VALIDATOR_COUNT - 1))); do
  VALIDATOR_IPS+=("10.0.0.$((10 + i))")
done

./target/release/rtd genesis \
  --force \
  --working-dir "$GENESIS_WORKDIR/generated" \
  --from-config "$GENESIS_WORKDIR/genesis-config.yaml" \
  --benchmark-ips "$(IFS=,; echo "${VALIDATOR_IPS[*]}")"
```

注意：

- `--benchmark-ips` 会生成确定性 key 和端口，适合开发或内测，不适合作为公开测试网长期使用。
- 本手册后续 Kubernetes 示例使用 DNS 地址，应该使用下一节的 genesis ceremony。
- 如果你想结合自定义 `genesis-config.yaml` 和多个 DNS 验证者，需要在 `validator_config_info` 中显式写入每个验证者的 key 与地址；实际公开测试网建议直接使用 ceremony。

### 5.2 方式 B：genesis ceremony

初始化 ceremony 工作目录：

```bash
export CEREMONY_DIR="$GENESIS_WORKDIR/ceremony"
mkdir -p "$CEREMONY_DIR"

./target/release/rtd genesis-ceremony \
  --path "$CEREMONY_DIR" \
  init
```

每个验证者准备自己的私钥目录：

```bash
for i in $(seq 0 $((VALIDATOR_COUNT - 1))); do
  mkdir -p "$GENESIS_WORKDIR/keys/validator-${i}"
  chmod 700 "$GENESIS_WORKDIR/keys/validator-${i}"
done
```

为每个验证者生成四类 key。`protocol.key` 是 BLS12-381，其余三个是 Ed25519：

```bash
for i in $(seq 0 $((VALIDATOR_COUNT - 1))); do
  keydir="$GENESIS_WORKDIR/keys/validator-${i}"

  (
    cd "$keydir"
    "$RTD_BIN" keytool generate bls12381
    mv bls-*.key protocol.key
    "$RTD_BIN" keytool generate ed25519
    mv 0x*.key account.key
    "$RTD_BIN" keytool generate ed25519
    mv 0x*.key network.key
    "$RTD_BIN" keytool generate ed25519
    mv 0x*.key worker.key
  )

  chmod 600 "$keydir"/*.key
done
```

把验证者加入 ceremony。下面使用 StatefulSet DNS 和固定端口：

```bash
for i in $(seq 0 $((VALIDATOR_COUNT - 1))); do
  host="rtd-validator-${i}.${NAMESPACE}.svc.cluster.local"
  keydir="$GENESIS_WORKDIR/keys/validator-${i}"

  ./target/release/rtd genesis-ceremony \
    --path "$CEREMONY_DIR" \
    add-validator \
    --name "rtd-validator-${i}" \
    --validator-key-file "$keydir/protocol.key" \
    --worker-key-file "$keydir/worker.key" \
    --account-key-file "$keydir/account.key" \
    --network-key-file "$keydir/network.key" \
    --network-address "/dns/${host}/tcp/8080/http" \
    --p2p-address "/dns/${host}/udp/8084" \
    --narwhal-primary-address "/dns/${host}/udp/8081" \
    --narwhal-worker-address "/dns/${host}/udp/8082" \
    --description "RTD testnet validator ${i}" \
    --image-url "" \
    --project-url ""
done
```

验证 ceremony 状态：

```bash
./target/release/rtd genesis-ceremony \
  --path "$CEREMONY_DIR" \
  validate-state

./target/release/rtd genesis-ceremony \
  --path "$CEREMONY_DIR" \
  list-validators
```

构建 unsigned checkpoint：

```bash
./target/release/rtd genesis-ceremony \
  --path "$CEREMONY_DIR" \
  build-unsigned-checkpoint
```

每个验证者签名：

```bash
for i in $(seq 0 $((VALIDATOR_COUNT - 1))); do
  ./target/release/rtd genesis-ceremony \
    --path "$CEREMONY_DIR" \
    verify-and-sign \
    --key-file "$GENESIS_WORKDIR/keys/validator-${i}/protocol.key"
done
```

最终生成 `genesis.blob`：

```bash
./target/release/rtd genesis-ceremony \
  --path "$CEREMONY_DIR" \
  finalize

cp "$CEREMONY_DIR/genesis.blob" "$GENESIS_WORKDIR/genesis.blob"
```

查看 genesis hash：

```bash
shasum -a 256 "$GENESIS_WORKDIR/genesis.blob"
ls -lh "$GENESIS_WORKDIR/genesis.blob"
```

说明：

- `genesis.blob` 必须在所有验证者和全节点上完全一致。
- ceremony 默认会给每个验证者分配并质押默认额度的 RTD。若需要自定义初始 token 分配，需要在 ceremony builder 的 `token-distribution-schedule` 文件中准备符合总供应量约束的 CSV/YAML。内部快速测试通常不需要改。

## 6. 生成节点配置

### 6.1 验证者配置模板

为每个验证者生成 `validator.yaml`：

```bash
for i in $(seq 0 $((VALIDATOR_COUNT - 1))); do
  host="rtd-validator-${i}.${NAMESPACE}.svc.cluster.local"
  out="$GENESIS_WORKDIR/configs/validator-${i}.yaml"

  cat > "$out" <<YAML
protocol-key-pair:
  path: /opt/rtd/key-pairs/protocol.key
worker-key-pair:
  path: /opt/rtd/key-pairs/worker.key
account-key-pair:
  path: /opt/rtd/key-pairs/account.key
network-key-pair:
  path: /opt/rtd/key-pairs/network.key
db-path: /opt/rtd/db/authorities_db
network-address: /dns/${host}/tcp/8080/http
json-rpc-address: 0.0.0.0:9000
metrics-address: 0.0.0.0:9184
admin-interface-port: 1337
consensus-config:
  db-path: /opt/rtd/db/consensus_db
p2p-config:
  listen-address: 0.0.0.0:8084
  external-address: /dns/${host}/udp/8084
  anemo-config:
    max-concurrent-connections: 0
genesis:
  genesis-file-location: /opt/rtd/config/genesis.blob
enable-index-processing: false
authority-store-pruning-config:
  num-latest-epoch-dbs-to-retain: 3
  epoch-db-pruning-period-secs: 3600
  num-epochs-to-retain: 0
  num-epochs-to-retain-for-checkpoints: 2
  max-checkpoints-in-batch: 10
  max-transactions-in-batch: 1000
checkpoint-executor-config:
  checkpoint-execution-max-concurrency: 200
  local-execution-timeout-sec: 30
db-checkpoint-config:
  perform-db-checkpoints-at-epoch-end: false
metrics:
  push-interval-seconds: 60
YAML
done
```

说明：

- `account-key-pair` 对 genesis ceremony 创建的验证者配置很重要，建议显式配置。
- `network-address`、`p2p-config.external-address` 这里使用和 genesis ceremony 相同的稳定 DNS。`rtd-node` 会绑定 `network-address`；如果你的 CNI 或 DNS 不允许 Pod 绑定 Service DNS 解析到的地址，可以把配置文件里的 `network-address` 改成 `/ip4/0.0.0.0/tcp/8080/http`，但不要改 genesis ceremony 中已经写入的对外地址。
- `max-concurrent-connections: 0` 会限制普通全节点同步。若你希望全节点可同步，需要在验证者配置中加入允许的全节点 seed peer，见 6.3。

### 6.2 全节点配置模板

先为每个全节点生成独立 network key。全节点不参与共识，但仍需要 network key 用于 P2P peer id：

```bash
for i in $(seq 0 $((FULLNODE_COUNT - 1))); do
  keydir="$GENESIS_WORKDIR/keys/fullnode-${i}"
  mkdir -p "$keydir"
  chmod 700 "$keydir"
  (
    cd "$keydir"
    "$RTD_BIN" keytool generate ed25519
    mv 0x*.key network.key
  )
  chmod 600 "$keydir/network.key"
done
```

读取验证者的 P2P peer id：

```bash
for i in $(seq 0 $((VALIDATOR_COUNT - 1))); do
  ./target/release/rtd keytool show "$GENESIS_WORKDIR/keys/validator-${i}/network.key"
done
```

输出中的 `peerId` 就是全节点 `seed-peers.peer-id` 应填写的值。

生成全节点配置时，把下面 `<VALIDATOR_N_PEER_ID>` 替换为对应验证者 `network.key` 的 `peerId`：

```bash
for i in $(seq 0 $((FULLNODE_COUNT - 1))); do
  host="rtd-fullnode-${i}.${NAMESPACE}.svc.cluster.local"
  out="$GENESIS_WORKDIR/configs/fullnode-${i}.yaml"

  cat > "$out" <<YAML
network-key-pair:
  path: /opt/rtd/key-pairs/network.key
db-path: /opt/rtd/db/full_node_db
network-address: /dns/${host}/tcp/8080/http
json-rpc-address: 0.0.0.0:9000
metrics-address: 0.0.0.0:9184
admin-interface-port: 1337
enable-index-processing: false
p2p-config:
  listen-address: 0.0.0.0:8084
  external-address: /dns/${host}/udp/8084
  seed-peers:
    - address: /dns/rtd-validator-0.${NAMESPACE}.svc.cluster.local/udp/8084
      peer-id: <VALIDATOR_0_PEER_ID>
    - address: /dns/rtd-validator-1.${NAMESPACE}.svc.cluster.local/udp/8084
      peer-id: <VALIDATOR_1_PEER_ID>
    - address: /dns/rtd-validator-2.${NAMESPACE}.svc.cluster.local/udp/8084
      peer-id: <VALIDATOR_2_PEER_ID>
    - address: /dns/rtd-validator-3.${NAMESPACE}.svc.cluster.local/udp/8084
      peer-id: <VALIDATOR_3_PEER_ID>
genesis:
  genesis-file-location: /opt/rtd/config/genesis.blob
authority-store-pruning-config:
  num-latest-epoch-dbs-to-retain: 3
  epoch-db-pruning-period-secs: 3600
  num-epochs-to-retain: 1
  max-checkpoints-in-batch: 10
  max-transactions-in-batch: 1000
  pruning-run-delay-seconds: 60
checkpoint-executor-config:
  checkpoint-execution-max-concurrency: 200
  local-execution-timeout-sec: 30
YAML
done
```

如果你只有 1 到 3 个验证者，删除多余的 `seed-peers` 条目。

确认所有占位符都已替换：

```bash
if grep -R '<VALIDATOR_' "$GENESIS_WORKDIR/configs"; then
  echo "仍有未替换占位符"
fi
```

如果命令输出任何 `<VALIDATOR_...>`，先修正配置再继续。

### 6.3 允许全节点从验证者同步

如果验证者配置使用了：

```yaml
p2p-config:
  anemo-config:
    max-concurrent-connections: 0
```

则需要把全节点也加入验证者的 `seed-peers`。先读取全节点 peer id：

```bash
for i in $(seq 0 $((FULLNODE_COUNT - 1))); do
  ./target/release/rtd keytool show "$GENESIS_WORKDIR/keys/fullnode-${i}/network.key"
done
```

然后在每个 `validator-N.yaml` 的 `p2p-config` 下加入：

```yaml
  seed-peers:
    - address: /dns/rtd-fullnode-0.rtd-testnet.svc.cluster.local/udp/8084
      peer-id: <FULLNODE_0_PEER_ID>
    - address: /dns/rtd-fullnode-1.rtd-testnet.svc.cluster.local/udp/8084
      peer-id: <FULLNODE_1_PEER_ID>
```

如果你希望测试网初期更宽松，也可以临时移除 `max-concurrent-connections: 0`，但公开环境不建议长期这样运行。

同样确认全节点占位符已替换：

```bash
if grep -R '<FULLNODE_' "$GENESIS_WORKDIR/configs"; then
  echo "仍有未替换占位符"
fi
```

## 7. 创建 Kubernetes Secret 和 ConfigMap

创建 genesis ConfigMap：

```bash
kubectl -n "$NAMESPACE" create configmap rtd-genesis \
  --from-file=genesis.blob="$GENESIS_WORKDIR/genesis.blob" \
  --dry-run=client -o yaml | kubectl apply -f -
```

Kubernetes 单个 ConfigMap 对象默认有约 1MiB 大小限制。当前 RTD/Sui 风格的 genesis blob 通常在几百 KiB 量级，但如果你加入大量初始对象或账户导致超过限制，应改为：

- 用 PVC 或对象存储在 initContainer 中下载 `genesis.blob`。
- 或把 `genesis.blob` 放进一个只含公开 genesis 文件的单独 config 镜像，再用 initContainer 复制到共享卷。
- 不建议把私钥放进镜像；`genesis.blob` 本身不是私钥。

为每个验证者创建配置和密钥 Secret：

```bash
for i in $(seq 0 $((VALIDATOR_COUNT - 1))); do
  kubectl -n "$NAMESPACE" create configmap "rtd-validator-${i}-config" \
    --from-file=validator.yaml="$GENESIS_WORKDIR/configs/validator-${i}.yaml" \
    --dry-run=client -o yaml | kubectl apply -f -

  kubectl -n "$NAMESPACE" create secret generic "rtd-validator-${i}-keys" \
    --from-file=protocol.key="$GENESIS_WORKDIR/keys/validator-${i}/protocol.key" \
    --from-file=account.key="$GENESIS_WORKDIR/keys/validator-${i}/account.key" \
    --from-file=network.key="$GENESIS_WORKDIR/keys/validator-${i}/network.key" \
    --from-file=worker.key="$GENESIS_WORKDIR/keys/validator-${i}/worker.key" \
    --dry-run=client -o yaml | kubectl apply -f -
done
```

为每个全节点创建配置和密钥 Secret：

```bash
for i in $(seq 0 $((FULLNODE_COUNT - 1))); do
  kubectl -n "$NAMESPACE" create configmap "rtd-fullnode-${i}-config" \
    --from-file=fullnode.yaml="$GENESIS_WORKDIR/configs/fullnode-${i}.yaml" \
    --dry-run=client -o yaml | kubectl apply -f -

  kubectl -n "$NAMESPACE" create secret generic "rtd-fullnode-${i}-keys" \
    --from-file=network.key="$GENESIS_WORKDIR/keys/fullnode-${i}/network.key" \
    --dry-run=client -o yaml | kubectl apply -f -
done
```

检查：

```bash
kubectl -n "$NAMESPACE" get configmap
kubectl -n "$NAMESPACE" get secret
```

## 8. 部署创世验证者

这里为每个创世验证者创建一个 Headless Service 和一个单副本 StatefulSet。这样每个验证者只挂载自己的 ConfigMap 和 Secret，避免一个 Pod 能读取所有验证者私钥。

生成验证者 Headless Service 和 StatefulSet：

```bash
cat > "$GENESIS_WORKDIR/k8s/validators.yaml" <<'YAML'
# Generated by doc/Kubernetes-Deploy/README.md
YAML

for i in $(seq 0 $((VALIDATOR_COUNT - 1))); do
  cat >> "$GENESIS_WORKDIR/k8s/validators.yaml" <<YAML
---
apiVersion: v1
kind: Service
metadata:
  name: rtd-validator-${i}
  namespace: ${NAMESPACE}
  labels:
    app: rtd-validator
    rtd.io/node-index: "${i}"
spec:
  clusterIP: None
  publishNotReadyAddresses: true
  selector:
    app: rtd-validator
    rtd.io/node-index: "${i}"
  ports:
    - name: validator-grpc
      port: 8080
      targetPort: 8080
      protocol: TCP
    - name: consensus-tcp
      port: 8081
      targetPort: 8081
      protocol: TCP
    - name: consensus-udp
      port: 8081
      targetPort: 8081
      protocol: UDP
    - name: narwhal-worker
      port: 8082
      targetPort: 8082
      protocol: UDP
    - name: p2p
      port: 8084
      targetPort: 8084
      protocol: UDP
    - name: metrics
      port: 9184
      targetPort: 9184
      protocol: TCP
---
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: rtd-validator-${i}
  namespace: ${NAMESPACE}
  labels:
    app: rtd-validator
    rtd.io/node-index: "${i}"
spec:
  serviceName: rtd-validator-${i}
  replicas: 1
  persistentVolumeClaimRetentionPolicy:
    whenDeleted: Retain
    whenScaled: Retain
  selector:
    matchLabels:
      app: rtd-validator
      rtd.io/node-index: "${i}"
  template:
    metadata:
      labels:
        app: rtd-validator
        rtd.io/node-index: "${i}"
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9184"
        prometheus.io/path: /metrics
    spec:
      terminationGracePeriodSeconds: 180
      containers:
        - name: rtd-node
          image: ${RTD_NODE_IMAGE}
          imagePullPolicy: IfNotPresent
          command:
            - /opt/rtd/bin/rtd-node
            - --config-path
            - /opt/rtd/config/validator.yaml
          env:
            - name: RUST_BACKTRACE
              value: "1"
            - name: RUST_LOG
              value: info,rtd_core=debug,rtd_network=debug,consensus=debug,jsonrpsee=error
          ports:
            - name: validator-grpc
              containerPort: 8080
              protocol: TCP
            - name: consensus-tcp
              containerPort: 8081
              protocol: TCP
            - name: consensus-udp
              containerPort: 8081
              protocol: UDP
            - name: narwhal-worker
              containerPort: 8082
              protocol: UDP
            - name: p2p
              containerPort: 8084
              protocol: UDP
            - name: rpc
              containerPort: 9000
              protocol: TCP
            - name: metrics
              containerPort: 9184
              protocol: TCP
          resources:
            requests:
              cpu: "8"
              memory: 32Gi
            limits:
              cpu: "24"
              memory: 128Gi
          volumeMounts:
            - name: data
              mountPath: /opt/rtd/db
            - name: validator-config
              mountPath: /opt/rtd/config/validator.yaml
              subPath: validator.yaml
              readOnly: true
            - name: validator-keys
              mountPath: /opt/rtd/key-pairs
              readOnly: true
            - name: genesis
              mountPath: /opt/rtd/config/genesis.blob
              subPath: genesis.blob
              readOnly: true
      volumes:
        - name: genesis
          configMap:
            name: rtd-genesis
        - name: validator-config
          configMap:
            name: rtd-validator-${i}-config
        - name: validator-keys
          secret:
            secretName: rtd-validator-${i}-keys
            defaultMode: 0400
  volumeClaimTemplates:
    - metadata:
        name: data
      spec:
        accessModes:
          - ReadWriteOnce
        storageClassName: rtd-ssd
        resources:
          requests:
            storage: 1Ti
YAML
done

kubectl apply -f "$GENESIS_WORKDIR/k8s/validators.yaml"
```

查看启动：

```bash
kubectl -n "$NAMESPACE" get pod -l app=rtd-validator -w
kubectl -n "$NAMESPACE" logs -f rtd-validator-0-0 -c rtd-node
```

等待所有验证者 Running：

```bash
for i in $(seq 0 $((VALIDATOR_COUNT - 1))); do
  kubectl -n "$NAMESPACE" rollout status "statefulset/rtd-validator-${i}" --timeout=20m
done
```

## 9. 部署全节点

全节点也按“一个全节点一个 Headless Service + 一个单副本 StatefulSet”生成。额外创建一个 `rtd-fullnode-rpc` ClusterIP Service，统一负载到所有全节点的 JSON-RPC 端口，再按需要接 Ingress 或 LoadBalancer。

```bash
cat > "$GENESIS_WORKDIR/k8s/fullnodes.yaml" <<YAML
apiVersion: v1
kind: Service
metadata:
  name: rtd-fullnode-rpc
  namespace: ${NAMESPACE}
spec:
  type: ClusterIP
  selector:
    app: rtd-fullnode
  ports:
    - name: rpc
      port: 9000
      targetPort: 9000
      protocol: TCP
YAML

for i in $(seq 0 $((FULLNODE_COUNT - 1))); do
  cat >> "$GENESIS_WORKDIR/k8s/fullnodes.yaml" <<YAML
---
apiVersion: v1
kind: Service
metadata:
  name: rtd-fullnode-${i}
  namespace: ${NAMESPACE}
  labels:
    app: rtd-fullnode
    rtd.io/node-index: "${i}"
spec:
  clusterIP: None
  publishNotReadyAddresses: true
  selector:
    app: rtd-fullnode
    rtd.io/node-index: "${i}"
  ports:
    - name: network
      port: 8080
      targetPort: 8080
      protocol: TCP
    - name: p2p
      port: 8084
      targetPort: 8084
      protocol: UDP
    - name: rpc
      port: 9000
      targetPort: 9000
      protocol: TCP
    - name: metrics
      port: 9184
      targetPort: 9184
      protocol: TCP
---
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: rtd-fullnode-${i}
  namespace: ${NAMESPACE}
  labels:
    app: rtd-fullnode
    rtd.io/node-index: "${i}"
spec:
  serviceName: rtd-fullnode-${i}
  replicas: 1
  persistentVolumeClaimRetentionPolicy:
    whenDeleted: Retain
    whenScaled: Retain
  selector:
    matchLabels:
      app: rtd-fullnode
      rtd.io/node-index: "${i}"
  template:
    metadata:
      labels:
        app: rtd-fullnode
        rtd.io/node-index: "${i}"
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9184"
        prometheus.io/path: /metrics
    spec:
      terminationGracePeriodSeconds: 180
      containers:
        - name: rtd-node
          image: ${RTD_NODE_IMAGE}
          imagePullPolicy: IfNotPresent
          command:
            - /opt/rtd/bin/rtd-node
            - --config-path
            - /opt/rtd/config/fullnode.yaml
          env:
            - name: RUST_BACKTRACE
              value: "1"
            - name: RUST_LOG
              value: info,rtd_core=debug,rtd_network=debug,jsonrpsee=error
          ports:
            - name: network
              containerPort: 8080
              protocol: TCP
            - name: p2p
              containerPort: 8084
              protocol: UDP
            - name: rpc
              containerPort: 9000
              protocol: TCP
            - name: metrics
              containerPort: 9184
              protocol: TCP
          resources:
            requests:
              cpu: "4"
              memory: 16Gi
            limits:
              cpu: "16"
              memory: 64Gi
          volumeMounts:
            - name: data
              mountPath: /opt/rtd/db
            - name: fullnode-config
              mountPath: /opt/rtd/config/fullnode.yaml
              subPath: fullnode.yaml
              readOnly: true
            - name: fullnode-keys
              mountPath: /opt/rtd/key-pairs
              readOnly: true
            - name: genesis
              mountPath: /opt/rtd/config/genesis.blob
              subPath: genesis.blob
              readOnly: true
      volumes:
        - name: genesis
          configMap:
            name: rtd-genesis
        - name: fullnode-config
          configMap:
            name: rtd-fullnode-${i}-config
        - name: fullnode-keys
          secret:
            secretName: rtd-fullnode-${i}-keys
            defaultMode: 0400
  volumeClaimTemplates:
    - metadata:
        name: data
      spec:
        accessModes:
          - ReadWriteOnce
        storageClassName: rtd-ssd
        resources:
          requests:
            storage: 1Ti
YAML
done

kubectl apply -f "$GENESIS_WORKDIR/k8s/fullnodes.yaml"
```

查看启动：

```bash
for i in $(seq 0 $((FULLNODE_COUNT - 1))); do
  kubectl -n "$NAMESPACE" rollout status "statefulset/rtd-fullnode-${i}" --timeout=20m
done
kubectl -n "$NAMESPACE" logs -f rtd-fullnode-0-0 -c rtd-node
```

集群内 RPC 地址：

```text
http://rtd-fullnode-rpc.rtd-testnet.svc.cluster.local:9000
```

本地临时访问：

```bash
kubectl -n "$NAMESPACE" port-forward svc/rtd-fullnode-rpc 9000:9000
```

然后在另一个终端请求 RPC：

```bash
curl -s -X POST http://127.0.0.1:9000 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"rtd_getLatestCheckpointSequenceNumber","params":[]}'
```

## 10. 互相发现和连通性检查

### 10.1 DNS 检查

进入任意 Pod 检查验证者 DNS：

```bash
kubectl -n "$NAMESPACE" exec -it rtd-validator-0-0 -- sh
```

在容器内执行：

```bash
getent hosts rtd-validator-0.rtd-testnet.svc.cluster.local
getent hosts rtd-validator-1.rtd-testnet.svc.cluster.local
getent hosts rtd-fullnode-0.rtd-testnet.svc.cluster.local
```

如果 `getent` 不存在，可以临时起一个 debug Pod：

```bash
kubectl -n "$NAMESPACE" run netshoot --rm -it \
  --image=nicolaka/netshoot \
  --restart=Never -- bash
```

### 10.2 端口检查

TCP 端口可以用 `nc`：

```bash
nc -vz rtd-validator-0.rtd-testnet.svc.cluster.local 8080
nc -vz rtd-validator-0.rtd-testnet.svc.cluster.local 8081
nc -vz rtd-fullnode-rpc.rtd-testnet.svc.cluster.local 9000
```

UDP 端口更依赖应用日志判断。重点看节点日志中是否出现 peer connection、state sync、checkpoint sync 相关信息：

```bash
kubectl -n "$NAMESPACE" logs rtd-validator-0-0 -c rtd-node --tail=300 | \
  egrep -i 'peer|state sync|checkpoint|consensus|narwhal'

kubectl -n "$NAMESPACE" logs rtd-fullnode-0-0 -c rtd-node --tail=300 | \
  egrep -i 'peer|state sync|checkpoint'
```

### 10.3 检查 checkpoint 推进

本地 port-forward 后：

```bash
curl -s -X POST http://127.0.0.1:9000 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"rtd_getLatestCheckpointSequenceNumber","params":[]}'
```

间隔 30 秒重复一次。如果 checkpoint number 在增长，说明共识和 checkpoint 执行基本正常。

也可以用 CLI：

```bash
./target/release/rtd --client.config "$GENESIS_WORKDIR/client.yaml" client envs
```

如果还没有 client config，可以初始化一个指向本地转发 RPC 的配置：

```bash
mkdir -p "$GENESIS_WORKDIR/client"
RTD_CONFIG_DIR="$GENESIS_WORKDIR/client" ./target/release/rtd client envs
```

再按 CLI 提示添加 `http://127.0.0.1:9000` 作为环境。也可以直接编辑生成的 `client.yaml`，把 `envs` 中的 RPC 地址指向该 URL。

## 11. 对公网暴露 RPC

最小化方式是把 `rtd-fullnode-rpc` 改成 LoadBalancer：

```bash
kubectl -n "$NAMESPACE" patch service rtd-fullnode-rpc \
  -p '{"spec":{"type":"LoadBalancer"}}'
```

查看外部地址：

```bash
kubectl -n "$NAMESPACE" get svc rtd-fullnode-rpc
```

生产建议：

- 使用 Ingress/Gateway API 加 TLS。
- 在入口层做 rate limit、body size limit、IP allow/deny list。
- RPC 对公网暴露全节点，不直接暴露验证者 JSON-RPC。
- 将内部服务名 `http://rtd-fullnode-rpc.${NAMESPACE}.svc.cluster.local:9000` 提供给 indexer、faucet、explorer 等内部服务。

## 12. 镜像升级

升级前原则：

- 不删除 PVC。
- 新镜像必须和链上协议版本兼容。
- 验证者滚动升级时不要同时重启所有节点，除非这是短期测试网且可以接受停机。

推送新镜像：

```bash
export IMAGE_TAG=$(git rev-parse --short=12 HEAD)
export RTD_NODE_IMAGE=${REGISTRY}/rtd-node:${IMAGE_TAG}

docker build \
  --platform linux/amd64 \
  -f docker/rtd-node/Dockerfile \
  --build-arg PROFILE=release \
  -t "$RTD_NODE_IMAGE" \
  .
docker push "$RTD_NODE_IMAGE"
```

升级全节点：

```bash
for i in $(seq 0 $((FULLNODE_COUNT - 1))); do
  kubectl -n "$NAMESPACE" set image "statefulset/rtd-fullnode-${i}" \
    rtd-node="$RTD_NODE_IMAGE"
  kubectl -n "$NAMESPACE" rollout status "statefulset/rtd-fullnode-${i}" --timeout=30m
done
```

升级验证者：

```bash
for i in $(seq 0 $((VALIDATOR_COUNT - 1))); do
  kubectl -n "$NAMESPACE" set image "statefulset/rtd-validator-${i}" \
    rtd-node="$RTD_NODE_IMAGE"
  kubectl -n "$NAMESPACE" rollout status "statefulset/rtd-validator-${i}" --timeout=30m
done
```

如果需要更保守的验证者升级，逐个 Pod 删除，让 StatefulSet 按新模板重建：

```bash
kubectl -n "$NAMESPACE" delete pod rtd-validator-0-0
kubectl -n "$NAMESPACE" wait --for=condition=Ready pod/rtd-validator-0-0 --timeout=20m

kubectl -n "$NAMESPACE" delete pod rtd-validator-1-0
kubectl -n "$NAMESPACE" wait --for=condition=Ready pod/rtd-validator-1-0 --timeout=20m
```

## 13. 常见故障排查

### 13.1 节点启动时报 key 文件错误

检查 Secret 是否包含正确文件：

```bash
kubectl -n "$NAMESPACE" describe secret rtd-validator-0-keys
kubectl -n "$NAMESPACE" exec rtd-validator-0-0 -- ls -l /opt/rtd/key-pairs
```

验证本地 key 文件格式：

```bash
./target/release/rtd keytool show "$GENESIS_WORKDIR/keys/validator-0/protocol.key"
./target/release/rtd keytool show "$GENESIS_WORKDIR/keys/validator-0/network.key"
```

### 13.2 验证者无法互连

重点检查：

- genesis ceremony 中的地址是否和 `validator.yaml` 中的地址一致。
- StatefulSet Pod DNS 是否可解析。
- `8080/tcp`、`8081/tcp`、`8081/udp`、`8082/udp`、`8084/udp` 是否被 NetworkPolicy、云安全组或防火墙拦截。
- 如果跨集群，不能使用 `svc.cluster.local`，必须使用公网 DNS 或跨集群可解析 DNS。

查看日志：

```bash
kubectl -n "$NAMESPACE" logs rtd-validator-0-0 -c rtd-node --tail=500 | \
  egrep -i 'error|warn|consensus|narwhal|peer|connection'
```

### 13.3 全节点不同步

重点检查：

- 全节点 `p2p-config.seed-peers` 是否填了验证者的 `peer-id` 和 P2P 地址。
- 验证者是否配置了 `max-concurrent-connections: 0`，但没有把全节点加入验证者 `seed-peers`。
- 全节点是否使用了和验证者完全一致的 `genesis.blob`。

检查 genesis hash：

```bash
kubectl -n "$NAMESPACE" exec rtd-validator-0-0 -- sh -c 'sha256sum /opt/rtd/config/genesis.blob || shasum -a 256 /opt/rtd/config/genesis.blob'
kubectl -n "$NAMESPACE" exec rtd-fullnode-0-0 -- sh -c 'sha256sum /opt/rtd/config/genesis.blob || shasum -a 256 /opt/rtd/config/genesis.blob'
```

### 13.4 ConfigMap 更新后节点没变化

Kubernetes 更新 ConfigMap 后，使用 `subPath` 挂载的文件不会自动热更新。需要重启 Pod：

```bash
kubectl -n "$NAMESPACE" delete pod rtd-fullnode-0-0
kubectl -n "$NAMESPACE" delete pod rtd-validator-0-0
```

验证者创世地址、公钥、committee 等进入了 genesis 的内容，不能通过改 ConfigMap 直接改变已经启动的创世事实。需要重新 genesis 或通过链上治理/validator operation 更新下一 epoch 元数据。

### 13.5 误删 Pod 后数据还在吗

如果使用 StatefulSet + PVC，删除 Pod 不会删除数据卷：

```bash
kubectl -n "$NAMESPACE" get pvc
kubectl -n "$NAMESPACE" delete pod rtd-validator-0-0
```

不要随意删除 PVC。删除 PVC 等价于清空节点数据库，节点需要从 genesis 或 snapshot 重新同步。

## 14. 清理测试网

停止工作负载但保留 PVC：

```bash
for i in $(seq 0 $((VALIDATOR_COUNT - 1))); do
  kubectl -n "$NAMESPACE" scale "statefulset/rtd-validator-${i}" --replicas=0
done

for i in $(seq 0 $((FULLNODE_COUNT - 1))); do
  kubectl -n "$NAMESPACE" scale "statefulset/rtd-fullnode-${i}" --replicas=0
done
```

删除工作负载和服务：

```bash
kubectl delete -f "$GENESIS_WORKDIR/k8s/fullnodes.yaml"
kubectl delete -f "$GENESIS_WORKDIR/k8s/validators.yaml"
```

确认 PVC：

```bash
kubectl -n "$NAMESPACE" get pvc
```

如果确定不要数据，再删除 PVC：

```bash
for i in $(seq 0 $((VALIDATOR_COUNT - 1))); do
  kubectl -n "$NAMESPACE" delete pvc "data-rtd-validator-${i}-0"
done

for i in $(seq 0 $((FULLNODE_COUNT - 1))); do
  kubectl -n "$NAMESPACE" delete pvc "data-rtd-fullnode-${i}-0"
done
```

删除命名空间会删除其中所有资源：

```bash
kubectl delete namespace "$NAMESPACE"
```

## 15. 生产化前检查清单

- 使用 Linux CI 构建并保存镜像 digest。
- 私钥只在 Secret 管理系统和 Kubernetes Secret 中流转，不进入 Git 和镜像。
- 每个验证者使用独立节点池、独立磁盘、反亲和调度。
- 验证者端口和全节点 RPC 入口有清晰的安全组、NetworkPolicy 和限流策略。
- Prometheus 已采集 `9184` metrics，日志已接入集中式日志系统。
- PVC `reclaimPolicy` 和备份策略已确认。
- 镜像升级、回滚、单节点重启、全节点扩容都在预发环境演练过。
- 对外公开的 RPC 使用全节点，不直接暴露验证者 RPC。
