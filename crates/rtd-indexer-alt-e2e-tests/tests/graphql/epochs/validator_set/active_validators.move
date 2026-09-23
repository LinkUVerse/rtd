// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//# init --protocol-version 108 --accounts A --simulator

//# run-graphql
{
  epoch(epochId: 0) {
    epochId
    validatorSet {
      activeValidators {
        pageInfo {
         hasPreviousPage
         hasNextPage
         startCursor
         endCursor
        }

        nodes {
          contents {
            json

            address: extract(path: "metadata.rtd_address") {
              asAddress {
                balances {
                  nodes {
                    coinType { repr }
                    totalBalance
                  }
                }

                objects {
                  nodes {
                    contents {
                      type { repr }
                      json
                    }
                  }
                }
              }
            }
          }

          atRisk
        }
      }
    }
  }
}
