# ash-rs

The `ash` create provides a Rust SDK to interact with the Ash protocol.

## Usage

```sh
git clone https://github.com/AshAvalanche/ash-rs.git
cd ash-rs

# Run the library tests
cargo test

# Build a release
cargo build --release
```

## Modules

- [ash::node](src/node.rs): Interact with Ash nodes through the `AshNode` struct.

## Roadmap

- [ ] CLI
- [ ] Ash protocol integration (abstract smart contracts interaction from the user)
- [ ] Ledger integration (to sign transactions)
- [ ] WASM integration (to allow the library to be used from JavaScript)
