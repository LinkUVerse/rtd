# Oracle adapter example

Reference material for the "Oracles for DeFi on Rtd" docs cluster.

- `move/` — the `oracle_adapter` Move package: a provider-neutral price adapter
  over Pyth with staleness, confidence, deviation, and fallback guards, plus a
  `demo` module that emits a read price. After replacing the historical Pyth
  dependency with a verified RTD-compatible release, build and test on its own:

  ```sh
  cd move && rtd move test
  ```

- `ts/` — TypeScript consumption samples (Pyth pull update + read, the stale-read
  rejection, and the Switchboard on-demand variant). Standalone package:

  ```sh
  cd ts && npm install && npm run build   # tsc --noEmit
  ```

The docs pull chunks with `<ImportContent>`, so samples stay tied to code that
compiles. `ts/run.mts` is a local execution harness (not committed): it loads the
active rtd keystore key in memory only and executes the Pyth flows on Testnet.

The provider integrations require deployments on the RTD chain. Before running
the TypeScript example, configure `RTD_JSON_RPC_URL`, `RTD_PYTH_STATE_ID`,
`RTD_WORMHOLE_STATE_ID`, `RTD_HERMES_ENDPOINT`, and
`RTD_PYTH_FEED_RTD_USD_ID` with values verified for the same RTD network.
There are no upstream provider addresses or feed IDs supplied as defaults.
The Move dependency and the Pyth/Switchboard client packages also need RTD
deployments and compatible releases before the on-chain flows can run.
