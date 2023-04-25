# ash-rs

This project provides Rust crates to interact with:

- Avalanche nodes, Subnets, and blockchains;
- other Ash tools.

## Crates

- [ash_sdk](crates/sdk): Ash Rust SDK
- [ash_cli](crates/cli): Ash CLI

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

A YAML configuration file can be generated using the `ash conf init` command, enriched and then reused in commands with the `--config` flag.

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

### Advanced testing

#### Local Avalanche network

Some tests (e.g. [avalanche::nodes::tests](./crates/ash/src/avalanche/nodes.rs)) require a local Avalanche network and are `ignored` by default. They are configured to work with [avalanche-network-runner](https://github.com/ava-labs/avalanche-network-runner). The easiest way to bootstrap a network is using [avalanche-cli](https://github.com/ava-labs/avalanche-cli):

```sh
# Install avalanche-cli
curl -sSfL https://raw.githubusercontent.com/ava-labs/avalanche-cli/main/scripts/install.sh | sh -s
export PATH=~/bin:$PATH

# Start the local network
avalanche network start

# Run all tests
cargo test -- --include-ignored
```

#### RPC endpoint configuration for tests

The Avalanche public APIs (provided by Ava Labs, Ankr, Blast, etc.) have rate limits that can impact testing. It is necessary to use a private RPC endpoint to have a reproducible testing behaviour.

A custom configuration file can be provided through the `ASH_TEST_AVAX_CONFIG` environment variable (defaults to [crates/ash/tests/conf/default.yml](./crates/ash/tests/conf/default.yml)). Tests are performed on the `fuji` network in this configuration file. See [Configuration](#configuration) to see how to generate a sample file.

###### Ash QuickNode endpoint

The PR GitHub Actions workflow run tests on the Ash team's [QuickNode](https://www.quicknode.com/) RCP endpoint.

To run tests locally using this endpoint, you need a local copy of the [ash-infra repo](https://github.com/AshAvalanche/ash-infra) (private). Generate the tests configuration file before running the tests:

```sh
# Set ASH_INFRA_PATH
ASH_INFRA_PATH=path/to/ash-infra
# Source the tests .env file
source crates/ash/tests/.env
# Generate the tests configuration file
envsubst < crates/ash/tests/conf/quicknode.yml > target/ash-test-avax-conf.yml
# Run the tests
ASH_TEST_AVAX_CONFIG="$PWD/target/ash-test-avax-conf.yml" cargo test
```

## Roadmap

- [x] CLI
- [x] Get Subnets and blockchains information from the Avalanche P-Chain
- [x] Get nodes information from the Avalanche P-Chain
- [x] Get Subnet validators information from the Avalanche P-Chain
- [ ] Subnet creation
- [ ] Blockchain creation
- [ ] WASM integration (to allow the library to be used from JavaScript)
