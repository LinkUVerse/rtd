// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
import React from "react";
import ReactDOM from "react-dom/client";
import "@linku/dapp-kit/dist/index.css";
import "@radix-ui/themes/styles.css";
import "./styles/base.css";

import { getFullnodeUrl } from "@linku/rtd/client";
import {
  RtdClientProvider,
  WalletProvider,
  createNetworkConfig,
} from "@linku/dapp-kit";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Theme } from "@radix-ui/themes";
import { router } from "@/routes/index.tsx";

import { RouterProvider } from "react-router-dom";

const queryClient = new QueryClient();

const { networkConfig } = createNetworkConfig({
  localnet: { url: getFullnodeUrl("localnet") },
  ...(import.meta.env.VITE_RTD_DEVNET_RPC_URL ? { devnet: { url: import.meta.env.VITE_RTD_DEVNET_RPC_URL } } : {}),
  ...(import.meta.env.VITE_RTD_TESTNET_RPC_URL ? { testnet: { url: import.meta.env.VITE_RTD_TESTNET_RPC_URL } } : {}),
  ...(import.meta.env.VITE_RTD_MAINNET_RPC_URL ? { mainnet: { url: import.meta.env.VITE_RTD_MAINNET_RPC_URL } } : {}),
});

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Theme appearance="light">
      <QueryClientProvider client={queryClient}>
        <RtdClientProvider networks={networkConfig} defaultNetwork="localnet">
          <WalletProvider autoConnect>
            <RouterProvider router={router} />
          </WalletProvider>
        </RtdClientProvider>
      </QueryClientProvider>
    </Theme>
  </React.StrictMode>,
);
