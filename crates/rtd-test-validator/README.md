The rtd-test-validator starts a local network that includes a Rtd Full node, a Rtd validator, a Rtd faucet and (optionally)
an indexer.

## Guide

Refer to [local network](https://docs.rtd.io/getting-started/onboarding/local-network)

## Run with a persisted state
You can combine this with indexer runs as well to save a persisted state on local development.

1. Generate a config to store db and genesis configs `rtd genesis -f --with-faucet --working-dir=[some-directory]`
2. `rtd-test-validator --config-dir [some-directory]`
