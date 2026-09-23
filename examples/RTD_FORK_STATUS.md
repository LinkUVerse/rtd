# RTD example deployment status

The examples are source references. A fresh RTD chain has no inherited
application packages, oracle state, USDC, DeepBook pools, Walrus metadata,
or public explorer. A matching RTD SDK release and network deployment are
required before a third-party example can run.

The current standalone TS SDK publishes source packages named `rtd-typescript`
and `rtd-dapp-kit-react`. Many inherited examples still import the old
`@linku/rtd` / `@linku/dapp-kit` package API, and DeepBook examples import an
SDK that was deliberately excluded from the retained TS package set. They are
kept as source references for review; they cannot be claimed as buildable RTD
examples until their imports and API calls are migrated to the retained SDK
packages and those packages are installable. The forked SDK projects themselves
have separate passing build and test results.

| Example | Required RTD deployment/configuration |
| --- | --- |
| Tic-tac-toe | Publish its Move package; `scripts/publish.sh` writes the UI and CLI package IDs. Configure remote RPC and optional explorer URLs in the UI. |
| Trading | Publish escrow and demo packages; supply package IDs to the frontend, and an RPC URL to the API for a remote network. |
| Oracle adapter | Deploy compatible Pyth/Wormhole packages and state objects, provide a feed ID and Hermes endpoint. The pinned Move dependency is a historical reference, not an RTD deployment. |
| DeepBook spot, margin, Predict, MEV | Deploy these separate protocols and use an SDK that records their RTD package/object IDs; configure an RTD node URL. Predict checks its deployment name and chain ID. |
| USDC transfer and Move USDC usage | Deploy or verify a third-party USDC coin on RTD; use its actual Move type and package. |
| Walrus indexer | Deploy Walrus on RTD; provide the metadata StructTag and RTD checkpoint store URL. |
| Regulated coin | Publish the example coin on RTD; provide the resulting package ID and node URL. |
| GraphQL production config | Add a name-service section only after an RTD name service exists. |

No sample address, package ID, or URL from the upstream chain can prove an
RTD deployment. The examples intentionally fail or remain unavailable when
their required configuration is missing.
