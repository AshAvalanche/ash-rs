// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the network subcommand parser

use crate::utils::error::CliError;
use ash::conf::AshConfig;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Interact with Avalanche networks")]
pub struct NetworkCommand {
    #[command(subcommand)]
    command: NetworkCommands,
}

#[derive(Subcommand)]
enum NetworkCommands {
    #[command(about = "List Avalanche networks")]
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
        println!("  - '{}'", network.name);
    }
    Ok(())
}

// Parse network subcommand
pub fn parse(network: NetworkCommand, config: Option<&str>, json: bool) -> Result<(), CliError> {
    match network.command {
        NetworkCommands::List => list(config, json),
    }
}
