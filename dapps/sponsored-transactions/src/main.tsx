// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { RtdClientProvider, WalletProvider } from '@linku/dapp-kit';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import React from 'react';
import ReactDOM from 'react-dom/client';

import { App } from './App';

import '@linku/dapp-kit/dist/index.css';
import './index.css';

import { getFullnodeUrl } from '@linku/rtd/client';

const queryClient = new QueryClient();

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
	<React.StrictMode>
		<QueryClientProvider client={queryClient}>
			<RtdClientProvider
				defaultNetwork="testnet"
				networks={{ testnet: { url: getFullnodeUrl('testnet') } }}
			>
				<WalletProvider enableUnsafeBurner>
					<App />
				</WalletProvider>
			</RtdClientProvider>
		</QueryClientProvider>
	</React.StrictMode>,
);
