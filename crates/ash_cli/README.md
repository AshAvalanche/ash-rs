# `ash_cli` Crate

This crate provides the `ash` command line interface. It aims at boosting Avalanche developers productivity by providing a set of commands to interact with Avalanche. Some of the commands are:

```bash
# List available Avalanche networks
ash avalanche network list

# List the Subnets of the mainnet network
ash avalanche subnet list --network mainnet

# Show detailed information about one of the mainnet Subnets
# The output can be set to JSON and piped to jq for maximum flexibility
ash avalanche subnet info Vn3aX6hNRstj5VHHm63TCgPNaeGnRSqCYXQqemSqDd2TQH4qJ --json | jq '.blockchains'

# Show detailed information about a validator of the mainnet Subnet
ash avalanche validator info --network fuji NodeID-FhFWdWodxktJYq884nrJjWD8faLTk9jmp
```

## Available commands

- `ash conf`

  ```bash
  Interact with Ash configuration files

  Usage: ash conf [OPTIONS] <COMMAND>

  Commands:
    init  Initialize an Ash config file
  ```

- `ash avalanche`

  ```bash
  Interact with Avalanche Subnets, blockchains and nodes

  Usage: ash avalanche [OPTIONS] <COMMAND>

  Commands:
    blockchain  Interact with Avalanche blockchains
    network     Interact with Avalanche networks
    node        Interact with Avalanche nodes
    subnet      Interact with Avalanche Subnets
    validator   Interact with Avalanche validators
    vm          Interact with Avalanche VMs
    wallet      Interact with Avalanche wallets
    x           Interact with Avalanche X-Chain
  ```

## Tutorials

See the [Tutorials](https://ash.center/docs/category/tutorials-1) section of the documentation.
