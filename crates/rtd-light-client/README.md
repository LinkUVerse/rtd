This crate contains a Command Line Interface light client for Rtd.

# What is a light client?

A light client allows checking the authenticity and validity of on-chain state, such as transactions, their effects including events and object contents, without the cost of running a full node.

Running a *full node* requires downloading the full sequence of all transaction and re-executing them. Then the full state of the blockchain is available locally to serve reads. This is however an expensive process in terms of network bandwidth needed to download the full sequence of transactions, as well as CPU to re-execute it, and storage to store the full state of the blockchain.

Alternatively, a *light client* only needs to download minimal information to authenticate blockchain state. Specifically in Rtd, the light client needs to *sync* all end-of-epoch checkpoints that contain information about the committee in the next epoch. Sync involves downloading the checkpoints and checking their validity by checking their certificate.

Once all end-of-epoch checkpoints are downloaded and checked, any event or current object can be checked for its validity. To do that the light client downloads the checkpoint in which the transaction was executed, and the effects structure that summarizes its effects on the system, including events emitted and objects created. The chain of validity from the checkpoint to the effects and its contents is checked via the certificate on the checkpoint and the hashes of all structures.

## Ensuring valid data display

A light client can ensure the correctness of the event and object data using the techniques defined above. However, the light client CLI utility also needs to pretty-print the structures in JSON, which requires knowledge of the correct type for each event or object. Types themselves are defined in modules that have been uploaded by past transactions. Therefore to ensure correct display the light client authenticates that all modules needed to display sought items are also correct.

# Usage

The light client requires a config file and a directory to cache checkpoints, and then can be used to check the validity of transaction and their events or of objects.

## Setup

The config file for the light client takes a URL for a full node, a directory to store checkpoint summaries (that must exist), and the name of a genesis blob generated for the same RTD network. RTD has no public Mainnet, Testnet, GraphQL, or checkpoint archive service yet. Set every endpoint explicitly for your own network before running the light client.

```
full_node_url: "http://127.0.0.1:9000"
# make sure this directory is created and has genesis.blob
checkpoint_summary_dir: "checkpoints_dir"
genesis_filename: "genesis.blob"
object_store_url: ""
graphql_url: null
archive_store_config: null
```

Generate a genesis blob for your own RTD network and place it inside the checkpoint summary directory.

The `mainnet.yaml` and `testnet.yaml` files are unconfigured templates, not working public network profiles.

## Sync

Every day there is a need to download new checkpoints through sync by doing:
```
$ rtd-light-client --config <your-config.yaml> sync
```

Where `mainnet.yaml` is the config file above.

This command will download all end-of-epoch checkpoints, and check them for validity. They will be cached within the checkpoint summary directory for use by future invocations.

Internally, sync works in two steps. It first downloads the end-of-epoch checkpoint numbers into the `checkpoints.yaml` file (which needs to be present in the checkpoint summaries directory). Next, it downloads the corresponding checkpoint summaries.

## Check Transaction

To check a transaction was executed, as well as the events it emitted do:
```
$ rtd-light-client --config <your-config.yaml> transaction -t <RTD_TRANSACTION_DIGEST>
```

Where the base58 encoding of the transaction ID is specified. If the transaction has been executed the transaction ID the effects digest are displayed and all the events are printed in JSON. If not an error is printed.

## Check Object

To check an object provide its ID in the following way:

```
$ rtd-light-client --config <your-config.yaml> object -o <RTD_OBJECT_ID>
```

The object ID is represented in Hex as displayed in explorers. If the object exists in the latest state it is printed out in JSON, otherwise an error is printed.
