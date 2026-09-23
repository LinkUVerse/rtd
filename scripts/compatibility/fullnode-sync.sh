#!/bin/bash
# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

set -e

DEFAULT_NETWORK="testnet"
CLEAN=0
LOG_LEVEL="info"
RTD_RUN_PATH="${RTD_RUN_PATH:-/opt/rtd}"
VERBOSE=""

function cleanup {
    echo "Performing exit cleanup..."
    [ -n "$RTD_NODE_PID" ] && kill "$RTD_NODE_PID" && echo "Shutdown rtd-node process running on pid $RTD_NODE_PID"
}

trap cleanup EXIT

while getopts "hvn:e:p:t:" OPT; do
    case $OPT in
        p) 
            RTD_BIN_PATH=$OPTARG ;;
        v)
            LOG_LEVEL="rtd=debug,error"
            VERBOSE="--verbose" 
            ;;
        n)
            NETWORK=$OPTARG ;;
        e)
            END_EPOCH=$OPTARG ;;
        t)
            EPOCH_TIMEOUT=$OPTARG ;;
        h)
            >&2 echo "Usage: $0 [-h] [-p] [-v] [-n NETWORK] [-e END_EPOCH] [-t EPOCH_TIMEOUT]"
            >&2 echo ""
            >&2 echo "Options:"
            >&2 echo " -p                 Path to rtd binary to run. If unspecified, will build from source."
            >&2 echo " -v                 Run with verbose logging."
            >&2 echo " -n NETWORK         The network to run the fullnode on."
            >&2 echo "                    (Default: ${DEFAULT_NETWORK})"
            >&2 echo " -e END_EPOCH       EpochID at which to stop syncing and declare success."
            >&2 echo "                    If unspecified or -1, will use current epoch of NETWORK."
            >&2 echo " -t EPOCH_TIMEOUT   Number of minutes to wait until epoch advancement before timing out."
            >&2 exit 0
            ;;
        \?)
            >&2 echo "Unrecognized option '$OPTARG'"
            exit 1
            ;;
    esac
done

if [[ -z "$NETWORK" ]]; then
    NETWORK=$DEFAULT_NETWORK
elif [[ "$NETWORK" != "testnet" && "$NETWORK" != "devnet" ]]; then
    >&2 echo "Invalid network ${NETWORK}"
    exit 1
fi

if [[ ! -f "${RTD_RUN_PATH}/genesis.blob" || ! -f "${RTD_RUN_PATH}/fullnode.yaml" ]]; then
    >&2 echo "Provide a verified RTD genesis.blob and fullnode.yaml in ${RTD_RUN_PATH} before syncing."
    exit 1
fi

if [[ -z $RTD_BIN_PATH ]]; then
    echo "Building rtd..."
    cargo build --release --bin rtd-node
    RTD_BIN_PATH="target/release/rtd-node"
    echo "Done"
fi


echo "Starting rtd-node..."
RUST_LOG=$LOG_LEVEL $RTD_BIN_PATH --config-path ${RTD_RUN_PATH}/fullnode.yaml &
RTD_NODE_PID=$!

# start monitoring script
END_EPOCH_ARG=""
if [[ ! -z $END_EPOCH && $END_EPOCH != -1 ]]; then
    END_EPOCH_ARG="--end-epoch $END_EPOCH"
fi

EPOCH_TIMEOUT_ARG=""
if [[ ! -z $EPOCH_TIMEOUT ]]; then
    EPOCH_TIMEOUT_ARG="--epoch-timeout $EPOCH_TIMEOUT"
fi

./scripts/compatibility/monitor_synced.py $END_EPOCH_ARG $EPOCH_TIMEOUT_ARG --env $NETWORK $VERBOSE

kill $RTD_NODE_PID
exit 0
