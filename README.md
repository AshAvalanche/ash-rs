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
- [ ] Ash protocol integration (abstract smart contracts interaction from the user)
- [ ] Ledger integration (to sign transactions)
- [ ] WASM integration (to allow the library to be used from JavaScript)
