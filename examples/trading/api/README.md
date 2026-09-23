# Escrow API Demo

This demo is built to showcase how we can build an event indexer + API
to efficiently serve on-chain data for our app.

The demo indexer uses polling to watch for new events.

The example defaults to localnet. For a remote RTD network, set
`NETWORK=<mainnet|testnet|devnet>` and `RTD_RPC_URL` to a JSON-RPC endpoint
serving that network. Publish the contracts there first; the generated
`escrow-contract.json` and `demo-contract.json` are local outputs and no
upstream package IDs are valid on RTD.

## Installation

1. Install dependencies by running

```
pnpm install --ignore-workspace
```

2. Setup the database by running

```
pnpm db:setup:dev
```

3. [Publish the contract & demo data](#demo-data)

4. Run both the API and the indexer

```
pnpm dev
```

5. Visit [http://localhost:3000/escrows](http://localhost:3000/escrows) or [http://localhost:3000/locked](http://localhost:3000/locked)

## Demo Data<a name="demo-data"></a>

> Make sure you have enough Testnet (or any net) RTD in the active address of the CLI.

There are some helper functions to:

1. Publish the smart contract
2. Create some demo data (for testnet)

To produce demo data:

1. Publish the smart contract by running

```
npx ts-node helpers/publish-contracts.ts
```

2. Produce demo non-locked and locked objects

```
npx ts-node helpers/create-demo-data.ts
```

3. Produce demo escrows

```
npx ts-node helpers/create-demo-escrows.ts
```

If you want to reset the database (start from scratch), run:

```
pnpm db:reset:dev && pnpm db:setup:dev
```

## API

The API exposes data written from the event indexer.

For each request, we have pagination with a max limit of 50 per page.

| Parameter | Expected value  |
| --------- | --------------- |
| limit     | number (1-50)   |
| cursor    | number          |
| sort      | 'asc' \| 'desc' |

There are two available routes:

### `/locked`: Returns indexed locked objects

Available query parameters:

| Parameter | Expected value    |
| --------- | ----------------- |
| deleted   | 'true' \| 'false' |
| keyId     | string            |
| creator   | string            |

### `/escrows`: Returns indexed escrow objects

Available query parameters:

| Parameter | Expected value    |
| --------- | ----------------- |
| cancelled | 'true' \| 'false' |
| swapped   | 'true' \| 'false' |
| recipient | string            |
| sender    | string            |

> Example Query: Get only active escrows for an address created on your RTD network (5 per page).

```
curl --get 'http://localhost:3000/escrows' --data-urlencode 'limit=5' --data-urlencode "recipient=$RTD_RECIPIENT_ADDRESS" --data-urlencode 'cancelled=false' --data-urlencode 'swapped=false'
```

## Event Indexer

> Run only a single instance of the indexer.

Indexer uses polling to watch for new events. We're saving the
cursor data in the database so we can start from where we left off
when restarting the API.

To run the indexer individually, run:

```
pnpm indexer
```
