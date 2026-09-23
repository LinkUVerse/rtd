#!/bin/bash
# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

NETWORK="$1"

git fetch -q || exit 1

if [ -z "$RELEASED_COMMIT" ]; then
  if [ -z "$RTD_PROMETHEUS_URL" ]; then
    echo "Error: set RTD_PROMETHEUS_URL to the verified RTD Prometheus query endpoint"
    exit 1
  fi

  REPO_ROOT=$(git rev-parse --show-toplevel)
  cd $REPO_ROOT

  if [ "$NETWORK" != "devnet" ] && [ "$NETWORK" != "testnet" ] && [ "$NETWORK" != "mainnet" ]; then
    echo "Invalid network: $NETWORK"
    echo "Usage: check-protocol-compatibility.sh <devnet|testnet|mainnet> <git|prometheus>"
    exit 1
  fi

  VERSIONS=$(curl -fsS -G "$RTD_PROMETHEUS_URL" --data-urlencode "query=uptime{network=\"$NETWORK\"}" | jq -r '.data.result[].metric.version' | sort | uniq -c | sort -rn)
  TOP_VERSION=$(echo "$VERSIONS" | head -n 1 | awk '{print $2}')

  echo "Found following versions on $NETWORK:"
  echo "$VERSIONS"
  echo ""

  # Versions look like "1.0.0-ae1212baf8"; the suffix is the commit the node
  # was built from. Use the most frequent version exactly when its commit
  # resolves here. During a private security release that suffix is a
  # rtd-private commit that does not resolve in this repo: approximate its
  # public base instead — the highest publicly-resolvable version that is not
  # newer than the dominant one (ties broken by node count). The semver
  # ceiling keeps a stray node on a newer public build from hijacking the
  # baseline, and "highest" (rather than most frequent) tracks the private
  # build's lineage even when an older public cohort has more nodes. This
  # check gates the public source, so the public base carries the protocol
  # snapshots being validated.
  TOP_SHA=$(echo "$TOP_VERSION" | cut -d- -f2)
  if git cat-file -e "${TOP_SHA}^{commit}" 2>/dev/null; then
    RELEASED_COMMIT="$TOP_SHA"
    echo "Using most frequent version $TOP_VERSION for compatibility check"
  else
    TOP_BASE=${TOP_VERSION%%-*}
    while read -r semver sha count fullver; do
      # skip versions newer than the dominant one
      if [ "$semver" != "$TOP_BASE" ] && \
         [ "$(printf '%s\n%s\n' "$semver" "$TOP_BASE" | sort -V | tail -n1)" = "$semver" ]; then
        continue
      fi
      if git cat-file -e "${sha}^{commit}" 2>/dev/null; then
        RELEASED_COMMIT="$sha"
        echo "WARNING: most frequent version $TOP_VERSION does not resolve in this repo (private release?)"
        echo "Falling back to highest publicly-resolvable version $fullver ($count nodes) for compatibility check"
        break
      fi
    done < <(echo "$VERSIONS" | awk '{split($2, a, "-"); print a[1], a[2], $1, $2}' | sort -k1,1Vr -k3,3nr)
  fi

  if [ -z "$RELEASED_COMMIT" ]; then
    echo "Error: no version on $NETWORK resolves to a commit in this repo"
    exit 1
  fi
fi

SOURCE_COMMIT=$(git rev-parse HEAD)
SOURCE_BRANCH=$(git branch -a --contains "$SOURCE_COMMIT" | head -n 1 | cut -d' ' -f2-)

echo "Source commit: $SOURCE_COMMIT"
echo "Source branch: $SOURCE_BRANCH"

echo "Checking protocol compatibility with $NETWORK ($RELEASED_COMMIT)"

# put code to check if git client is clean into function
function check_git_clean {
  message="$1"
  path="$2"
  # if any files are edited or staged, exit with error
  if ! git diff --quiet --exit-code -- $path || ! git diff --cached --quiet --exit-code -- $path; then
    echo "Error: $message"
    git diff HEAD
    exit 1
  fi
}

check_git_clean "Please commit or stash your changes before running this script" "*"

# check out all files in crates/rtd-protocol-config/src/snapshots at origin commit
echo "Checking out $NETWORK snapshot files"
git checkout $RELEASED_COMMIT -- crates/rtd-protocol-config/src/snapshots || exit 1

if [ "$NETWORK" != "testnet" ] && [ "$NETWORK" != "mainnet" ]; then
  NETWORK_PATTERN="*__version_*"
else
  NETWORK_PATTERN="*__"$(echo "$NETWORK" | awk '{print toupper(substr($0, 1, 1)) substr($0, 2)}')"_version_*"
fi

echo "Checking for changes to snapshot files matching $NETWORK_PATTERN"
check_git_clean "Detected changes to snapshot files since $RELEASED_COMMIT - not safe to release" "$NETWORK_PATTERN"

# remove any snapshot file changes that were ignored
git restore --source=HEAD --staged --worktree -- crates/rtd-protocol-config/src/snapshots

echo "Running snapshot tests..."
cargo test --package rtd-protocol-config snapshot_tests || exit 1

exit 0
