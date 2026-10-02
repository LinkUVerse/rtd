// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// docs::#setup
import React, { useState, useEffect } from "react";
import {
  createNetworkConfig,
  RtdClientProvider,
  useRtdClient,
  ConnectButton,
  useCurrentAccount,
  useSignAndExecuteTransaction,
  WalletProvider,
} from "@linku/dapp-kit";
import { Transaction } from "@linku/rtd/transactions";
import { getFullnodeUrl } from "@linku/rtd/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import "@linku/dapp-kit/dist/index.css";
// docs::/#setup

const network = process.env.RTD_NETWORK || "localnet";
const rpcUrl = process.env.RTD_JSON_RPC_URL ||
  (network === "localnet" ? getFullnodeUrl("localnet") : undefined);
if (!rpcUrl) throw new Error("RTD_JSON_RPC_URL is required for a remote RTD network");

const { networkConfig } = createNetworkConfig({
  [network]: { url: rpcUrl },
});

// Create a new QueryClient for managing and caching asynchronous queries
const queryClient = new QueryClient();

// Set this to the coin type actually deployed on the selected RTD network.
const USDC_TYPE = process.env.RTD_USDC_TYPE;

function HomeContent() {
  // docs::#state
  // Use the wallet kit to get the current account and transaction signing function
  const currentAccount = useCurrentAccount();
  const { mutate: signAndExecuteTransaction } = useSignAndExecuteTransaction();
  // Get the Rtd client for interacting with the Rtd network
  const rtdClient = useRtdClient();
  const [open, setOpen] = useState(false);
  const [connected, setConnected] = useState(false);
  const [amount, setAmount] = useState("");
  const [recipientAddress, setRecipientAddress] = useState("");
  const [txStatus, setTxStatus] = useState("");
  // docs::/#state

  // docs::#useeffect
  useEffect(() => {
    setConnected(!!currentAccount);
  }, [currentAccount]);
  // docs::/#useeffect

  const handleSendTokens = async () => {
    if (!currentAccount || !amount || !recipientAddress) {
      setTxStatus("Please connect wallet and fill in all fields");
      return;
    }
    if (!USDC_TYPE || !/^0x[0-9a-fA-F]{1,64}::[A-Za-z_][A-Za-z_0-9]*::[A-Za-z_][A-Za-z_0-9]*$/.test(USDC_TYPE)) {
      setTxStatus("Configure RTD_USDC_TYPE with a coin type deployed on this RTD network");
      return;
    }
    try {
      // Fetch USDC coins owned by the current account
      // This uses the RtdClient to get coins of the specified type owned by the current address
      const { data: coins } = await rtdClient.getCoins({
        owner: currentAccount.address,
        coinType: USDC_TYPE,
      });
      if (coins.length === 0) {
        setTxStatus("No USDC coins found in your wallet");
        return;
      }
      // Create a new transaction block
      // Transaction is used to construct and execute transactions on Rtd
      const tx = new Transaction();
      // Convert amount to smallest unit (6 decimals)
      const amountInSmallestUnit = BigInt(parseFloat(amount) * 1_000_000);
      // Split the coin and get a new coin with the specified amount
      // This creates a new coin object with the desired amount to be transferred
      const [coin] = tx.splitCoins(coins[0].coinObjectId, [
        tx.pure.u64(amountInSmallestUnit),
      ]);
      // Transfer the split coin to the recipient
      // This adds a transfer operation to the transaction block
      tx.transferObjects([coin], tx.pure.address(recipientAddress));
      // Sign and execute the transaction block
      // This sends the transaction to the network and waits for it to be executed
      const result = await signAndExecuteTransaction(
        {
          transaction: tx,
        },
        {
          onSuccess: (result) => {
            console.log("Transaction result:", result);
            setTxStatus(`Transaction successful. Digest: ${result.digest}`);
          },
        }
      );
    } catch (error) {
      console.error("Error sending tokens:", error);
      setTxStatus(
        `Error: ${error instanceof Error ? error.message : "Unknown error"}`
      );
    }
  };

  // docs::#ui
  return (
    <main className="mainwrapper">
      <div className="outerwrapper">
        <h1 className="h1">RTD USDC Sender</h1>
        <ConnectButton />
        {connected && currentAccount && (
          <p className="status">Connected: {currentAccount.address}</p>
        )}
        <div className="form">
          <input
            type="text"
            placeholder="Amount (in USDC)"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            className="input"
          />
          <input
            type="text"
            placeholder="Recipient Address"
            value={recipientAddress}
            onChange={(e) => setRecipientAddress(e.target.value)}
            className="input"
          />
          <button
            onClick={handleSendTokens}
            disabled={!connected}
            className={`${
              connected && amount && recipientAddress
                ? "connected"
                : "notconnected"
            } transition`}
          >
            Send USDC
          </button>
        </div>
        {txStatus && <p className="status">{txStatus}</p>}
      </div>
    </main>
  );
  // docs::/#ui
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <RtdClientProvider networks={networkConfig} defaultNetwork={network}>
        <WalletProvider>
          <HomeContent />
        </WalletProvider>
      </RtdClientProvider>
    </QueryClientProvider>
  );
}

export default App;
