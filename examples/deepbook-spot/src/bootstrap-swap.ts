// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// docs::#bootstrap-swap
import { Transaction } from '@linku/rtd/transactions';
import type { DeepBookTestnetClient } from './client.js';

// Swap RTD for DEEP on the DEEP_RTD Testnet pool. That pool is whitelisted
// (zero fee) and the swap needs no BalanceManager, so it bootstraps DEEP from
// faucet RTD. DEEP is the base and RTD the quote, so a quote-for-base swap
// spends RTD and returns DEEP. Set `minDeepOut` to a nonzero value (for example
// 99% of getQuantityOut's `baseOut`): with `minOut: 0`, a thin or empty book
// silently returns your RTD unfilled instead of reverting.
export function swapRtdForDeep(
	client: DeepBookTestnetClient,
	rtdAmount: number,
	minDeepOut: number,
	recipient: string,
): Transaction {
	const tx = new Transaction();
	const [deepOut, rtdRemainder, deepFee] = tx.add(
		client.deepbook.deepBook.swapExactQuoteForBase({
			poolKey: 'DEEP_RTD',
			amount: rtdAmount,
			deepAmount: 0,
			minOut: minDeepOut,
		}),
	);
	tx.transferObjects([deepOut, rtdRemainder, deepFee], recipient);
	return tx;
}
// docs::/#bootstrap-swap
