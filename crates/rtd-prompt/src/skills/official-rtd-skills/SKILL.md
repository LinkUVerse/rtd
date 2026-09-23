---
name: official-rtd-skills
description: >
  Status of RTD-specific developer skills and pointers to checked-in framework,
  CLI, and SDK sources. Use for Move contracts, publishing, upgrades, SDKs,
  transaction blocks, and accessing local-chain data.
---

# RTD developer guidance

A separately maintained, official RTD skills repository has not been verified for this fork.
Use these checked-in sources for current behavior:

- `crates/rtd-framework/packages/` for framework and system Move modules.
- `docs/content/` for RTD developer guidance, with deployment status called out per page.
- `crates/rtd/src/` and `crates/rtd-sdk/` for the CLI and Rust client behavior.
- The sibling `rtd-ts-sdk/packages/` checkout for retained TypeScript SDK packages.

The audit catalog bundled with `rtd prompt` has historical upstream lineage recorded in
`crates/rtd-prompt/src/maintenance/UPSTREAMS.md`. Validate each inherited rule against the
current RTD framework before applying it. RTD has no public Mainnet endpoint.
