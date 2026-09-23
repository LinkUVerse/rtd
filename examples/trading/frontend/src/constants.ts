// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

function requiredPackageId(name: string): string {
  const value = import.meta.env[name]?.trim();
  if (!value || !/^0x[0-9a-fA-F]{64}$/.test(value)) {
    throw new Error(`${name} must contain a package ID deployed on this RTD network`);
  }
  return value;
}

const escrowContract = { packageId: requiredPackageId("VITE_RTD_ESCROW_PACKAGE_ID") };
const demoContract = { packageId: requiredPackageId("VITE_RTD_DEMO_PACKAGE_ID") };

export enum QueryKey {
  Locked = "locked",
  Escrow = "escrow",
  GetOwnedObjects = "getOwnedObjects",
}

export const CONSTANTS = {
  escrowContract: {
    ...escrowContract,
    lockedType: `${escrowContract.packageId}::lock::Locked`,
    lockedKeyType: `${escrowContract.packageId}::lock::Key`,
    lockedObjectDFKey: `${escrowContract.packageId}::lock::LockedObjectKey`,
  },
  demoContract: {
    ...demoContract,
    demoBearType: `${demoContract.packageId}::demo_bear::DemoBear`,
  },
  apiEndpoint: import.meta.env.VITE_RTD_API_URL || "http://localhost:3000/",
};
