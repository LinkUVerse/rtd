// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { useRtdClient, useRtdClientContext } from '@linku/dapp-kit';
import { KioskClient, Network } from '@linku/kiosk';
import { createContext, ReactNode, useContext, useMemo } from 'react';

export const KioskClientContext = createContext<KioskClient | undefined>(undefined);

export function KioskClientProvider({ children }: { children: ReactNode }) {
	const rtdClient = useRtdClient();
	const { network } = useRtdClientContext();
	const kioskClient = useMemo(
		() =>
			new KioskClient({
				client: rtdClient,
				network: network as Network,
			}),
		[rtdClient, network],
	);

	return <KioskClientContext.Provider value={kioskClient}>{children}</KioskClientContext.Provider>;
}

export function useKioskClient() {
	const kioskClient = useContext(KioskClientContext);
	if (!kioskClient) {
		throw new Error('kioskClient not setup properly.');
	}
	return kioskClient;
}
