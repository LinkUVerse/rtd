# Trading e2e demo - Frontend

This dApp was created using `@linku/create-dapp` that sets up a basic React
Client dApp.

## First Steps

Before running the frontend, it's recommended that you follow the API setup to
[publish the contracts](../api/README.md) (or re-use the published ones).

### Demo Contracts

No demo packages are pre-deployed on RTD. Publish both contracts on the target
RTD network and set `VITE_RTD_ESCROW_PACKAGE_ID` and
`VITE_RTD_DEMO_PACKAGE_ID` to the generated package IDs. The frontend requires
these values at startup. It defaults to localnet; for a remote network set its
`VITE_RTD_<NETWORK>_RPC_URL` before connecting a wallet. Set
`VITE_RTD_API_URL` if the local API is not at `http://localhost:3000/`.
Optional `VITE_RTD_EXPLORER_URL` enables object and address links; without
an RTD explorer deployment, IDs remain copyable text.

### Constants

You can change package addresses, the api endpoint, etc, on the `constants.ts`
file.

## Starting the dApp

To install dependencies you can run

```bash
pnpm install --ignore-workspace
```

To start your dApp in development mode run

```bash
pnpm dev
```

## Building

To build your app for deployment you can run

```bash
pnpm build
```
