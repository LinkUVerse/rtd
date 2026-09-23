// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { ACTIVE_NETWORK, publishPackage } from '../rtd-utils';

/// A demo showing how we could publish the escrow contract
/// and our DEMO objects contract.
///
/// We're publishing both as part of our demo.
(async () => {
	await publishPackage({
		packagePath: __dirname + '/../../contracts/escrow',
		network: ACTIVE_NETWORK,
		exportFileName: 'escrow-contract',
	});

	await publishPackage({
		packagePath: __dirname + '/../../contracts/demo',
		network: ACTIVE_NETWORK,
		exportFileName: 'demo-contract',
	});
})();
