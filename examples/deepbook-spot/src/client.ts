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

// Testnet DeepBook client. The SDK ships Testnet package, coin, and pool
// constants, so you reference pools and coins by key (for example 'DEEP_RTD' or
// 'DEEP') instead of hardcoding IDs. Read-only calls work without a manager.
export function deepbookClient(
	address: string,
	balanceManagers?: { [key: string]: BalanceManager },
): DeepBookTestnetClient {
	return new RtdGrpcClient({
		network: 'testnet',
		baseUrl: 'https://fullnode.testnet.rtd.io:443',
	}).$extend(deepbook({ address, balanceManagers }));
}
// docs::/#client
