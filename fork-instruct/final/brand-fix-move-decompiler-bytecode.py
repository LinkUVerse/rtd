#!/usr/bin/env python3
# Copyright (c) LinkU Labs, Inc.
# SPDX-License-Identifier: Apache-2.0

"""Rebrand Move decompiler bytecode fixtures and their scoped snapshots.

The ordinary text pass updates ``*.mv.snap`` but deliberately skips binary
``*.mv`` files. These three substitutions preserve every bytecode length
prefix. Other brand substitutions change length and require recompilation.
``suilend`` is an unrelated protocol name in the upstream test corpus.
"""

import os
import subprocess
import sys
from pathlib import Path


REPLACEMENTS = ((b"SUI", b"RTD"), (b"Sui", b"Rtd"), (b"sui", b"rtd"))
PROTECTED = b"suilend"


def rewrite_bytecode(data: bytes) -> bytes:
    if any(token in data for token in (b"MystenLabs", b"Mysten", b"mysten")):
        raise ValueError("A fixture contains a length-changing brand; recompile it from source")

    result = bytearray()
    index = 0
    while index < len(data):
        if data[index : index + len(PROTECTED)].lower() == PROTECTED:
            result.extend(data[index : index + len(PROTECTED)])
            index += len(PROTECTED)
            continue
        for old, new in REPLACEMENTS:
            if data.startswith(old, index):
                result.extend(new)
                index += len(old)
                break
        else:
            result.append(data[index])
            index += 1
    if len(result) != len(data):
        raise AssertionError("Move bytecode size changed")
    return bytes(result)


def fix_fixtures(root: Path) -> None:
    fixtures = root / "external-crates/move/crates/move-decompiler/tests/bytecode"
    workspace = root / "external-crates/move"
    originals = {path: path.read_bytes() for path in fixtures.glob("*.mv")}
    rewritten = {path: rewrite_bytecode(data) for path, data in originals.items()}
    changed = {path for path, data in rewritten.items() if data != originals[path]}
    snapshots = {path: path.read_bytes() for path in fixtures.glob("*.mv.snap")}

    for path in changed:
        path.write_bytes(rewritten[path])

    try:
        environment = os.environ.copy()
        environment["INSTA_UPDATE"] = "always"
        command = ["cargo", "test", "-p", "move-decompiler", "--test", "tests", "bytecode"]
        subprocess.run(command, cwd=workspace, env=environment, check=True)

        changed_snapshots = {
            path for path in fixtures.glob("*.mv.snap") if path.read_bytes() != snapshots.get(path)
        }
        unexpected = changed_snapshots - {fixtures / (path.name + ".snap") for path in changed}
        if unexpected:
            raise RuntimeError(f"Unrelated bytecode snapshots changed: {sorted(unexpected)}")
        if list(fixtures.glob("*.snap.new")):
            raise RuntimeError("Unreviewed .snap.new files remain")

        environment.pop("INSTA_UPDATE", None)
        subprocess.run(command, cwd=workspace, env=environment, check=True)
    except Exception:
        for path in changed:
            path.write_bytes(originals[path])
        for path in fixtures.glob("*.mv.snap"):
            if path not in snapshots:
                path.unlink()
        for path, data in snapshots.items():
            path.write_bytes(data)
        raise

    print(f"Rebranded {len(changed)} Move bytecode fixtures; refreshed {len(changed_snapshots)} snapshots")


if __name__ == "__main__":
    if len(sys.argv) == 2 and sys.argv[1] == "--self-test":
        assert rewrite_bytecode(b"SUI Sui sui sui-20 suilend") == b"RTD Rtd rtd rtd-20 suilend"
        assert len(rewrite_bytecode(b"\x03sui\x03SUI")) == len(b"\x03sui\x03SUI")
        print("self-test passed")
    elif len(sys.argv) == 2:
        fix_fixtures(Path(sys.argv[1]).resolve())
    else:
        raise SystemExit("usage: brand-fix-move-decompiler-bytecode.py ROOT | --self-test")
