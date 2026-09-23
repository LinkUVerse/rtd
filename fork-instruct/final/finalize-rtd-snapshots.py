#!/usr/bin/env python3
# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

"""Keep only snapshots generated for this chain's first protocol version."""

import json
import os
import shutil
import sys
from pathlib import Path

FIRST_RTD_VERSION = 138


def main() -> None:
    root = Path(__file__).resolve().parents[2]
    snapshot_root = root / "crates/rtd-framework-snapshot/bytecode_snapshot"
    manifest_path = root / "crates/rtd-framework-snapshot/manifest.json"
    manifest = json.loads(manifest_path.read_text())
    current = manifest.get(str(FIRST_RTD_VERSION))
    if current is None or len(current.get("packages", [])) != 5:
        sys.exit("Generate and verify the five RTD genesis packages first")

    current_dir = snapshot_root / str(FIRST_RTD_VERSION)
    expected = {package["id"] for package in current["packages"]}
    actual = {path.name for path in current_dir.iterdir() if path.is_file()}
    if actual != expected or len(list(current_dir.iterdir())) != len(expected):
        sys.exit("RTD genesis snapshot files differ from the manifest")
    for path in current_dir.iterdir():
        binary = path.read_bytes().lower()
        if b"mysten" in binary or b"sui" in binary:
            sys.exit(f"Old chain branding found in {path}")

    old_dirs = [
        path
        for path in snapshot_root.iterdir()
        if path.is_dir() and path.name.isdecimal() and int(path.name) < FIRST_RTD_VERSION
    ]
    old_entries = [key for key in manifest if int(key) < FIRST_RTD_VERSION]
    for path in old_dirs:
        shutil.rmtree(path)
    for key in old_entries:
        del manifest[key]

    temporary_path = manifest_path.with_suffix(".json.tmp")
    temporary_path.write_text(json.dumps(manifest, indent=2) + "\n")
    os.replace(temporary_path, manifest_path)
    print(f"Removed {len(old_dirs)} upstream snapshot directories and {len(old_entries)} manifest entries")


if __name__ == "__main__":
    main()
