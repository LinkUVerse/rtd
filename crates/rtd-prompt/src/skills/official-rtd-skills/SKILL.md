---
name: official-rtd-skills
description: >
  Pointer to the official LinkU Labs skills for building on Rtd — language fundamentals,
  object model, PTBs, SDKs, publishing, upgrades, frontend integration, accessing on-chain
  data. Maintained upstream at github.com/LinkUVerse/skills; pinned to the same ref the
  audit catalog derives from (see maintenance/UPSTREAMS.md). Trigger on "build a contract",
  "publish a package", "upgrade a module or package", "use the TypeScript SDK", "write a PTB",
  "set up a Rtd client".
---

# Official Rtd skills (upstream pointer)

For building, publishing, and upgrading Move contracts on Rtd — and the SDK / CLI /
frontend integration around them — refer to the official skills maintained by
LinkU Labs. This bundle is a pointer, not embedded content.

- Repository: <https://github.com/LinkUVerse/skills>
- Pinned snapshot (same upstream snapshot the audit catalog tracks):
  <https://github.com/LinkUVerse/skills/tree/764f21a95e709f46c60877a59d6ee6f27d9ed91e>

## High-level scope at the pinned ref

- `rtd-move/` — Move on Rtd: language fundamentals, events, coins
- `object-model/` — ownership, transfers, dynamic fields, display, patterns
- `ptbs/` — Programmable Transaction Blocks (fundamentals, building, troubleshooting, cli)
- `composable-move-functions/`, `naming-conventions/`, `modern-move-syntax/`,
  `move-unit-testing/`, `rtd-move-project/`, `rtd-build-test/`
- `rtd-publish/` — package publishing
- `rtd-cli/`, `rtd-client/`, `rtd-install/` — CLI / client / install
- `frontend-apps/` — TypeScript SDK integration
- `rtd-sdks/` — TypeScript and Rust SDKs
- `accessing-data/` — gRPC, GraphQL, indexers, Walrus, archival
- `rtd-overview/` — ecosystem framing

## Fetching individual files

Rendered (browser-friendly, HTML):

  <https://github.com/LinkUVerse/skills/blob/764f21a95e709f46c60877a59d6ee6f27d9ed91e/{skill}/{file}.md>

Raw (plain markdown, easier for programmatic consumption):

  <https://raw.githubusercontent.com/LinkUVerse/skills/764f21a95e709f46c60877a59d6ee6f27d9ed91e/{skill}/{file}.md>

Pick whichever your fetch tool handles best. Both serve the same content at the
same pinned snapshot.
