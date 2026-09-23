# DeepBook Margin examples

These third-party examples require a DeepBook Margin deployment and an SDK
release configured for the same RTD chain. Set `RTD_DEEPBOOK_GRPC_URL` to its
RTD Testnet gRPC endpoint. No public fullnode or margin package IDs are assumed.

Runnable TypeScript sources for the DeepBook Margin docs (the leveraged position
workflow). The docs pull each chunk in with
`<ImportContent mode="code" tag="..." />`, so published samples stay tied to code
that type-checks.

Standalone package (excluded from the root pnpm workspace because it pins
`@linku/rtd` 2.x). Build it on its own:

```sh
npm install
npm run build   # tsc --noEmit
```

Each `src/*.ts` file wraps its doc chunk in `// docs::#<tag>` / `// docs::/#<tag>`
markers: `client`, `find-manager`, `create-manager`, `risk-params`,
`pool-liquidity`, `deposit-collateral`, `safe-collateral`, `withdraw-collateral`,
`borrow`, `repay`, `open-position`, `reduce-position`, `read-risk`.

Margin trading borrows funds and can be liquidated. These samples target Rtd
Testnet with the SDK's Testnet constants; do not point them at Mainnet without
understanding the margin risks (see the checked-in `docs/content` tree).

`run-margin.mts` (not committed) is a local execution harness, not a doc sample.
It reads a Testnet key from `DEEPBOOK_DOCS_TESTNET_KEY` (env only, never logged),
creates a MarginManager, and prints the manager ID plus the pool's on-chain risk
parameters and borrow liquidity for maintainer verification:

```sh
export DEEPBOOK_DOCS_TESTNET_KEY='rtdprivkey1...'   # same shell
npm install && npx tsx run-margin.mts
```
