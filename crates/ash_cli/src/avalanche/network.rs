// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the network subcommand parser

use crate::utils::{error::CliError, templating::*, version_tx_cmd};
use ash_sdk::conf::AshConfig;
use clap::{Parser, Subcommand};

/// Interact with Avalanche networks
#[derive(Parser)]
#[command()]
pub(crate) struct NetworkCommand {
    #[command(subcommand)]
    command: NetworkSubcommands,
}

#[derive(Subcommand)]
enum NetworkSubcommands {
    /// List known Avalanche networks
    #[command(version = version_tx_cmd(false))]
    List,
}

// List available Avalanche networks
fn list(config: Option<&str>, json: bool) -> Result<(), CliError> {
    let networks = AshConfig::load(config)
        .map_err(|e| CliError::configerr(format!("Error listing networks: {e}")))?
        .avalanche_networks;

    if json {
        // Print the list of networks in JSON format
        // Only keep the name of the networks
        let networks = networks
            .iter()
            .map(|network| network.name.clone())
            .collect::<Vec<String>>();
        println!("{}", serde_json::to_string(&networks).unwrap());
        return Ok(());
    }

    println!("Available Avalanche networks:");
    for network in networks {
        println!("  - '{}'", type_colorize(&network.name));
    }

    Ok(())
}

// Parse network subcommand
pub(crate) fn parse(
    network: NetworkCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match network.command {
        NetworkSubcommands::List => list(config, json),
    }
}
