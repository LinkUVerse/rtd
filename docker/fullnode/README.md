# Local RTD full node container template

This Compose template needs an RTD image built from this source checkout, a genesis blob from the **same RTD network**, and that network's full node configuration. The source fork does not publish a verified Mainnet or Testnet image, public genesis download, or seed peer list.

From the repository root, build the image and prepare local files:

```sh
docker build -f docker/rtd-node/Dockerfile -t rtd-node:local .
cp crates/rtd-config/data/fullnode-template.yaml docker/fullnode/fullnode-template.yaml
cp /path/to/your/rtd-network/genesis.blob docker/fullnode/genesis.blob
```

Review `docker/fullnode/fullnode-template.yaml` and set the peer list, network address, and paths for the RTD network that produced the genesis blob. A standalone node cannot join a network without the correct genesis and reachable peers. The local development network started by `rtd start` is a separate workflow; see [local network setup](../../docs/content/getting-started/onboarding/local-network.mdx).

Start the container only after the image, genesis and config are ready:

```sh
cd docker/fullnode
RTD_NODE_IMAGE=rtd-node:local docker compose up -d
RTD_NODE_IMAGE=rtd-node:local docker compose logs -f fullnode
```

Verify health through the endpoint configured in your full node file, the node logs, and a successful request to that exact network. JSON-RPC may be available in a local development configuration; future RTD Mainnet will not expose JSON-RPC. Stop with `RTD_NODE_IMAGE=rtd-node:local docker compose down`.
