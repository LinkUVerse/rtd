#!/usr/bin/env python3
"""Rename remaining upstream-branded file and directory names bottom-up."""

import os
import runpy
import sys
from pathlib import Path


root = Path(sys.argv[1]).resolve()
rename_text = runpy.run_path(str(Path(__file__).with_name("brand-replace.py")))["rename_text"]
exclude = {".git", "target", "node_modules", "fork-instruct", "doc"}
paths = []
for parent, dirs, files in os.walk(root):
    relative = Path(parent).relative_to(root)
    if not relative.parts:
        dirs[:] = [name for name in dirs if name not in exclude]
    else:
        dirs[:] = [name for name in dirs if name not in {".git", "target", "node_modules"}]
    paths.extend(Path(parent) / name for name in dirs + files)

count = 0
for path in sorted(paths, key=lambda item: len(item.parts), reverse=True):
    new_name = rename_text(path.name)
    if new_name == path.name:
        continue
    target = path.with_name(new_name)
    if target.exists():
        raise SystemExit(f"Renamed path already exists: {path} -> {target}")
    path.rename(target)
    count += 1

print(f"RTD path replacements: {count} files and directories renamed")
