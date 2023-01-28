// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

// Module that contains the network subcommand  parser

use ash::conf::AshConfig;
use clap::{Parser, Subcommand};
use std::process::exit;

#[derive(Parser)]
#[command(about = "Interact with Avalanche networks", long_about = None)]
pub struct NetworkCommand {
    #[command(subcommand)]
    command: NetworkCommands,
}

#[derive(Subcommand)]
enum NetworkCommands {
    #[command(about = "List Avalanche networks", long_about = None)]
    List,
}

// List available Avalanche networks
fn list(config: Option<&str>, json: bool) {
    let networks = match AshConfig::load(config) {
        Ok(ash_config) => ash_config.avalanche_networks,
        Err(err) => {
            eprintln!("Error listing networks: {}", err);
            exit(exitcode::CONFIG);
        }
    };

    if json {
        // Print the list of networks in JSON format
        // Only keep the name of the networks
        let networks = networks
            .iter()
            .map(|network| network.name.clone())
            .collect::<Vec<String>>();
        println!("{}", serde_json::to_string(&networks).unwrap());
        return;
    }

    println!("Available Avalanche networks:");
    for network in networks {
        println!("  - '{}'", network.name);
    }
}

// Parse network subcommand
pub fn parse(network: NetworkCommand, config: Option<&str>, json: bool) {
    match network.command {
        NetworkCommands::List => list(config, json),
    }
}
