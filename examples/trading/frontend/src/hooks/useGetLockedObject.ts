// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { useRtdClientQuery } from "@linku/dapp-kit";

/**
 * A re-usable hook for querying a locked object by ID
 * from the on-chain state.
 */
export function useGetLockedObject({ lockedId }: { lockedId: string }) {
  return useRtdClientQuery(
    "getObject",
    {
      id: lockedId,
      options: {
        showType: true,
        showOwner: true,
        showContent: true,
      },
    },
    {
      enabled: !!lockedId,
    },
  );
}
