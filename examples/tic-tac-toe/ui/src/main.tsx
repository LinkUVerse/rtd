// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import React from "react";
import ReactDOM from "react-dom/client";

import "@linku/dapp-kit/dist/index.css";
import "@radix-ui/themes/styles.css";

import { RtdClientProvider, WalletProvider } from "@linku/dapp-kit";
import { Theme } from "@radix-ui/themes";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Toaster } from "react-hot-toast";

import App from "./App.tsx";
import { networkConfig } from "./config";

const queryClient = new QueryClient();

ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
        <Theme appearance="light">
            <QueryClientProvider client={queryClient}>
                <RtdClientProvider networks={networkConfig} defaultNetwork="testnet">
                    <WalletProvider autoConnect>
                        <>
                            <Toaster position="top-center" />
                            <App />
                        </>
                    </WalletProvider>
                </RtdClientProvider>
            </QueryClientProvider>
        </Theme>
    </React.StrictMode>,
);
