# ash-rs

This project provides Rust crates to interact with the Ash protocol.

## Crates

- [ash](crates/ash): Rust SDK to interact with the Ash protocol
- [ashcli](crates/cli): CLI to interact with the Ash protocol

## Ash CLI Installation

```sh
git clone https://github.com/AshAvalanche/ash-rs.git
cd ash-rs

cargo install --path crates/cli

# The CLI is then available globally
ash --help
```

See [Available commands](crates/cli/README.md#available-commands).

## Configuration

A YAML configuration file can be generated using the `ash conf init` command and then reused in commands with the `--config` flag.

This allows to query custom networks with the CLI:

```yaml
avalancheNetworks:
  - name: local
    subnets:
      - id: 11111111111111111111111111111111LpoYY
        blockchains:
          - id: yH8D7ThNJkxmtkuv2jgBa4P1Rn3Qpr4pPr7QYNfcdoS6k6HWp
            name: C-Chain
            vmType: EVM
            rpcUrl: https://localhost:9650/ext/bc/C/rpc
```

## Development

```sh
git clone https://github.com/AshAvalanche/ash-rs.git
cd ash-rs

# Run the library tests
cargo test

# Build a release
cargo build --release

# Run the CLI
## Debug mode
cargo run -- --help
## Release mode
cargo run --release -- --help
```

## Roadmap

- [x] CLI
- [ ] Get Subnets and blockchains information from the Avalanche P-Chain
- [ ] Ash protocol integration (abstract smart contracts interaction from the user)
- [ ] Ledger integration (to sign transactions)
- [ ] WASM integration (to allow the library to be used from JavaScript)
