// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// The inherited site contains unverified public-network and third-party
// claims. Keep the two repository deployment scripts closed until the
// documented release checks have been completed and this gate is reviewed.
console.error(
  'RTD docs publication is blocked: review docs/RTD_PUBLICATION_GATE.md and ' +
    'verify network identity, external services, tokenomics, and generated pages before deployment.',
);
process.exitCode = 1;
