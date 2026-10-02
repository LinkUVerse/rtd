This crate provides the Rtd Rust SDK, containing APIs to interact with the Rtd network. Build API documentation locally with `cargo doc -p rtd-sdk`.

**Network status:** RTD has no public Mainnet, Testnet, Devnet, or faucet yet. Use a locally running RTD node or provide an endpoint for a network you operate. The named network builders require explicit RPC URLs in the corresponding environment variables.

## Getting started

Add the `rtd-sdk` dependency as following:

```toml
rtd_sdk = { git = "https://github.com/linkuverse/rtd", package = "rtd-sdk"}
tokio = { version = "1.2", features = ["full"] }
anyhow = "1.0"
```

The main building block for the Rtd Rust SDK is the `RtdClientBuilder`, which provides a simple and straightforward way of connecting to a Rtd network and having access to the different available APIs.

In the following example, the application connects to a local RTD node and prints its RPC API version.

```rust
use rtd_sdk::RtdClientBuilder;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let rtd_local = RtdClientBuilder::default().build_localnet().await?;
    println!("Rtd local version: {}", rtd_local.api_version());

    Ok(())
}

```

## Documentation for rtd-sdk crate

The generated documentation can be opened from the local `target/doc/rtd_sdk/index.html` file.

### Building documentation locally

You can also build the documentation locally. To do so,

1. Clone the `rtd` repo locally. Open a Terminal or Console and go to the `rtd/crates/rtd-sdk` directory.

1. Run `cargo doc` to build the documentation into the `rtd/target` directory. Take note of location of the generated file from the last line of the output, for example `Generated /Users/foo/rtd/target/doc/rtd_sdk/index.html`.

1. Use a web browser, like Chrome, to open the `.../target/doc/rtd_sdk/index.html` file at the location your console reported in the previous step.

## Rust SDK examples

The [examples](https://github.com/LinkUVerse/rtd/tree/main/crates/rtd-sdk/examples) folder provides both basic and advanced examples.

Several files ending in `_api.rs` show the corresponding APIs and methods. These examples were inherited from upstream and require adaptation to a local or explicitly configured RTD network before they can be run.

### Prerequisites

Unless otherwise specified, these examples assume `Rust` and `cargo` are installed, and that a funded local wallet and RTD node are available. Any example that requests tokens from a public faucet must be changed to use a faucet that you operate.

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
The `RtdClientBuilder` struct connects to an RTD RPC server. The local development URL is:

- Local: http://127.0.0.1:9000
- Other networks: explicitly configure the RPC URL for a network you operate.

For a local network, build the `rtd` binary from this repository and run `rtd start`.


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
   1. Start a local RTD node and connect the CLI to its local RPC endpoint.
   1. [Make sure you have two addresses with gas](https://github.com/LinkUVerse/rtd/blob/main/docs/content/guides/developer/getting-started/get-address.mdx) by using the `new-address` command to create new addresses:
      ```shell
      rtd client new-address ed25519
      ```
      You must specify the key scheme, one of `ed25519` or `secp256k1` or `secp256r1`.
      You can skip this step if you are going to play with a friend. :)
   1. Fund the local addresses using the faucet attached to your local RTD network.

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
