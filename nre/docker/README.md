# Run Rtd Node using Docker Compose

Tested using:
- ubuntu 20.04 (linux/amd64) on bare metal
- ubuntu 22.04 (linux/amd64) on bare metal

## Prerequisites and Setup

1. Confirm you have either [Docker Engine](https://docs.docker.com/engine/install/) or [Docker Desktop](https://docs.docker.com/desktop/install/linux-install/) installed, as well as [Docker Compose](https://github.com/docker/compose#linux).

2. Update [validator.yaml](../config/validator.yaml) and place it in the same directory as `docker-compose.yaml`.

Add the paths to your private keys to validator.yaml. If you chose to put them in `/opt/rtd/key-pairs`, you can use the following example: 

```
protocol-key-pair:
  path: /opt/rtd/key-pairs/protocol.key
worker-key-pair: 
  path: /opt/rtd/key-pairs/worker.key
network-key-pair: 
  path: /opt/rtd/key-pairs/network.key
```

3. Place `genesis.blob` in the same directory as `docker-compose.yaml`. (available post genesis ceremony)

Build the node image from this RTD checkout before starting the validator:

```shell
docker build -f docker/rtd-node/Dockerfile -t rtd-node:local .
export RTD_NODE_IMAGE=rtd-node:local
```

Run the build command from the repository root. The source fork does not provide a verified public validator image or network genesis. Use only the genesis, private keys, and configuration from your own RTD ceremony.

## Connectivity

You may need to explicitly open the ports outlined in [Rtd for Node Operators](../rtd_for_node_operators.md#connectivity) for the required Rtd Node connectivity.

## Start the node

Start Rtd Node in detached mode:

`docker compose up -d`

## Logs

By default, logs are stored at `/var/lib/docker/containers/[container-id]/[container-id]-json.log`.

- View and follow

```shell
docker compose logs -f validator
```

- By default all logs are output, limit this using `--since`

```shell
docker logs --since 10m -f validator
```

## Storage

- What is the size of the local Rtd database?

```shell
# the compose file mounts this host directory directly
sudo du -sh /opt/rtd/db
```

- Delete the local Rtd databases (volume)

```shell
docker compose down -v
```

## Updates

- **DO NOT** delete the Rtd databases

1. Stop docker compose

```shell
docker compose down
```

2. Build and select the new RTD image from the intended source revision:

```
docker build -f docker/rtd-node/Dockerfile -t rtd-node:<RTD_COMMIT> .
export RTD_NODE_IMAGE=rtd-node:<RTD_COMMIT>
```

3. Start docker compose in detached mode:

```shell
docker compose up -d
```
