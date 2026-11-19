#!/usr/bin/env python3
"""Apply the RTD text mapping without changing ordinary words such as suitable.

The fork guide's shell script invokes this on the upstream checkout before
renaming paths. Historical RTD diagnosis documents and fork instructions are
preserved as source evidence; only tracked upstream text files are rewritten.
"""

import re
import subprocess
import sys
from pathlib import Path


BINARY_SUFFIXES = {
    ".a", ".avif", ".bcs", ".bin", ".blob", ".class", ".db",
    ".dylib", ".gif", ".gz", ".heic", ".ico", ".icns", ".jar",
    ".jpeg", ".jpg", ".lockb", ".mov", ".mp3", ".mp4", ".mv",
    ".otf", ".parquet", ".pdf", ".pb", ".png", ".rlib", ".so",
    ".ttf", ".wasm", ".wav", ".webp", ".woff", ".woff2", ".zip",
}


def rename_text(value: str) -> str:
    for old, new in (
        ("MystenLabs", "LinkUVerse"),
        ("mystenlabs", "linkuverse"),
        ("Mysten Labs", "LinkU Labs"),
        ("Mysten", "LinkU"),
        ("mysten", "linku"),
        ("SUINS", "RTDNS"),
        ("SuiNS", "RtdNS"),
        ("suins", "rtdns"),
    ):
        value = value.replace(old, new)

    # Brand stems also occur in snake_case, kebab-case, and CamelCase names.
    # A lower-case suffix indicates an ordinary word unless it was handled
    # explicitly above (for example, suins).
    value = re.sub(r"(?<![A-Z])SUI(?=$|[^A-Za-z]|[A-Z][a-z])", "RTD", value)
    value = re.sub(r"Sui(?=$|[^a-z]|[A-Z])", "Rtd", value)
    value = re.sub(r"(?<![a-z])sui(?=$|[^a-z]|[A-Z])", "rtd", value)
    return value


def main(root: Path) -> int:
    names = subprocess.check_output(["git", "ls-files", "-z"], cwd=root).split(b"\0")
    changed = skipped_binary = 0
    for raw in names:
        if not raw:
            continue
        relative = Path(raw.decode("utf-8", "surrogateescape"))
        if relative.parts[0] in {"doc", "fork-instruct"}:
            continue
        path = root / relative
        if not path.is_file() or path.is_symlink() or path.suffix.lower() in BINARY_SUFFIXES:
            continue
        data = path.read_bytes()
        if b"\0" in data:
            skipped_binary += 1
            continue
        try:
            original = data.decode("utf-8")
        except UnicodeDecodeError:
            skipped_binary += 1
            continue
        updated = rename_text(original)
        if updated != original:
            path.write_bytes(updated.encode("utf-8"))
            changed += 1
    print(f"RTD text replacements: {changed} files changed; {skipped_binary} binary files skipped")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(Path(sys.argv[1]).resolve()))
