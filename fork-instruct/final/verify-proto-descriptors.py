#!/usr/bin/env python3
"""Check the on-wire packages in generated protobuf FileDescriptorSets.

This only reads the wire fields required here, so it does not need protoc or a
Python protobuf installation. A successful Rust build alone cannot verify
checked-in binary descriptors.
"""

import sys
from pathlib import Path


DESCRIPTORS = {
    "crates/rtd-fork/src/proto/generated/forking_descriptor.bin": "rtd.forking.v1alpha",
    "crates/rtd-indexer-alt-consistent-api/src/proto/generated/rtd.rpc.consistent.v1alpha.fds.bin":
        "rtd.rpc.consistent.v1alpha",
}


def varint(data: bytes, offset: int) -> tuple[int, int]:
    result = 0
    shift = 0
    while offset < len(data) and shift < 70:
        byte = data[offset]
        offset += 1
        result |= (byte & 0x7F) << shift
        if byte < 0x80:
            return result, offset
        shift += 7
    raise ValueError("truncated or oversized protobuf varint")


def fields(data: bytes):
    offset = 0
    while offset < len(data):
        tag, offset = varint(data, offset)
        field, wire = tag >> 3, tag & 7
        if wire == 0:
            value, offset = varint(data, offset)
        elif wire == 1:
            value = data[offset : offset + 8]
            offset += 8
        elif wire == 2:
            length, offset = varint(data, offset)
            value = data[offset : offset + length]
            offset += length
        elif wire == 5:
            value = data[offset : offset + 4]
            offset += 4
        else:
            raise ValueError(f"unsupported protobuf wire type {wire}")
        if offset > len(data):
            raise ValueError("truncated protobuf field")
        yield field, wire, value


def verify(path: Path, expected_package: str) -> None:
    packages = set()
    custom_files = 0
    for field, wire, descriptor in fields(path.read_bytes()):
        if field != 1 or wire != 2:  # FileDescriptorSet.file
            continue
        values = {
            number: value.decode("utf-8")
            for number, kind, value in fields(descriptor)
            if number in {1, 2} and kind == 2
        }
        name, package = values.get(1, ""), values.get(2, "")
        if name.startswith("google/protobuf/") and package == "google.protobuf":
            continue
        custom_files += 1
        if not name.startswith("rtd/") or not package.startswith("rtd."):
            raise ValueError(f"old-chain or unexpected descriptor: {name} ({package})")
        packages.add(package)
    if custom_files == 0 or expected_package not in packages:
        raise ValueError(f"missing expected package {expected_package}: {packages}")
    print(f"{path}: {custom_files} RTD proto file(s), {', '.join(sorted(packages))}")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: verify-proto-descriptors.py ROOT")
    root = Path(sys.argv[1]).resolve()
    for relative, expected in DESCRIPTORS.items():
        verify(root / relative, expected)
