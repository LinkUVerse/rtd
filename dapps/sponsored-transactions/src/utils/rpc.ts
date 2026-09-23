// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import { getFullnodeUrl, RtdClient } from '@linku/rtd/client';

export const client = new RtdClient({ url: getFullnodeUrl('testnet') });
