// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// docs::#config
import { getConfig, getDeployment, getUnits } from '@linku/deepbook-v3/predict';

const environment = (globalThis as { process?: { env?: Record<string, string | undefined> } })
	.process?.env;
function required(name: string): string {
	const value = environment?.[name]?.trim();
	if (!value) throw new Error(`${name} is required for an RTD DeepBook Predict deployment`);
	return value;
}

export const NETWORK = required('RTD_PREDICT_NETWORK') as 'testnet' | 'mainnet';
if (NETWORK !== 'testnet' && NETWORK !== 'mainnet') {
	throw new Error('RTD_PREDICT_NETWORK must be testnet or mainnet');
}

export const FULLNODE_URL = required('RTD_PREDICT_GRPC_URL');

// One underlying is live on this deployment.
export const UNDERLYING = 'BTC';

// The SDK carries the IDs of whichever deployments its release was cut against.
// Assert the name at startup, so a later SDK release that moves a network to a
// new deployment fails loudly here rather than quietly trading against a
// deployment these examples were never checked against.
export const EXPECTED_DEPLOYMENT = required('RTD_PREDICT_DEPLOYMENT');
const expectedChainId = required('RTD_CHAIN_ID');

export const DEPLOYMENT = getDeployment(NETWORK);

if (DEPLOYMENT.deployment !== EXPECTED_DEPLOYMENT || DEPLOYMENT.chainId !== expectedChainId) {
	throw new Error(
		`Expected DeepBook Predict deployment ${EXPECTED_DEPLOYMENT}, got ` +
			`${DEPLOYMENT.deployment} (chain ${DEPLOYMENT.chainId}, ` +
			`deepbookv3 commit ${DEPLOYMENT.sourceCommit}).`,
	);
}

// The SDK deployment record supplies package and object IDs, quote-coin type,
// and oracle IDs. Use it only after its chain ID matches this RTD deployment.
// Always read the quote coin from `CONFIG.quoteCoinType`.
export const CONFIG = getConfig(NETWORK);

// Scale constants the deployment owns: position quantities are whole
// `positionLotSize` lots, amounts are `quoteCoinDecimals`-decimal USDC, and
// probabilities, prices, and rates are fixed point at `fixedPointScale`.
export const UNITS = getUnits(NETWORK);
// docs::/#config
