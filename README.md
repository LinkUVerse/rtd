<p align="center">
  <img src="docs/site/static/img/logo.svg" alt="Rtd logo" width="100" height="100">
</p>

# Rtd

Rtd is a Move based blockchain forked from the Sui source revision
`ea36d0cbc33376eb33b8083c1e27cb3e7bb9f368`. Its native coin is
`Coin<rtd::rtd::RTD>`. The system modules, RPC methods, private key prefix,
and protobuf services use the RTD namespace. A new genesis produces a new
chain ID; Rtd nodes and clients cannot join a Sui network.

The fork source is under active validation. Formal RTD mainnet and testnet
genesis digests and public service endpoints have not been configured. Sample
URLs in inherited documentation are not evidence of deployed RTD services.

## Build

```bash
cargo check --workspace
cargo build -p rtd -p rtd-node
```

The fork procedure, current upstream refresh decisions, and verification
limits are recorded in
[`fork-instruct/final/UPSTREAM_REFRESH_2026-09-23.md`](fork-instruct/final/UPSTREAM_REFRESH_2026-09-23.md).
Documentation inherited from the upstream repository needs a separate review
before publication.

## Contributing

See the [Contributing Guide](CONTRIBUTING.md) and
[Code of Conduct](CODE_OF_CONDUCT.MD).

## License

See [LICENSE](LICENSE) and the notices retained in vendored dependencies.
