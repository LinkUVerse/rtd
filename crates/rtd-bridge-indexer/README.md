## Overview

Rtd Bridge Indexer is a binary that scans Rtd Bridge transactions on Rtd and Ethereum networks, and indexes the processed data for further use.

## Get Binary

```bash
cargo build --bin bridge-indexer --release
```

Build the indexer from source. No public RTD Bridge Indexer image is assumed.

## Run Binary

```
bridge-indexer --config-path config.yaml
```


## Config

RTD has no public Mainnet or published Bridge deployment. Replace every placeholder
below with values from a network and bridge contract that you operate before running
the indexer. The upstream Sui deployment addresses and checkpoint numbers do not
identify RTD deployments.

```yaml
---
remote_store_url: "<RTD_CHECKPOINT_STORE_URL>"
eth_rpc_url: "<ETH_RPC_URL>"
rtd_rpc_url: "<RTD_RPC_URL>"

concurrency: 500
checkpoints_path: "<CHECKPOINTS_PATH>"

eth_rtd_bridge_contract_address: "<DEPLOYED_BRIDGE_CONTRACT_ADDRESS>"
metric_port: <METRICS_PORT>

rtd_bridge_genesis_checkpoint: <RTD_BRIDGE_GENESIS_CHECKPOINT>
eth_bridge_genesis_block: <ETH_BRIDGE_GENESIS_BLOCK>

eth_ws_url: "<ETH_WEBSOCKET_URL>"

```
