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

## Connectivity

You may need to explicitly open the ports outlined in [Rtd for Node Operators](../rtd_for_node_operators.md#connectivity) for the required Rtd Node connectivity.

## Start the node

Start Rtd Node in detached mode:

`sudo docker compose up -d`

## Logs

By default, logs are stored at `/var/lib/docker/containers/[container-id]/[container-id]-json.log`.

- View and follow

```shell
sudo docker compose logs -f validator
```

- By default all logs are output, limit this using `--since`

```shell
sudo docker logs --since 10m -f validator
```

## Storage

- What is the size of the local Rtd database?

```shell
# get the volume location on disk
sudo docker volume inspect docker_rtddb
# get the size of the volume on disk
sudo du -sh /var/lib/docker/volumes/docker_rtddb/_data
```

- Delete the local Rtd databases (volume)

```shell
sudo docker-compose down -v
```

## Updates

- **DO NOT** delete the Rtd databases

1. Stop docker compose

```shell
sudo docker compose down
```

2. Update docker-compose.yaml to reference the new image

```
-    image: linku/rtd-node:<OLD_RTD_SHA>
+    image: linku/rtd-node:<NEW_RTD_SHA>
```

3. Start docker compose in detached mode:

```shell
sudo docker compose up -d
```
