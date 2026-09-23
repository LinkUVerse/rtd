// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { Transaction } from '@linku/rtd/transactions';
import { RtdGrpcClient } from '@linku/rtd/grpc';
import { Ed25519Keypair } from '@linku/rtd/keypairs/ed25519';

const client = new RtdGrpcClient({ baseUrl: 'https://fullnode.testnet.rtd.io:443', network: 'testnet' });
const keypair = new Ed25519Keypair();
const playerAddress = '0xPlayer...';

// docs::#result-chaining
const tx = new Transaction();

// Step 1: Create a new game character.
const [character] = tx.moveCall({
	target: '0xGAME::character::create',
	arguments: [tx.pure('string', 'Hero')],
});

// Step 2: Mint a sword and equip it to the character.
const [sword] = tx.moveCall({
	target: '0xGAME::items::mint_sword',
	arguments: [tx.pure('u64', 100)], // attack power
});

tx.moveCall({
	target: '0xGAME::character::equip',
	arguments: [character, sword],
});

// Step 3: Transfer the character to the player.
tx.transferObjects([character], tx.pure.address(playerAddress));

await client.signAndExecuteTransaction({ signer: keypair, transaction: tx });
// docs::/#result-chaining
