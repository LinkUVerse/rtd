# Shared Bigtable protocol types

This crate owns the generated Rust types previously private to `rtd-kvstore`.
The four generated files were moved without byte changes. `rtd-kvstore` re-exports
these same types at its existing internal module path; this is not a second wire
format or a second generated implementation.

The authoritative `.proto` inputs remain in `../rtd-kvstore/proto` for existing
development tools. Regenerate with `../rtd-kvstore/build-protos`; its output now
targets this crate. Ordinary builds use the checked-in generated files and do not
invoke protoc, Python, Cargo scripts, or network downloads at runtime.

Archive control uses this protocol-only dependency to issue bounded, read-only
Bigtable `ReadRows` requests without depending on the full indexer runtime.
