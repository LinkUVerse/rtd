// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// docs::#config
// Pyth and Wormhole must first be deployed on the RTD network you use.
// Never reuse package, state-object, or feed IDs from another chain.
const environment = (globalThis as { process?: { env?: Record<string, string | undefined> } }).process
	?.env;

function required(name: string): string {
	const value = environment?.[name]?.trim();
	if (!value) {
		throw new Error(`${name} is required for the RTD oracle example`);
	}
	return value;
}

function objectId(name: string): string {
	const value = required(name);
	if (!/^0x[0-9a-fA-F]{64}$/.test(value)) {
		throw new Error(`${name} must be a 32-byte RTD object ID`);
	}
	return value;
}

export const TESTNET = {
	// The JSON-RPC endpoint must serve the same RTD network as these objects.
	rpcUrl: required('RTD_JSON_RPC_URL'),
	pythStateId: objectId('RTD_PYTH_STATE_ID'),
	wormholeStateId: objectId('RTD_WORMHOLE_STATE_ID'),
	hermesEndpoint: required('RTD_HERMES_ENDPOINT'),
	// Keeper push interval. Each push is a transaction that costs gas plus the
	// Pyth base fee, so trade freshness against cost.
	keeperIntervalMs: 15_000,
	// Price feed IDs (chain-agnostic). Add the feeds your app consumes.
	feeds: {
		'RTD/USD': objectId('RTD_PYTH_FEED_RTD_USD_ID'),
	},
} as const;
// docs::/#config
