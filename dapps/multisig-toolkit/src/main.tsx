// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import '@linku/dapp-kit/dist/index.css';
import './index.css';
import '@fontsource-variable/inter';
import '@fontsource-variable/red-hat-mono';

import { RtdClientProvider, WalletProvider } from '@linku/dapp-kit';
import { getFullnodeUrl } from '@linku/rtd/client';
import { QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import ReactDOM from 'react-dom/client';
import { RouterProvider } from 'react-router-dom';

import { queryClient } from './lib/queryClient';
import { router } from './routes';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
	<React.StrictMode>
		<QueryClientProvider client={queryClient}>
			<RtdClientProvider
				defaultNetwork="rtd:mainnet"
				networks={{
					'rtd:testnet': { url: getFullnodeUrl('testnet') },
					'rtd:mainnet': { url: getFullnodeUrl('mainnet') },
					'rtd:devnet': { url: getFullnodeUrl('devnet') },
				}}
			>
				<WalletProvider>
					<RouterProvider router={router} />
				</WalletProvider>
			</RtdClientProvider>
		</QueryClientProvider>
	</React.StrictMode>,
);
