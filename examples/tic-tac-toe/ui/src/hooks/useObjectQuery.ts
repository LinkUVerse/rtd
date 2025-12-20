// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { useRtdClientContext, useRtdClientQuery, UseRtdClientQueryOptions } from "@linku/dapp-kit";
import { GetObjectParams, RtdObjectResponse } from "@linku/rtd/client";
import { useQueryClient, UseQueryResult } from "@tanstack/react-query";

export type UseObjectQueryOptions = UseRtdClientQueryOptions<"getObject", RtdObjectResponse>;
export type UseObjectQueryResponse = UseQueryResult<RtdObjectResponse, Error>;
export type InvalidateUseObjectQuery = () => void;

/**
 * Fetches an object, returning the response from RPC and a callback
 * to invalidate it.
 */
export function useObjectQuery(
    params: GetObjectParams,
    options?: UseObjectQueryOptions,
): [UseObjectQueryResponse, InvalidateUseObjectQuery] {
    const ctx = useRtdClientContext();
    const client = useQueryClient();
    const response = useRtdClientQuery("getObject", params, options);

    const invalidate = async () => {
        await client.invalidateQueries({
            queryKey: [ctx.network, "getObject", params],
        });
    };

    return [response, invalidate];
}
