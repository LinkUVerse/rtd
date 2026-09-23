// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// docs::#client
import { RtdGrpcClient } from '@linku/rtd/grpc';
import { Ed25519Keypair } from '@linku/rtd/keypairs/ed25519';
import { decodeRtdPrivateKey } from '@linku/rtd/cryptography';
import { deepbook, type DeepBookClient, type BalanceManager } from '@linku/deepbook-v3';
import type { ClientWithExtensions } from '@linku/rtd/client';

export type DeepBookTestnetClient = ClientWithExtensions<{ deepbook: DeepBookClient }>;

export function getKeypair(privateKey: string): Ed25519Keypair {
	const { secretKey } = decodeRtdPrivateKey(privateKey);
	return Ed25519Keypair.fromSecretKey(secretKey);
}

// The DeepBook SDK must contain deployment records for this RTD Testnet.
// Once verified, reference pools and coins by key instead of hardcoding IDs.
export function deepbookClient(
	address: string,
	balanceManagers?: { [key: string]: BalanceManager },
): DeepBookTestnetClient {
	const baseUrl = (globalThis as { process?: { env?: Record<string, string | undefined> } })
		.process?.env?.RTD_DEEPBOOK_GRPC_URL;
	if (!baseUrl) throw new Error('RTD_DEEPBOOK_GRPC_URL is required for an RTD DeepBook deployment');
	return new RtdGrpcClient({
		network: 'testnet',
		baseUrl,
	}).$extend(deepbook({ address, balanceManagers }));
}
// docs::/#client
