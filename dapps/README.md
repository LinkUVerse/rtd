## Inherited dApp source

These dApps are retained as source references. They were written against the
older `@linku/rtd` and `@linku/dapp-kit` APIs and are excluded from the root
pnpm workspace. The current RTD TypeScript SDK packages are
`rtd-typescript`, `rtd-dapp-kit-react`, and `rtd-kiosk`; changing import names
alone does not migrate the old JSON-RPC client and wallet APIs.

Before running a dApp, migrate its imports and API calls, build it against the
current SDK, and deploy its Move packages and any required services to the
target RTD network. RTD has no preconfigured public testnet, faucet, or
explorer. See [the example status](../examples/RTD_FORK_STATUS.md) for other
deployment requirements.
