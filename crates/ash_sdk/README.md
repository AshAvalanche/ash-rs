# ash_sdk

`ash-rs` is a Rust SDK for [Avalanche](https://avax.network) and [Ash](https://ash.center) tools.

It provides a higher level of abstraction than [avalanche-types-rs](https://github.com/ava-labs/avalanche-types-rs) and comes with a CLI that simplifies the interaction with Avalanche networks.

## Opinionated structs layout

The library provides an opinionated layout to represent Avalanche networks, Subnets and blockchains. The layout could be represented as follows:

```
AvalancheNetwork
└── subnets
    ├── AvalancheSubnet 1
    │   ├── blockchains
    │   │   ├── AvalancheBlockchain 1
    │   │   └── AvalancheBlockchain 2
    │   └── validators
    │       ├── AvalancheSubnetValidator 1
    │       └── AvalancheSubnetValidator 2
    └── AvalancheSubnet 2
        ├── blockchains
        │   ├── AvalancheBlockchain 3
        │   └── AvalancheBlockchain 4
        └── validators
            ├── AvalancheSubnetValidator 1
            └── AvalancheSubnetValidator 2
```

### Avalanche networks

An `AvalancheNetwork` is a top level struct that represents an Avalanche network. It contains the list of its `AvalancheSubnet`s. Most of the updating methods are implemented on this struct (e.g. `update_subnet`, `update_blockchains`, etc.).

### Avalanche Subnets and validators

An `AvalancheSubnet` is a struct that represents an Avalanche Subnet. It contains all the Subnet metadata, the list of its `AvalancheBlockchain`s and the list of its validators (as `AvalancheSubnetsValidator`s).

### Avalanche blockchains

An `AvalancheBlockchain` is a struct that represents an Avalanche blockchain. It contains all the blockchain metadata.

### Avalanche nodes

An `AvalancheNode` is a struct that represents an Avalanche node. An `AvalancheNode` is not directly linked to an `AvalancheNetwork` as its metadata are retrieved directly from its endpoint.

## Configuration

The library relies on YAML configuration files that contains the list of known Avalanche networks. For each network, at least the P-Chain configuration has to be provided (in the Primary Network) with its ID and RPC endpoint. All the other Subnets/blockchains will be retrieved/enriched from the P-Chain.

A default configuration is embedded in the library (see [conf/default.yaml](https://github.com/AshAvalanche/ash-rs/blob/main/crates/ash_sdk/conf/default.yml)) and contains the following networks:

- `mainnet` and `fuji` use the default Avalanche public endpoints
- `mainnet-ankr` and `fuji-ankr` use the Ankr Avalanche public endpoints
- `mainnet-blast` and `fuji-blast` use the Blast Avalanche public endpoints

Configuration example:

```yaml
# Default configuration of the mainnet network
avalancheNetworks:
  - name: mainnet
    subnets:
      - id: 11111111111111111111111111111111LpoYY
        controlKeys: []
        threshold: 0
        blockchains:
          - id: 11111111111111111111111111111111LpoYY
            name: P-Chain
            vmType: PVM
            rpcUrl: https://api.avax.network/ext/bc/P
          - id: 2q9e4r6Mu3U68nU1fYjgbR6JvwrRx36CohpAX5UQxse55x1Q5
            name: C-Chain
            vmId: mgj786NP7uDwBCcq6YwThhaN8FLyybkCa4zBWTQbNgmK6k9A6
            vmType: EVM
            rpcUrl: https://api.avax.network/ext/bc/C/rpc
          - id: 2oYMBNV4eNHyqk2fjjV5nVQLDbtmNJzq5s3qs3Lo6ftnC6FByM
            name: X-Chain
            vmId: jvYyfQTxGMJLuGWa55kdP2p2zSUYsQ5Raupu4TW34ZAUBAbtq
            vmType: AVM
            rpcUrl: https://api.avax.network/ext/bc/X
```

**Note:** You can generate a configuration file with the CLI using the `ash conf init` command.

## Usage

One can check out the [CLI code](https://github.com/AshAvalanche/ash-rs/tree/main/crates/ash_cli) to see examples of how to use the library.

## Modules

- `conf`: Interact with the library configuration in YAML
- `errors`: Generate errors for the library
- `avalanche`: Interact with Avalanche networks, Subnets and blockchains
  - `avalanche::subnets`: Interact with Avalanche Subnets and validators
  - `avalanche::blockchains`: Interact with Avalanche blockchains
  - `avalanche::jsonrpc`: Interact with Avalanche VMs JSON RPC endpoints
