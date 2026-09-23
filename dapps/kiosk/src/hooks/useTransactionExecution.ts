// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { useSignTransaction, useRtdClient } from '@linku/dapp-kit';
import { RtdTransactionBlockResponseOptions } from '@linku/rtd/client';
import { Transaction } from '@linku/rtd/transactions';

// A helper to execute transactions by:
// 1. Signing them using the wallet
// 2. Executing them using the rpc provider
export function useTransactionExecution() {
	const provider = useRtdClient();

	// sign transaction from the wallet
	const { mutateAsync: signTransaction } = useSignTransaction();

	// tx: Transaction
	const signAndExecute = async ({
		tx,
		options = { showEffects: true },
	}: {
		tx: Transaction;
		options?: RtdTransactionBlockResponseOptions | undefined;
	}) => {
		const signedTx = await signTransaction({ transaction: tx });

		const res = await provider.executeTransactionBlock({
			transactionBlock: signedTx.bytes,
			signature: signedTx.signature,
			options,
		});

		const status = res.effects?.status?.status === 'success';

		if (status) return true;
		else throw new Error('Transaction execution failed.');
	};

	return { signAndExecute };
}
