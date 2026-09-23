# RTD 本地 fullnode checkpoint 追平检查命令

日期：2026-07-09

本文档说明如何判断本地 `rtd start` 启动后的 RPC fullnode 是否已经追平 validator 启动时的 checkpoint 高度。

## 背景

本轮问题中，validator DB 已经保留较新的链上状态，但 RPC fullnode 可能因为新建或落后的 fullnode DB 仍在追赶 checkpoint。

如果 fullnode 未追平，RPC 读路径可能返回旧 object ref，例如旧 gas coin version。客户端用旧 ref 签名后，validator 会按自身 live object 状态拒绝交易，典型错误是：

```text
ObjectVersionUnavailableForConsumption
```

因此，本地重启后不要只检查 RPC 端口可连，还要检查 fullnode checkpoint 是否追到 validator 启动高度。

## 查看当前 RPC fullnode checkpoint

默认本地 RPC 端口为 `9000`：

```bash
curl --silent --location --request POST 'http://127.0.0.1:9000' \
  --header 'Content-Type: application/json' \
  --data-raw '{"jsonrpc":"2.0","id":1,"method":"rtd_getLatestCheckpointSequenceNumber","params":[]}'
```

返回示例：

```json
{"jsonrpc":"2.0","result":"1107355","id":1}
```

这里的 `result` 就是当前 RPC fullnode 已执行或可见的最新 checkpoint sequence number。

## 查看 validator 启动时 checkpoint

validator 启动高度目前主要从本地节点日志中提取。先找到本次 `rtd start` 的日志文件；如果使用 `toggle_local_rtd.sh` 启动，通常脚本会把日志写到它配置的 local deploy 日志路径。

可以用下面命令在日志中搜索启动 checkpoint 相关记录：

```bash
grep -E 'highest checkpoint|highest_checkpoint|checkpoint.*startup|startup.*checkpoint|Starting.*checkpoint|validator.*checkpoint' /path/to/rtd.log | tail -50
```

如果日志路径不确定，可以先在常见目录中搜索：

```bash
find /Users/changzechuan/WenchuanProjects/RTD-Blockchain -name '*.log' -mtime -1 -print
```

然后对候选日志执行：

```bash
grep -E 'highest checkpoint|highest_checkpoint|checkpoint.*startup|startup.*checkpoint|Starting.*checkpoint|validator.*checkpoint' <候选日志文件> | tail -50
```

本轮排查中记录过的 validator 启动 checkpoint 示例：

```text
1209471
```

## 判断是否追平

比较两个数字：

- `fullnode_checkpoint`：`rtd_getLatestCheckpointSequenceNumber` 返回值。
- `validator_startup_checkpoint`：日志中提取到的 validator 启动高度。

判断规则：

```text
fullnode_checkpoint >= validator_startup_checkpoint
```

满足该条件时，说明 RPC fullnode 至少追到了 validator 本次启动时已有的 checkpoint 高度。

不满足时，说明 RPC fullnode 仍落后。此时不要用该 RPC 返回的 gas coin ref 构造转账或 Move 调用交易，否则可能继续签出旧版本 object ref。

## 一次性轮询命令

如果已经知道 validator 启动高度，例如 `1209471`，可以直接轮询等待：

```bash
TARGET_CHECKPOINT=1209471

while true; do
  CURRENT=$(
    curl --silent --location --request POST 'http://127.0.0.1:9000' \
      --header 'Content-Type: application/json' \
      --data-raw '{"jsonrpc":"2.0","id":1,"method":"rtd_getLatestCheckpointSequenceNumber","params":[]}' \
      | sed -E 's/.*"result":"?([0-9]+)"?.*/\1/'
  )

  date '+%Y-%m-%d %H:%M:%S'
  echo "fullnode_checkpoint=${CURRENT} target=${TARGET_CHECKPOINT}"

  if [ "${CURRENT}" -ge "${TARGET_CHECKPOINT}" ]; then
    echo "fullnode checkpoint caught up"
    break
  fi

  sleep 5
done
```

## 同时检查 gas object 是否追到最新版本

checkpoint 追平后，还应检查具体 gas object 返回的 version/digest 是否与最近成功交易 effects 中的 mutated gas object 一致。

本轮问题中的 gas object：

```text
0x4a3ced14c55e220d6ffeefd39111ce80df6753c4ef40947aedebc8c26a8fa178
```

查询命令：

```bash
curl --silent --location --request POST 'http://127.0.0.1:9000' \
  --header 'Content-Type: application/json' \
  --data-raw '{"jsonrpc":"2.0","id":1,"method":"rtd_getObject","params":["0x4a3ced14c55e220d6ffeefd39111ce80df6753c4ef40947aedebc8c26a8fa178",{"showContent":false,"showPreviousTransaction":true}]}'
```

如果最近成功交易 effects 显示 gas object 已 mutated 到 version `106`，但 `rtd_getObject` 仍返回 version `105`，说明 RPC fullnode 对该 object 仍未追上，继续发交易仍有复现风险。

## 与重启脚本的关系

`toggle_local_rtd.sh` 已增加等待逻辑：

```bash
WAIT_FULLNODE_CATCHUP_TO_VALIDATOR=1
```

默认情况下，脚本应在 RPC 可连后继续等待 fullnode checkpoint 追到 validator 启动高度，再报告 ready。

如需临时跳过该等待逻辑用于调试，可以显式设置：

```bash
WAIT_FULLNODE_CATCHUP_TO_VALIDATOR=0 toggle_local_rtd.sh
```

跳过后，必须手动执行本文档中的 checkpoint 和 gas object 检查，再决定是否发送交易。
