# ash-rs

This project provides Rust crates to interact with:

- Avalanche nodes, Subnets, and blockchains;
- other Ash tools.

## Crates

- [ash_sdk](crates/ash_sdk): Ash Rust SDK  
  [<img alt="crates.io" src="https://img.shields.io/crates/v/ash_sdk.svg?style=flat&color=fc8d62&logo=rust">](https://crates.io/crates/ash_sdk)
  [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-ash_sdk-green?style=flat&labelColor=555555&logo=docs.rs">](https://docs.rs/ash_sdk)
- [ash_cli](crates/ash_cli): Ash CLI  
  [<img alt="crates.io" src="https://img.shields.io/crates/v/ash_cli.svg?style=flat&color=fc8d62&logo=rust">](https://crates.io/crates/ash_cli)

## Ash CLI Installation

See the [Installation](https://ash.center/docs/toolkit/ash-cli/installation) section of the documentation.

## Configuration

See the [Custom Configuration tutorial](https://ash.center/docs/toolkit/ash-cli/tutorials/custom-configuration) section of the documentation.

## Development

```sh
git clone https://github.com/AshAvalanche/ash-rs.git
cd ash-rs

# Run the library tests
cargo test

# Run the CLI
## Debug mode
cargo run -- --help
## Release mode
cargo run --release -- --help
```

### Releasing

Use the `build*.sh` scripts to build the CLI binary. The binary will be archived to `ash-${PLATFORM}-${ARCH}-v${VERSION}.tar.gz` where `PLATFORM` is `linux` or `macos`, `ARCH` is `amd64` or `arm64`, and `VERSION` is the version of the crate. A SHA512 checksum file will also be generated.

For MacOS builds, [osxcross](https://github.com/tpoechtrager/osxcross) is used to cross-compile the binary. See [scripts/osxcross_setup.sh](./scripts/osxcross_setup.sh) for the setup script.

```sh
# Build a release for Linux only
./scripts/build_linux.sh --release
# Build a release for Mac only
OSXCROSS_PATH=/full/path/to/osxcross ./scripts/build_macos.sh --release
# Build a release for both Mac and Linux
# Requires osxcross to be installed. See ./scripts/osxcross_setup.sh
OSXCROSS_PATH=/full/path/to/osxcross ./sripts/build.sh --release
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
envsubst < crates/ash_sdk/tests/conf/quicknode.yml > target/ash-test-avax-conf.yml
# Run the tests
ASH_TEST_AVAX_CONFIG="$PWD/target/ash-test-avax-conf.yml" cargo test
```

## Roadmap

- [x] CLI
- [x] Get Subnets and blockchains information from the Avalanche P-Chain
- [x] Get nodes information from the Avalanche P-Chain
- [x] Get Subnet validators information from the Avalanche P-Chain
- [x] Subnet creation
- [x] Blockchain creation
- [ ] WASM integration (to allow the library to be used from JavaScript)
