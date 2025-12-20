# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

rtd client --client.config config.yaml switch --env base

rtd client --client.config config.yaml envs
rtd client --client.config config.yaml --client.env one envs
rtd client --client.config config.yaml --client.env two envs

rtd client --client.config config.yaml active-env
rtd client --client.config config.yaml --client.env one active-env
rtd client --client.config config.yaml --client.env two active-env

# Unknown name -- Should give you None and nothing active
rtd client --client.config config.yaml --client.env not_an_env envs
rtd client --client.config config.yaml --client.env not_an_env active-env
