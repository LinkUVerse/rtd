// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// docs::#client
import { RtdClient } from '@linku/rtd/client';
import { RtdPythClient, RtdPriceServiceConnection } from '@pythnetwork/pyth-rtd-js';
import { TESTNET } from './config.js';

// A Rtd RPC client, the Pyth on-chain client (which builds the Wormhole verify
// plus price-update commands), and a Hermes connection (which serves the signed
// off-chain price updates). Pyth on Rtd is a pull oracle: you fetch an update
// from Hermes and apply it on-chain in the same transaction that reads it.
export function rtdClient(): RtdClient {
	return new RtdClient({ url: TESTNET.rpcUrl });
}

export function pythClient(rtd: RtdClient): RtdPythClient {
	return new RtdPythClient(rtd, TESTNET.pythStateId, TESTNET.wormholeStateId);
}

export function hermes(): RtdPriceServiceConnection {
	return new RtdPriceServiceConnection(TESTNET.hermesEndpoint);
}
// docs::/#client
