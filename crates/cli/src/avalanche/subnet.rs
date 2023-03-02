// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the subnet subcommand parser

use crate::utils::{error::CliError, templating::*};
use ash::avalanche::AvalancheNetwork;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Interact with Avalanche Subnets")]
pub struct SubnetCommand {
    #[command(subcommand)]
    command: SubnetCommands,
    #[arg(
        long,
        help = "Avalanche network",
        default_value = "mainnet",
        global = true
    )]
    network: String,
}

#[derive(Subcommand)]
enum SubnetCommands {
    #[command(about = "List the network's Subnets")]
    List,
    #[command(about = "Show Subnet information")]
    Info {
        #[arg(long, help = "Subnet ID (CB58)")]
        id: String,
    },
}

// Load the network configuation and recursively update the Subnets (and their blockchains)
fn load_network_and_update_subnets(
    network_name: &str,
    config: Option<&str>,
) -> Result<AvalancheNetwork, CliError> {
    let mut network = AvalancheNetwork::load(network_name, config)
        .map_err(|e| CliError::dataerr(format!("Error loading network: {e}")))?;
    network
        .update_subnets()
        .map_err(|e| CliError::dataerr(format!("Error updating subnets: {e}")))?;
    network
        .update_blockchains()
        .map_err(|e| CliError::dataerr(format!("Error updating blockchains: {e}")))?;

    Ok(network)
}

// Update a Subnet's validators
fn update_subnet_validators(
    network: &mut AvalancheNetwork,
    subnet_id: &str,
) -> Result<(), CliError> {
    network
        .update_subnet_validators(subnet_id)
        .map_err(|e| CliError::dataerr(format!("Error updating validators: {e}")))?;
    Ok(())
}

// List the network's Subnets
fn list(network_name: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let network = load_network_and_update_subnets(network_name, config)?;

    if json {
        println!("{}", serde_json::to_string(&network.subnets).unwrap());
        return Ok(());
    }

    println!(
        "Found {} Subnet(s) on '{}':",
        network.subnets.len(),
        network.name
    );
    for subnet in network.subnets.iter() {
        println!("{}", template_subnet_info(subnet, true, 0));
    }
    Ok(())
}

fn info(network: &str, id: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut network = load_network_and_update_subnets(network, config)?;
    update_subnet_validators(&mut network, id).map_err(|e| CliError::dataerr(e.message))?;

    let subnet = network
        .get_subnet(id)
        .ok_or_else(|| CliError::dataerr(format!("Subnet '{id}' not found")))?;

    if json {
        println!("{}", serde_json::to_string(&subnet).unwrap());
        return Ok(());
    }

    println!("{}", template_subnet_info(subnet, false, 0));
    Ok(())
}

// Parse subnet subcommand
pub fn parse(subnet: SubnetCommand, config: Option<&str>, json: bool) -> Result<(), CliError> {
    match subnet.command {
        SubnetCommands::Info { id } => info(&subnet.network, &id, config, json),
        SubnetCommands::List => list(&subnet.network, config, json),
    }
}
