// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { RtdGrpcClient } from '@linku/rtd/grpc';

const url = (globalThis as { process?: { env?: Record<string, string | undefined> } }).process
	?.env?.RTD_GRPC_URL;
if (!url) throw new Error('RTD_GRPC_URL must point to your RTD Testnet node');

export const client = new RtdGrpcClient({ baseUrl: url, network: 'testnet' });
