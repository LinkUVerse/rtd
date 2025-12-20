This crate provides the Rtd Rust SDK, containing APIs to interact with the Rtd network. Auto-generated documentation for this crate is [here](https://linkulabs.github.io/rtd/rtd_sdk/index.html).

## Getting started

Add the `rtd-sdk` dependency as following:

```toml
rtd_sdk = { git = "https://github.com/linkulabs/rtd", package = "rtd-sdk"}
tokio = { version = "1.2", features = ["full"] }
anyhow = "1.0"
```

The main building block for the Rtd Rust SDK is the `RtdClientBuilder`, which provides a simple and straightforward way of connecting to a Rtd network and having access to the different available APIs.

In the following example, the application connects to the Rtd `testnet` and `devnet` networks and prints out their respective RPC API versions.

```rust
use rtd_sdk::RtdClientBuilder;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Rtd testnet -- https://fullnode.testnet.rtd.io:443
    let rtd_testnet = RtdClientBuilder::default().build_testnet().await?;
    println!("Rtd testnet version: {}", rtd_testnet.api_version());

     // Rtd devnet -- https://fullnode.devnet.rtd.io:443
    let rtd_devnet = RtdClientBuilder::default().build_devnet().await?;
    println!("Rtd devnet version: {}", rtd_devnet.api_version());

    // Rtd mainnet -- https://fullnode.mainnet.rtd.io:443
    let rtd_mainnet = RtdClientBuilder::default().build_mainnet().await?;
    println!("Rtd mainnet version: {}", rtd_mainnet.api_version());

    Ok(())
}

```

## Documentation for rtd-sdk crate

[GitHub Pages](https://linkulabs.github.io/rtd/rtd_sdk/index.html) hosts the generated documentation for all Rust crates in the Rtd repository.

### Building documentation locally

You can also build the documentation locally. To do so,

1. Clone the `rtd` repo locally. Open a Terminal or Console and go to the `rtd/crates/rtd-sdk` directory.

1. Run `cargo doc` to build the documentation into the `rtd/target` directory. Take note of location of the generated file from the last line of the output, for example `Generated /Users/foo/rtd/target/doc/rtd_sdk/index.html`.

1. Use a web browser, like Chrome, to open the `.../target/doc/rtd_sdk/index.html` file at the location your console reported in the previous step.

## Rust SDK examples

The [examples](https://github.com/LinkUVerse/rtd/tree/main/crates/rtd-sdk/examples) folder provides both basic and advanced examples.

There are serveral files ending in `_api.rs` which provide code examples of the corresponding APIs and their methods. These showcase how to use the Rtd Rust SDK, and can be run against the Rtd testnet. Below are instructions on the prerequisites and how to run these examples.

### Prerequisites

Unless otherwise specified, most of these examples assume `Rust` and `cargo` are installed, and that there is an available internet connection. The examples connect to the Rtd testnet (`https://fullnode.testnet.rtd.io:443`) and execute different APIs using the active address from the local wallet. If there is no local wallet, it will create one, generate two addresses, set one of them to be active, and it will request 1 RTD from the testnet faucet for the active address.

### Running the existing examples

In the root folder of the `rtd` repository (or in the `rtd-sdk` crate folder), you can individually run examples using the command  `cargo run --example filename` (without `.rs` extension). For example:
* `cargo run --example rtd_client` -- this one requires a local Rtd network running (see [here](#Connecting to Rtd Network
)). If you do not have a local Rtd network running, please skip this example.
* `cargo run --example coin_read_api`
* `cargo run --example event_api` -- note that this will subscribe to a stream and thus the program will not terminate unless forced (Ctrl+C)
* `cargo run --example governance_api`
* `cargo run --example read_api`
* `cargo run --example programmable_transactions_api`
* `cargo run --example sign_tx_guide`

### Basic Examples

#### Connecting to Rtd Network
The `RtdClientBuilder` struct provides a connection to the JSON-RPC server that you use for all read-only operations. The default URLs to connect to the Rtd network are:

- Local: http://127.0.0.1:9000
- Devnet: https://fullnode.devnet.rtd.io:443
- Testnet: https://fullnode.testnet.rtd.io:443
- Mainnet: https://fullnode.mainnet.rtd.io:443

For all available servers, see [here](https://rtd.io/networkinfo).

For running a local Rtd network, please follow [this guide](https://docs.rtd.io/build/rtd-local-network) for installing Rtd and [this guide](https://docs.rtd.io/build/rtd-local-network#start-the-local-network) for starting the local Rtd network.


```rust
use rtd_sdk::RtdClientBuilder;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let rtd = RtdClientBuilder::default()
        .build("http://127.0.0.1:9000") // local network address
        .await?;
    println!("Rtd local network version: {}", rtd.api_version());

    // local Rtd network, like the above one but using the dedicated function
    let rtd_local = RtdClientBuilder::default().build_localnet().await?;
    println!("Rtd local network version: {}", rtd_local.api_version());

    // Rtd devnet -- https://fullnode.devnet.rtd.io:443
    let rtd_devnet = RtdClientBuilder::default().build_devnet().await?;
    println!("Rtd devnet version: {}", rtd_devnet.api_version());

    // Rtd testnet -- https://fullnode.testnet.rtd.io:443
    let rtd_testnet = RtdClientBuilder::default().build_testnet().await?;
    println!("Rtd testnet version: {}", rtd_testnet.api_version());

    Ok(())
}
```

#### Read the total coin balance for each coin type owned by this address
```rust
use std::str::FromStr;
use rtd_sdk::types::base_types::RtdAddress;
use rtd_sdk::{ RtdClientBuilder};
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {

   let rtd_local = RtdClientBuilder::default().build_localnet().await?;
   println!("Rtd local network version: {}", rtd_local.api_version());

   let active_address = RtdAddress::from_str("<YOUR RTD ADDRESS>")?; // change to your Rtd address

   let total_balance = rtd_local
      .coin_read_api()
      .get_all_balances(active_address)
      .await?;
   println!("The balances for all coins owned by address: {active_address} are {:#?}", total_balance);
   Ok(())
}
```

## Advanced examples

See the programmable transactions [example](https://github.com/LinkUVerse/rtd/blob/main/crates/rtd-sdk/examples/programmable_transactions_api.rs).

## Games examples

### Tic Tac Toe quick start

1. Prepare the environment
   1. Install `rtd` binary following the [Rtd installation](https://github.com/LinkUVerse/rtd/blob/main/docs/content/guides/developer/getting-started/rtd-install.mdx) docs.
   1. [Connect to Rtd Devnet](https://github.com/LinkUVerse/rtd/blob/main/docs/content/guides/developer/getting-started/connect.mdx).
   1. [Make sure you have two addresses with gas](https://github.com/LinkUVerse/rtd/blob/main/docs/content/guides/developer/getting-started/get-address.mdx) by using the `new-address` command to create new addresses:
      ```shell
      rtd client new-address ed25519
      ```
      You must specify the key scheme, one of `ed25519` or `secp256k1` or `secp256r1`.
      You can skip this step if you are going to play with a friend. :)
   1. [Request Rtd tokens](https://github.com/LinkUVerse/rtd/blob/main/docs/content/guides/developer/getting-started/get-coins.mdx) for all addresses that will be used to join the game.

2. Publish the move contract
   1. [Download the Rtd source code](https://github.com/LinkUVerse/rtd/blob/main/docs/content/guides/developer/getting-started/rtd-install.mdx).
   1. Publish the [`tic-tac-toe` package](https://github.com/LinkUVerse/rtd/tree/main/examples/tic-tac-toe/move)
      using the Rtd client:
      ```shell
      rtd client publish --path /path-to-rtd-source-code/examples/tic-tac-toe/move
      ```
   1. Record the package object ID.

3. Create a new tic-tac-toe game
   1. Run the following command in the [`tic-tac-toe/cli` directory](https://github.com/LinkUVerse/rtd/tree/main/examples/tic-tac-toe/cli) to start a new game, replacing the game package objects ID with the one you recorded:
      ```shell
      cargo run -- new --package-id <<tic-tac-toe package object ID>> <<player O address>>
      ```
      This will create a game between the active address in the keystore, and the specified Player O.
   1. Copy the game ID and pass it to your friend to join the game.

4. Making a move

   Run the following command in the [`tic-tac-toe/cli` directory](https://github.com/LinkUVerse/rtd/tree/main/examples/tic-tac-toe/cli) to make a move in an existing game, as the active address in the CLI, replacing the game ID and address accordingly:
   ```shell
   cargo run -- move --package-id <<tic-tac-toe package object ID>> --row $R --col $C <<game ID>>
   ```

## License

[SPDX-License-Identifier: Apache-2.0](https://github.com/LinkUVerse/rtd/blob/main/LICENSE)
