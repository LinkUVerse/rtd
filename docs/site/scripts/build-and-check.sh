#!/usr/bin/env bash
set -euo pipefail

# Build only the checked-in RTD documentation. Pulling upstream Sui docs or
# unverified external apps during a build would recreate false RTD claims.
node scripts/validate-frontmatter.mjs --summary
node scripts/generate-import-context.js
docusaurus build
