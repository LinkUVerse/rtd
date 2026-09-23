#!/usr/bin/env python3
"""Rename protocol identifiers that are embedded inside longer words.

Bech32 private keys need a new checksum when their human-readable prefix
changes. Replacing only the prefix makes every example key invalid.
"""

import os
import re
import sys
from pathlib import Path


CHARSET = "qpzry9x8gf2tvdw0s3jn54khce6mua7l"
GENERATORS = (0x3B6A57B2, 0x26508E6D, 0x1EA119FA, 0x3D4233DD, 0x2A1462B3)
KEY_PATTERN = re.compile(r"suiprivkey1[023456789acdefghjklmnpqrstuvwxyz]+")
SKIP_DIRS = {".git", "target", "node_modules", "doc", "fork-instruct", ".next", "dist"}
SKIP_SUFFIXES = {
    ".a", ".avif", ".bcs", ".bin", ".blob", ".class", ".db", ".dylib",
    ".gif", ".gz", ".heic", ".ico", ".icns", ".jar", ".jpeg", ".jpg",
    ".lockb", ".mov", ".mp3", ".mp4", ".mv", ".otf", ".parquet",
    ".pdf", ".pb", ".png", ".rlib", ".so", ".ttf", ".wasm", ".wav",
    ".webp", ".woff", ".woff2", ".zip",
}


def polymod(values: list[int]) -> int:
    state = 1
    for value in values:
        head = state >> 25
        state = (state & 0x1FFFFFF) << 5 ^ value
        for bit, generator in enumerate(GENERATORS):
            if (head >> bit) & 1:
                state ^= generator
    return state


def hrp_expand(hrp: str) -> list[int]:
    return [ord(char) >> 5 for char in hrp] + [0] + [ord(char) & 31 for char in hrp]


def change_key_prefix(match: re.Match[str]) -> str:
    encoded = match.group()
    old_hrp = "suiprivkey"
    new_hrp = "rtdprivkey"
    digits = [CHARSET.index(char) for char in encoded[len(old_hrp) + 1 :]]
    if len(digits) < 7 or polymod(hrp_expand(old_hrp) + digits) != 1:
        # Some test inputs intentionally contain malformed or truncated keys.
        return encoded.replace(old_hrp, new_hrp, 1)
    payload = digits[:-6]
    checksum_state = polymod(hrp_expand(new_hrp) + payload + [0] * 6) ^ 1
    checksum = [(checksum_state >> 5 * (5 - index)) & 31 for index in range(6)]
    return f"{new_hrp}1{''.join(CHARSET[digit] for digit in payload + checksum)}"


def rename_text(value: str) -> str:
    value = KEY_PATTERN.sub(change_key_prefix, value)
    for old, new in (
        ("SUIX", "RTDX"),
        ("Suix", "Rtdx"),
        ("suix", "rtdx"),
        ("SuinsRegistration", "RtdnsRegistration"),
    ):
        value = value.replace(old, new)
    # Non-key references to the Bech32 human-readable prefix have no checksum.
    return value.replace("suiprivkey", "rtdprivkey")


def rewrite_tree(root: Path) -> None:
    changed = 0
    for directory, dirs, files in os.walk(root):
        dirs[:] = [name for name in dirs if name not in SKIP_DIRS]
        for name in files:
            path = Path(directory) / name
            if path.is_symlink() or path.suffix.lower() in SKIP_SUFFIXES:
                continue
            original = path.read_bytes()
            if b"\0" in original:
                continue
            try:
                text = original.decode("utf-8")
            except UnicodeDecodeError:
                continue
            updated = rename_text(text)
            if updated != text:
                path.write_text(updated)
                changed += 1
    print(f"RTD protocol literals: {changed} files changed")


if __name__ == "__main__":
    if len(sys.argv) == 2 and sys.argv[1] == "--self-test":
        source = "suiprivkey1qzdlfxn2qa2lj5uprl8pyhexs02sg2wrhdy7qaq50cqgnffw4c2477kg9h3"
        expected = "rtdprivkey1qzdlfxn2qa2lj5uprl8pyhexs02sg2wrhdy7qaq50cqgnffw4c247cdptmr"
        assert change_key_prefix(re.match(KEY_PATTERN, source)) == expected
        assert rename_text('"suix_getBalance"') == '"rtdx_getBalance"'
        print("self-test passed")
    elif len(sys.argv) == 2:
        rewrite_tree(Path(sys.argv[1]).resolve())
    else:
        raise SystemExit("usage: brand-fix-protocol-literals.py ROOT | --self-test")
