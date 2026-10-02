// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { createNetworkConfig } from "@linku/dapp-kit";
import { getFullnodeUrl } from "@linku/rtd/client";

import DevnetPackage from "./env.devnet.ts";
import LocalnetPackage from "./env.localnet.ts";
import MainnetPackage from "./env.mainnet.ts";
import TestnetPackage from "./env.testnet.ts";

const { networkConfig, useNetworkVariable } = createNetworkConfig({
    localnet: {
        url: getFullnodeUrl("localnet"),
        variables: {
            explorer: explorerUrl(import.meta.env.VITE_RTD_LOCALNET_EXPLORER_URL),
            ...LocalnetPackage,
        },
    },
    ...(import.meta.env.VITE_RTD_DEVNET_RPC_URL ? { devnet: {
        url: import.meta.env.VITE_RTD_DEVNET_RPC_URL,
        variables: {
            explorer: explorerUrl(import.meta.env.VITE_RTD_DEVNET_EXPLORER_URL),
            ...DevnetPackage,
        },
    } } : {}),
    ...(import.meta.env.VITE_RTD_TESTNET_RPC_URL ? { testnet: {
        url: import.meta.env.VITE_RTD_TESTNET_RPC_URL,
        variables: {
            explorer: explorerUrl(import.meta.env.VITE_RTD_TESTNET_EXPLORER_URL),
            ...TestnetPackage,
        },
    } } : {}),
    ...(import.meta.env.VITE_RTD_MAINNET_RPC_URL ? { mainnet: {
        url: import.meta.env.VITE_RTD_MAINNET_RPC_URL,
        variables: {
            explorer: explorerUrl(import.meta.env.VITE_RTD_MAINNET_EXPLORER_URL),
            ...MainnetPackage,
        },
    } } : {}),
});

function explorerUrl(base: string | undefined): (id: string) => string | undefined {
    return (id) => base?.trim() ? `${base.replace(/\/$/, "")}/object/${encodeURIComponent(id)}` : undefined;
}

export { networkConfig, useNetworkVariable };
