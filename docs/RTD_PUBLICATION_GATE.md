# RTD documentation publication gate

This documentation tree was copied from upstream Sui and mechanically renamed.
It is useful as source material for the fork, but it is **not approved for public
RTD publication**. A successful blockchain build or local devnet startup does
not verify the public services, historical claims, third-party integrations,
economic policy, or screenshots in these pages.

RTD has not launched Mainnet. Its future Mainnet will not expose JSON-RPC,
following the upstream direction; the upstream 2026 shutdown dates are not RTD
events. Public endpoint examples now use `YOUR_RTD_*` configuration placeholders;
they do not imply that a service has been deployed. Supply an explicitly
configured RTD endpoint before using an example. The documentation site's canonical URL is
localhost for preview, analytics and inherited AI widgets are disabled, and
`robots.txt` disallows crawling.

## Verified corrections in this audit

- The PDFs previously served as `rtd.pdf`, `rtd-lutris.pdf`, and
  `rtd-cli-cheatsheet.pdf` contain the original Sui paper, Sui Lutris paper,
  and Sui CLI commands. They now retain `sui` names in the source and static
  directories. `tokenomics.pdf` is the original 2022 Sui economics paper and
  is named `sui-tokenomics.pdf`. The original PDF text and author credits have
  **not** been rewritten.
- The research index identifies upstream authors and measurements. The RTD
  Move concepts page links to the Sui paper as historical design context.
- The RTD CLI cheatsheet no longer offers the Sui CLI PDF as an RTD download.
- The shared explorer snippet no longer presents Suiscan or SuiVision as RTD
  explorers. Their existing domains display Sui data.
- The home page and Rtd Stack landing page now distinguish source-level
  capabilities from inherited Sui services; navigation labels do not establish
  that any public RTD integration exists.

## Open blockers

1. **Network identity and services.** Confirm a deployed RTD genesis and
   chain ID, then verify every Mainnet/Testnet/Devnet claim, RPC URL, faucet,
   snapshot bucket, wallet, and explorer against that chain. Until then, pages
   such as `content/develop/rtd-architecture/networks.mdx`,
   `content/getting-started/onboarding/get-coins.mdx`, and
   `content/operators/snapshots.mdx` must not be published as instructions.
2. **Third-party Sui links.** Many pages link to `suiscan.xyz`,
   `suivision.xyz`, `slush.app`, Sui package IDs, upstream DeepBook objects,
   and Sui transaction digests while calling them RTD examples. Each link
   needs an RTD-chain replacement or an explicit upstream attribution. The
   initial audit found these terms in roughly 37 documentation files.
   Individual child pages and generated content still require a release audit;
   navigation and landing-page corrections do not validate every example.
3. **Installer and distribution.** `suiup`, Homebrew, Chocolatey, GitHub
   Releases, and hosted downloadable binaries are not verified RTD delivery
   channels. Installation pages and reusable snippets must be rebuilt around
   an actually published RTD binary or source build.
4. **Economics and provenance.** The inherited 2022 Sui tokenomics PDF,
   Sui mainnet launch/vesting dates, exchange listings, and Sui foundation
   policies do not describe RTD. RTD's formal genesis allocation, schedule,
   and any public economics policy require independent approval.
5. **Inherited product stack.** Walrus, Seal, Nautilus, zkLogin providers,
   SuiNS, MVR, DeepBook, bridge, game, and wallet pages need feature-by-feature
   validation. A package present in source does not prove an RTD service or
   public deployment exists.
6. **Site publication.** The checked-in site uses localhost as its canonical
   URL, hides inherited search and AI widgets, and blocks its deployment
   commands. Review generated pages, `site/vercel.json`, `site/static/llms.txt`,
   analytics, banners, and external links against an actual RTD release before
   enabling publication.
7. **JSON-RPC policy.** RTD has not launched Mainnet. Future RTD Mainnet will
   not expose JSON-RPC, following the upstream Sui direction. The Sui
   July/October 2026 migration dates are not RTD events. Local development
   deployments may still expose the legacy API while clients migrate.

## Release check

Publication requires an owner to inventory every external URL and onchain
identifier in `content/` and `site/`, verify the target chain and content,
approve the economic and security claims, remove or attribute every Sui-only
example, and review the generated site plus `llms.txt`. Keep the site private
until that review passes. Historical Sui papers, original author names, and
upstream commit titles should remain accurately attributed rather than
mechanically changed to RTD.
