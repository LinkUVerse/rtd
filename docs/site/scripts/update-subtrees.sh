#!/usr/bin/env bash

# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

cd "$(git rev-parse --show-toplevel)" || exit 1

git subtree pull --prefix=docs/site/src/shared git@github.com:LinkUVerse/ML-Shared-Docusaurus.git master --squash
git subtree pull --prefix=docs/subtree/awesome-rtd git@github.com:rtd-foundation/awesome-rtd.git main --squash
git subtree pull --prefix=docs/subtree/awesome-gaming git@github.com:becky-rtd/awesome-rtd-gaming.git main --squash

echo "✅ All subtree content updated — commit and push the changes"
