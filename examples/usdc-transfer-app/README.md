# RTD USDC transfer example

USDC is a third-party coin and is not provided by RTD genesis. Before running
this app, deploy or verify a USDC coin on the target RTD network and set
`RTD_USDC_TYPE` to its actual Move type. The app defaults to localnet. For a
remote network, set `RTD_NETWORK` and `RTD_JSON_RPC_URL` to the same RTD network.
There is no upstream USDC package ID in this example.
