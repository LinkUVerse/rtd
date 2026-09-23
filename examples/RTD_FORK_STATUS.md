# RTD example deployment status

The examples are source references. A fresh RTD chain has no inherited
application packages, oracle state, USDC, DeepBook pools, Walrus metadata,
or public explorer. A matching RTD SDK release and network deployment are
required before a third-party example can run.

The current standalone TS SDK contains `rtd-typescript`, `rtd-dapp-kit-react`,
`rtd-kiosk`, and `rtd-bcs`. Many inherited examples still import the old
`@linku/rtd` / `@linku/dapp-kit` package API. In particular, the current
`rtd-typescript/client` no longer exports the old `RtdClient` or
`getFullnodeUrl` JSON-RPC API. DeepBook examples import an SDK that was
deliberately excluded from the retained TS package set. The root pnpm
workspace therefore excludes all inherited `dapps/**` and `examples/**`
JavaScript packages; their old registry dependencies must not be installed as
part of a main-chain build. They are source references for review, not
buildable RTD examples. Each one needs API migration and a build against the
retained SDK before rejoining the workspace. The forked SDK projects have
separate build and test evidence.

The root `pnpm-lock.yaml` still contains unreferenced package snapshots for
these inherited SDK dependencies. Its only importers are the root package and
the local Move formatter, and neither importer depends on an old SDK package.
The offline frozen lockfile check passes. A full offline install was blocked by
an unrelated missing `prettier` tarball in the local pnpm store, so no JS build
claim is made here.

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
