#!/usr/bin/env python3
"""Vendor the HTTP server crate used by the September 2026 upstream fork.

The new chain uses APIs added after the older in-tree RTD HTTP crate, so the
published 0.3.1 source is pinned here and kept as a local, renamed package.
"""

import hashlib
import io
import sys
import tarfile
import urllib.request
from pathlib import Path


VERSION = "0.3.1"
ARCHIVE_SHA256 = "b6b46940575ea7ee7eaf2a7000da28d82a61b76f8d4160590674946263975782"
ARCHIVE_URL = f"https://static.crates.io/crates/sui-http/sui-http-{VERSION}.crate"
PREFIX = f"sui-http-{VERSION}/"


def vendor(root: Path) -> None:
    destination = root / "crates/rtd-http"
    if destination.exists():
        workspace = (root / "Cargo.toml").read_text()
        manifest = (destination / "Cargo.toml").read_text()
        if (
            'name = "rtd-http"' in manifest
            and '    "crates/rtd-http",' in workspace
            and 'rtd-http = { path = "crates/rtd-http" }' in workspace
        ):
            print(f"rtd-http {VERSION} is already vendored")
            return
        raise SystemExit(f"{destination} already exists but is not configured as the local package")

    with urllib.request.urlopen(ARCHIVE_URL, timeout=120) as response:
        archive_bytes = response.read()
    digest = hashlib.sha256(archive_bytes).hexdigest()
    if digest != ARCHIVE_SHA256:
        raise SystemExit(f"source checksum mismatch: {digest}")

    manifest = None
    with tarfile.open(fileobj=io.BytesIO(archive_bytes), mode="r:gz") as archive:
        for entry in archive:
            if not entry.isfile() or not entry.name.startswith(PREFIX):
                continue
            relative = entry.name[len(PREFIX) :]
            if relative == "Cargo.toml.orig":
                manifest = archive.extractfile(entry).read().decode("utf-8")
                continue
            if relative != "LICENSE" and not relative.startswith(("src/", "tests/")):
                continue
            output = destination / relative
            output.parent.mkdir(parents=True, exist_ok=True)
            data = archive.extractfile(entry).read()
            if output.suffix == ".rs":
                data = data.decode("utf-8").replace("sui_http", "rtd_http").replace(
                    "sui-http", "rtd-http"
                ).encode("utf-8")
            output.write_bytes(data)

    if manifest is None:
        raise SystemExit("published package did not contain Cargo.toml.orig")
    manifest = manifest.replace('name = "sui-http"', 'name = "rtd-http"', 1)
    manifest = manifest.replace('repository = "https://github.com/mystenlabs/sui-http/"\n', "", 1)
    manifest = manifest.replace(
        'description = "HTTP server and utils used by many sui services"',
        'description = "HTTP server and utilities for RTD services"',
        1,
    )
    (destination / "Cargo.toml").write_text(manifest)
    (destination / "README.md").write_text(
        "# rtd-http\n\n"
        "HTTP server and middleware used by RTD services.\n\n"
        "This local fork is based on the Apache-2.0 licensed `sui-http` 0.3.1 crate\n"
        "by Mysten Labs. Its upstream package and copyright notices remain attributed\n"
        "to their original owners. The source was copied from the crates.io release\n"
        "and renamed to provide the RTD crate API without depending on a Sui-named\n"
        "package. See `LICENSE` for the license terms.\n"
    )

    workspace = root / "Cargo.toml"
    original = workspace.read_text()
    if '    "crates/rtd-futures",\n' not in original or 'rtd-http = "0.3.1"' not in original:
        raise SystemExit("expected rtd-http workspace manifest entries were not found")
    updated = original.replace(
        '    "crates/rtd-futures",\n',
        '    "crates/rtd-futures",\n    "crates/rtd-http",\n',
        1,
    ).replace('rtd-http = "0.3.1"', 'rtd-http = { path = "crates/rtd-http" }', 1)
    workspace.write_text(updated)
    print(f"Vendored rtd-http {VERSION} from verified upstream package")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: vendor-rtd-http.py ROOT")
    vendor(Path(sys.argv[1]).resolve())
