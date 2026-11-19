# Run a Rtd Node using Systemd

Tested using:
- Ubuntu 20.04 (linux/amd64) on bare metal
- Ubuntu 22.04 (linux/amd64) on bare metal

## Prerequisites and Setup

1. Add a `rtd` user and the `/opt/rtd` directories

```shell
sudo useradd rtd
sudo mkdir -p /opt/rtd/bin
sudo mkdir -p /opt/rtd/config
sudo mkdir -p /opt/rtd/db
sudo mkdir -p /opt/rtd/key-pairs
sudo chown -R rtd:rtd /opt/rtd
```

2. Install the Rtd Node (rtd-node) binary, two options:
    
- Pre-built binary stored in Amazon S3:
        
```shell
wget https://releases.rtd.io/$RTD_SHA/rtd-node
chmod +x rtd-node
sudo mv rtd-node /opt/rtd/bin
```

- Build from source:

```shell
git clone https://github.com/LinkUVerse/rtd.git && cd rtd
git checkout $RTD_SHA
cargo build --release --bin rtd-node
mv ./target/release/rtd-node /opt/rtd/bin/rtd-node
```

3. Copy your key-pairs into `/opt/rtd/key-pairs/` 

If generated during the Genesis ceremony these will be at `RtdExternal.git/rtd-testnet-wave3/genesis/key-pairs/`

Make sure when you copy them they retain `rtd` user permissions. To be safe you can re-run: `sudo chown -R rtd:rtd /opt/rtd`

4. Update the node configuration file and place it in the `/opt/rtd/config/` directory.

Add the paths to your private keys to validator.yaml. If you chose to put them in `/opt/rtd/key-pairs`, you can use the following example: 

```
protocol-key-pair: 
  path: /opt/rtd/key-pairs/protocol.key
worker-key-pair: 
  path: /opt/rtd/key-pairs/worker.key
network-key-pair: 
  path: /opt/rtd/key-pairs/network.key
```

5. Place genesis.blob in `/opt/rtd/config/` (should be available after the Genesis ceremony)

6. Copy the rtd-node systemd service unit file 

File: [rtd-node.service](./rtd-node.service)

Copy the file to `/etc/systemd/system/rtd-node.service`.

7. Reload systemd with this new service unit file, run:

```shell
sudo systemctl daemon-reload
```

8. Enable the new service with systemd

```shell
sudo systemctl enable rtd-node.service
```

## Connectivity

You may need to explicitly open the ports outlined in [Rtd for Node Operators](../rtd_for_node_operators.md#connectivity) for the required Rtd Node connectivity.

## Start the node

Start the Validator:

```shell
sudo systemctl start rtd-node
```

Check that the node is up and running:

```shell
sudo systemctl status rtd-node
```

Follow the logs with:

```shell
journalctl -u rtd-node -f
```

## Updates

When an update is required to the Rtd Node software the following procedure can be used. It is highly **unlikely** that you will want to restart with a clean database.

- assumes rtd-node lives in `/opt/rtd/bin/`
- assumes systemd service is named rtd-node
- **DO NOT** delete the Rtd databases

1. Stop rtd-node systemd service

```
sudo systemctl stop rtd-node
```

2. Fetch the new rtd-node binary

```shell
wget https://releases.rtd.io/${RTD_SHA}/rtd-node
```

3. Update and move the new binary:

```
chmod +x rtd-node
sudo mv rtd-node /opt/rtd/bin/
```

4. start rtd-node systemd service

```
sudo systemctl start rtd-node
```
