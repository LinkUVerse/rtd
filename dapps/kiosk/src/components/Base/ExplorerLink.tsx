// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { useState } from 'react';

type LinkOptions = { address: string; text: string } | { object: string; text: string };

/** Display an RTD identifier and allow copying it without assuming an explorer network. */
export function ExplorerLink(opts: LinkOptions) {
	const [copied, setCopied] = useState<boolean>(false);

	const copyToClipboard = async () => {
		await navigator.clipboard.writeText('address' in opts ? opts.address : opts.object);
		setCopied(true);
		setTimeout(() => {
			setCopied(false);
		}, 3000);
	};
	return (
		<>
			<span>{opts.text}</span>
			<button
				className="!p-1 ml-3 text-xs ease-in-out duration-300 rounded border border-transparent bg-gray-200"
				onClick={copyToClipboard}
			>
				{copied ? 'copied' : 'copy'}
			</button>
		</>
	);
}
