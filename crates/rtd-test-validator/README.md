The rtd-test-validator starts a local network that includes a Rtd Full node, a Rtd validator, a Rtd faucet and (optionally)
an indexer.

## Guide

Refer to [rtd-local-network.md](../../docs/content/guides/developer/getting-started/local-network.mdx)

## Experimental Feature - Running with Indexer

**Note** Similar to the fullnode db, all state will be wiped upon restart

1. Follow the [Prerequisites section](../../crates/rtd-indexer/README.md#prerequisites) in the `rtd-indexer` README to set up the postgresdb on your local machine
2. Make sure the `Posgresdb` starts on your local machine
3. run `RUST_LOG="consensus=off" ./target/debug/rtd-test-validator --with-indexer`
4. To check your local db, if you use the default db url `postgres://postgres:postgres@localhost:5432/rtd_indexer`, you can login to the `postgres` database and run `\dt` to show all tables.

## Run with a persisted state
You can combine this with indexer runs as well to save a persisted state on local development.

1. Generate a config to store db and genesis configs `rtd genesis -f --with-faucet --working-dir=[some-directory]`
2. `rtd-test-validator --config-dir [some-directory]`
