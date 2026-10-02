# Example - Use a MEV bot to Clear Expired DeepBook Orders

## Overview

This example shows how to use a MEV bot to clear expired DeepBook orders. The bot scans DeepBook pools for expired orders and cancels them. The example bot uses the Rtd TS SDK to interact with the Rtd Full node and retrieve on chain DeepBook data.

## How to run

This historical DeepBook V2 example is inactive until that protocol is
separately deployed on RTD. Set `RTD_JSON_RPC_URL`,
`RTD_DEEPBOOK_V2_PACKAGE_ID`, and `RTD_INSPECT_SENDER` to values from one RTD
network before starting. The script contains no usable upstream deployment ID.

```bash
pnpm start
```
