// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// docs::#client
import { RtdGrpcClient } from '@linku/rtd/grpc';
import { Ed25519Keypair } from '@linku/rtd/keypairs/ed25519';
import { decodeRtdPrivateKey } from '@linku/rtd/cryptography';
import {
	deepbook,
	type DeepBookClient,
	type MarginManager,
	type BalanceManager,
} from '@linku/deepbook-v3';
import type { ClientWithExtensions } from '@linku/rtd/client';

export type DeepBookMarginClient = ClientWithExtensions<{ deepbook: DeepBookClient }>;

export function getKeypair(privateKey: string): Ed25519Keypair {
	const { secretKey } = decodeRtdPrivateKey(privateKey);
	return Ed25519Keypair.fromSecretKey(secretKey);
}

// The SDK must contain RTD Testnet margin package IDs, pools, and Pyth config.
// After verifying that deployment, reference pools, coins, and managers by key.
// Read-only calls (risk parameters, pool liquidity)
// work without a manager; borrowing and trading through a margin manager need
// `marginManagers`. Supplying to a margin pool and staking use a spot
// `BalanceManager`, so pass `balanceManagers` when you compose those legs.
export function marginClient(
	address: string,
	options?: {
		marginManagers?: { [key: string]: MarginManager };
		balanceManagers?: { [key: string]: BalanceManager };
	},
): DeepBookMarginClient {
	const baseUrl = (globalThis as { process?: { env?: Record<string, string | undefined> } })
		.process?.env?.RTD_DEEPBOOK_GRPC_URL;
	if (!baseUrl) throw new Error('RTD_DEEPBOOK_GRPC_URL is required for an RTD DeepBook deployment');
	return new RtdGrpcClient({
		network: 'testnet',
		baseUrl,
	}).$extend(deepbook({ address, ...options }));
}
// docs::/#client
