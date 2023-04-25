# `ash_cli` Crate

This crate provides the `ash` command line interface. It aims at boosting Avalanche developers productivity by providing a set of commands to interact with Avalanche. Some of the commands are:

```bash
# List available Avalanche networks
ash avalanche network list

# List the Subnets of the mainnet network
ash avalanche subnet list --network mainnet

# Show detailed information about one of the mainnet Subnets
# The ouput can be set to JSON and piped to jq for maximum flexibility
ash avalanche subnet info --id Vn3aX6hNRstj5VHHm63TCgPNaeGnRSqCYXQqemSqDd2TQH4qJ --json | jq '.blockchains'

# Show detailed information about a validator of the mainnet Subnet
ash avalanche validator info --network fuji --id NodeID-FhFWdWodxktJYq884nrJjWD8faLTk9jmp
```

## Available commands

- `ash conf`: Manipulate the Ash lib configuration
- `ash avalanche network`: Interact with Avalanche networks
- `ash avalanche node`: Interact with Avalanche nodes
- `ash avalanche subnet`: Interact with Avalanche Subnets
- `ash avalanche validator`: Interact with Avalanche validators

## Tutorials

See the [Tutorials](https://ash.center/docs/category/tutorials-1) section of the documentation.
