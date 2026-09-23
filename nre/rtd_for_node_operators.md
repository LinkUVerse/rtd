# Rtd for Node Operators

## Overview

This document is focused on running the Rtd Node software as a Validator.

## Contents

- [Requirements](#requirements)
- [Deployment](#deployment)
- [Configuration](#configuration)
- [Connectivity](#connectivity)
- [Storage](storage.md)
- [Key Management](#key-management)
- [Monitoring](#monitoring)
  - [Logs](#logs)
  - [Metrics](#metrics)
  - [Dashboards](#dashboards)
- [Software Updates](#software-updates)
- [State Sync](#state-sync)
- [Chain Operations](#chain-operations)
- [Private Security Fixes](#private-security-fixes)

## Requirements

To run a Rtd Validator a machine with the following is required:

- CPU: 24 physical cores (or 48 virtual cores)
- Memory: 128 GB
- Storage: 4 TB NVME
- Network: 1 Gbps

## Deployment

Rtd Node can be deployed in a number of ways.

RTD does not currently publish a public Mainnet release endpoint. Obtain a
signed `linux/amd64` binary from a release channel you trust, and obtain its
verification key through a separate trusted channel. From the repository root:

```shell
export RTD_RELEASE_BASE_URL="https://<your-artifact-host>/releases"
export RTD_RELEASE_PUBLIC_KEY="/path/to/trusted-rtd-release-key.pem"
./nre/download_and_verify_private_binary.sh "$RTD_SHA" rtd-node
```

To build directly from source:

```shell
git clone https://github.com/LinkUVerse/rtd.git && cd rtd
git checkout [SHA|BRANCH|TAG]
cargo build --release --bin rtd-node
```

Configuration and guides are available for the following deployment options:

- [Systemd](./systemd/README.md)
- [Ansible](./ansible/README.md)
- [Docker Compose](./docker/README.md)

## Configuration

Rtd Node runs with a single configuration file provided as an argument, example:

`./rtd-node --config-path /opt/rtd/config/validator.yaml`.

Configuration templates are available here:

- [Validator](./config/validator.yaml)

## Connectivity

Rtd Node uses the following ports by default:

| protocol/port | reachability     | purpose                           |
| ------------- | ---------------- | --------------------------------- |
| TCP/8080      | inbound          | protocol / transaction interface  |
| TCP/8081      | inbound/outbound | consensus interface               |
| UDP/8081      | inbound/outbound | narwhal primary interface         |
| UDP/8082      | inbound/outbound | narwhal worker interface          |
| UDP/8084      | inbound/outbound | peer to peer state sync interface |
| TCP/8443      | outbound         | optional operator metrics proxy  |
| TCP/9184      | localhost        | metrics scraping                  |

To run a validator successfully it is critical that ports 8080-8084 are open as outlined above, including the specific protocol (TCP/UDP).

## Storage

All Rtd Node-related data is stored by default under `/opt/rtd/db/`. This is controlled in the Rtd Node configuration file.

```shell
$ cat /opt/rtd/config/validator.yaml | grep db-path
db-path: /opt/rtd/db/authorities_db
  db-path: /opt/rtd/db/consensus_db
```

Ensure that you have an appropriately sized disk mounted for the database to write to.

- To check the size of the local Rtd Node databases:

```shell
du -sh /opt/rtd/db/
du -sh /opt/rtd/db/authorities_db
du -sh /opt/rtd/db/consensus_db
```

- To delete the local Rtd Node databases:

```shell
sudo systemctl stop rtd-node
sudo rm -rf /opt/rtd/db/authorities_db /opt/rtd/db/consensus_db
```

## Key Management

The following keys are used by Rtd Node:

| key          | scheme   | purpose                         |
| ------------ | -------- | ------------------------------- |
| protocol.key | bls12381 | transactions, narwhal consensus |
| account.key  | ed25519  | controls assets for staking     |
| network.key  | ed25519  | narwhal primary, rtd state sync |
| worker.key   | ed25519  | validate narwhal workers        |

These are configured in the [Rtd Node configuration file](#configuration).

You can generate each of these with the locally built `rtd` CLI. RTD has no
public documentation site yet; use `rtd keytool --help` for available commands.

```
$ rtd keytool generate bls12381
$ rtd keytool generate ed25519
$ rtd keytool generate ed25519
$ rtd keytool generate ed25519
```

This will create files like `0x0061b30cdda02b6f55f575f1485a2890ec5c95b753deabbf823b6de7c936eb26.key` & `bls-0x1b7a4038f207d6c65cc106dd5be7270b3031e671fc8f9c1318b19e94a3bf3ed5.key`
which you can copy to your validator and rename to `protocol.key` or `account.key`, etc.

## Monitoring

### Metrics

Rtd Node exposes metrics via a local HTTP interface. These can be scraped for use in a central monitoring system as well as viewed directly from the node.

- View all metrics:

```shell
curl -s http://localhost:9184/metrics
```

- Search for a particular metric:

```shell
curl http://localhost:9184/metrics | grep <METRIC>
```

Rtd Node can push metrics when your network operator configures a metrics
proxy. The example validator configuration does not enable a public proxy.

### Logs

Logs are controlled using the `RUST_LOG` environment variable.

The `RUST_LOG_JSON=1` environment variable can optionally be set to enable logging in JSON structured format.

Depending on your deployment method, these will be configured in the following places:

- If using Ansible, [here](./ansible/roles/rtd-node/files/rtd-node.service)
- If using Systemd natively, [here](./systemd/rtd-node.service)
- If using Docker Compose, [here](./docker/docker-compose.yaml)

To view and follow the Rtd Node logs:

```shell
journalctl -u rtd-node -f
```

To search for a particular match

```shell
journalctl -u rtd-node -g <SEARCH_TERM>
```

- If using Docker Compose, look at the examples [here](./docker/README.md#logs)

It is possible to change the logging configuration while a node is running using the admin interface.

To view the currently configured logging values:

```shell
curl localhost:1337/logging
```

To change the currently configured logging values:

```shell
curl localhost:1337/logging -d "info"
```

### Dashboards

RTD has no public Mainnet or Testnet validator dashboard yet. Use a dashboard
connected to the metrics service operated by your network.

For viewing total stake, the current validator set, and candidates, use a
verified explorer connected to the RTD network you operate.

## Software Updates

When an update is required to the Rtd Node software the following process can be used. Follow the relevant Systemd or Docker Compose runbook depending on your deployment type. It is highly unlikely that you will want to restart with a clean database.

- If using Systemd, [here](./systemd/README.md#updates)
- If using Docker Compose, [here](./docker/README.md#updates)

## State Sync

Checkpoints in Rtd contain the permanent history of the network. They are comparable to blocks in other blockchains with one big difference being that they are lagging instead of leading. All transactions are final and executed prior to being included in a checkpoint.

These checkpoints are synchronized between validators and fullnodes via a dedicated peer to peer state sync interface.

Inter-validator state sync is always permitted however there are controls available to limit what fullnodes are allowed to sync from a specific validator.

The default and recommended `max-concurrent-connections: 0` configuration does not affect inter-validator state sync, but will restrict all fullnodes from syncing. The Rtd Node [configuration](#configuration) can be modified to allow a known fullnode to sync from a validator:

```shell
p2p-config:
  anemo-config:
    max-concurrent-connections: 0
  seed-peers:
    - address: <multiaddr>  # The p2p address of the fullnode
      peer-id: <peer-id>    # hex encoded network public key of the node
    - address: ...          # another permitted peer
      peer-id: ...
```

## Chain Operations

The following chain operations use the `rtd` CLI. Build it from source or
download a signed binary from your trusted release channel:

```shell
export RTD_RELEASE_BASE_URL="https://<your-artifact-host>/releases"
export RTD_RELEASE_PUBLIC_KEY="/path/to/trusted-rtd-release-key.pem"
./nre/download_and_verify_private_binary.sh "$RTD_SHA" rtd
chmod +x rtd
```

It is recommended and often required that the `rtd` binary release/version matches that of the deployed network.

### Querying On-chain Metadata

Validator metadata can be queried by validator address, using `validator` subcommand of Rtd CLI:

```
rtd validator display-metadata {validator_address}
```

### Updating On-chain Metadata

You can leverage [Validator Tool](validator_tool.md) to perform majority of the following tasks.

An active/pending validator can update its on-chain metadata by submitting a transaction. Some metadata changes take effect immediately, including:

- name
- description
- image url
- project url

Other metadata (keys, addresses etc) only come into effect at the next epoch.

To update metadata, a validator makes a MoveCall transaction that interacts with the System Object. For example:

1. to update name to `new_validator_name`, use the Rtd Client CLI to call `rtd_system::update_validator_name`:

```
rtd client call --package 0x3 --module rtd_system --function update_validator_name --args 0x5 \"new_validator_name\" --gas-budget 10000
```

2. to update p2p address starting from next epoch to `/ip4/192.168.1.1`, use the Rtd Client CLI to call `rtd_system::update_validator_next_epoch_p2p_address`:

```
rtd client call --package 0x3 --module rtd_system --function update_validator_next_epoch_p2p_address --args 0x5 "[4, 192, 168, 1, 1]" --gas-budget 10000
```

See the [full list of metadata update functions here](https://github.com/LinkUVerse/rtd/blob/main/crates/rtd-framework/packages/rtd-system/sources/rtd_system.move#L267-L444).

### Operation Cap

To avoid touching account keys too often and allowing them to be stored off-line, validators can delegate the operation ability to another address. This address can then update the reference gas price and tallying rule on behalf of the validator.

Upon creating a `Validator`, an `UnverifiedValidatorOperationCap` is created as well and transferred to the validator address. The holder of this `Cap` object (short for "Capability") therefore could perform operational actions for this validator. To authorize another address to conduct these operations, a validator transfers the object to another address that they control. The transfer can be done by using Rtd Client CLI: `rtd client transfer`.

To rotate the delegatee address or revoke the authorization, the current holder of `Cap` transfers it to another address. In the event of compromised or lost keys, the validator could create a new `Cap` object to invalidate the incumbent one. This is done by calling `rtd_system::rotate_operation_cap`:

```
rtd client call --package 0x3 --module rtd_system --function rotate_operation_cap --args 0x5 --gas-budget 10000
```

By default the new `Cap` object is transferred to the validator address, which then could be transferred to the new delegatee address. At this point, the old `Cap` becomes invalidated and no longer represents eligibility.

To get the current valid `Cap` object's ID of a validator, use the Rtd Client
CLI `rtd client objects` command after setting the holder as the active
address. RTD has no public Mainnet explorer yet.

### Updating the Gas Price Survey Quote

To update the Gas Price Survey Quote of a validator, which is used to calculate the Reference Gas Price at the end of the epoch, the sender needs to hold a valid [`UnverifiedValidatorOperationCap`](#operation-cap). The sender could be the validator itself, or a trusted delegatee. To do so, call `rtd_system::request_set_gas_price`:

```
rtd client call --package 0x3 --module rtd_system --function request_set_gas_price --args 0x5 {cap_object_id} {new_gas_price} --gas-budget 10000
```

### Updating Validator Commission

To update the commission of a validator, call `rtd_system::request_set_commission_rate`, the update will effectuate in the next epoch. The sender of the transaction must be the validator, no additional objects / capabilities are required. Commission rate is expressed in basis points with 0 being 0.00%, and 10000 being 100%.

```sh
rtd client call --package 0x3 --module rtd_system --function request_set_commission_rate --args 0x5 {commission_rate}
```

If a validator is not yet in active set (candidate state), commission is updated using the `set_candidate_validator_commission_rate` function with the same arguments, like this:

```sh
rtd client call --package 0x3 --module rtd_system --function set_candidate_validator_commission_rate --args 0x5 {commission_rate}
```

### Reporting/Un-reporting Validators

To report a validator or undo an existing reporting, the sender needs to hold a valid [`UnverifiedValidatorOperationCap`](#operation-cap). The sender could be the validator itself, or a trusted delegatee. To do so, call `rtd_system::report_validator/undo_report_validator`:

```
rtd client call --package 0x3 --module rtd_system --function report_validator/undo_report_validator --args 0x5 {cap_object_id} {reportee_address} --gas-budget 10000
```

Once a validator is reported by `2f + 1` other validators by voting power, their staking rewards will be slashed.

### Joining the Validator Set

In order for a Rtd address to join the validator set, they need to first sign up as a validator candidate by calling `rtd_system::request_add_validator_candidate` with their metadata and initial configs:

```
rtd client call --package 0x3 --module rtd_system --function request_add_validator_candidate --args 0x5 {protocol_pubkey_bytes} {network_pubkey_bytes} {worker_pubkey_bytes} {proof_of_possession} {name} {description} {image_url} {project_url} {net_address}
{p2p_address} {primary_address} {worker_address} {gas_price} {commission_rate} --gas-budget 10000
```

After an address becomes a validator candidate, any address (including the candidate address itself) can start staking with the candidate's staking pool. Refer to our dedicated staking FAQ on how staking works. Once a candidate's staking pool has accumulated at least `rtd_system::MIN_VALIDATOR_JOINING_STAKE` amount of stake, the candidate can call `rtd_system::request_add_validator` to officially add themselves to the next epoch's active validator set:

```
rtd client call --package 0x3 --module rtd_system --function request_add_validator --args 0x5 --gas-budget 10000000
```

### Leaving the Validator Set

To leave the validator set starting the next epoch, the sender needs to be an active validator in the current epoch and should call `rtd_system::request_remove_validator`:

```
rtd client call --package 0x3 --module rtd_system --function request_remove_validator --args 0x5 --gas-budget 10000
```

After the validator is removed at the next epoch change, the staking pool will become inactive and stakes can only be withdrawn from an inactive pool.

## Private Security Fixes

If your network operator distributes signed security binaries, obtain the
artifact directory URL and verification public key through trusted channels.
There is currently no default RTD public security release bucket or public key
URL. The scripts refuse to download anything until both are supplied. Keep the
public key on local disk and run the commands from the repository root:

```shell
export RTD_RELEASE_BASE_URL="https://<your-artifact-host>/releases"
export RTD_RELEASE_PUBLIC_KEY="/path/to/trusted-rtd-release-key.pem"
./nre/download_private.sh <commit-sha>
./nre/download_and_verify_private_binary.sh <commit-sha> <binary-name>
```
