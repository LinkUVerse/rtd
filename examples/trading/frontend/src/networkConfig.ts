// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
import { getFullnodeUrl } from "@linku/rtd/client";
import { createNetworkConfig } from "@linku/dapp-kit";

const { networkConfig, useNetworkVariable, useNetworkVariables } =
  createNetworkConfig({
    localnet: { url: getFullnodeUrl("localnet") },
    ...(import.meta.env.VITE_RTD_DEVNET_RPC_URL ? { devnet: { url: import.meta.env.VITE_RTD_DEVNET_RPC_URL } } : {}),
    ...(import.meta.env.VITE_RTD_TESTNET_RPC_URL ? { testnet: { url: import.meta.env.VITE_RTD_TESTNET_RPC_URL } } : {}),
    ...(import.meta.env.VITE_RTD_MAINNET_RPC_URL ? { mainnet: { url: import.meta.env.VITE_RTD_MAINNET_RPC_URL } } : {}),
  });

export { useNetworkVariable, useNetworkVariables, networkConfig };
