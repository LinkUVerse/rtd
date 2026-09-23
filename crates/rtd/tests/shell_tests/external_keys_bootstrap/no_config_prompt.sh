# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

# If no Rtd config exists yet, external-keys should prompt, create the config,
# and bootstrap the external keystore files before attempting the signer call.
export RTD_CONFIG_DIR="$PWD/config"

set -o pipefail
echo "" | rtd external-keys list-keys missing-signer 2>&1 \
  | sed 's|No rtd config found in `.*|No rtd config found in `<SANDBOX_DIR>/config/client.yaml`, create one [Y/n]?|g' \
  | sed 's/Generated new keypair.*$/Generated new keypair <REDACTED>/g' \
  | sed 's/recovery phrase : \[.*\]/recovery phrase : <REDACTED>/g' \
  | sed 's|Created \".*\"|Created \"<SANDBOX_DIR>/config/client.yaml\"|g' \
  | sed 's/program not found.*/<REDACTED>/g' \
  | sed 's/No such file or directory (os error 2)/<REDACTED>/g'
echo "exit_code: $?"

echo ""
echo "check for created files:"
ls config

echo ""
echo "client config:"
sed 's|  File: .*|  File: <SANDBOX_DIR>/config/rtd.keystore|g' config/client.yaml \
  | sed 's|  External: .*|  External: <SANDBOX_DIR>/config/external.keystore|g' \
  | sed 's/active_address:.*$/active_address: <REDACTED>/g'
