#!/bin/bash
# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

echo "Start Rosetta online server"
rtd-rosetta start-online-server --data-path ./data &

echo "Start Rosetta offline server"
rtd-rosetta start-offline-server &
