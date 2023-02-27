// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the subnet subcommand parser

use crate::error::CliError;
use ash::avalanche::{subnets::AvalancheSubnet, AvalancheNetwork};
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
    #[command(about = "List the network's subnets")]
    List,
    #[command(about = "Show subnet information")]
    Info {
        #[arg(long, help = "Subnet ID (CB58)")]
        id: String,
    },
}

// Load the network configuation and recursively update the subnets (and their blockchains)
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

// List the network's subnets
fn list(network_name: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let network = load_network_and_update_subnets(network_name, config)?;

    if json {
        println!("{}", serde_json::to_string(&network.subnets).unwrap());
        return Ok(());
    }

    println!(
        "Found {} subnet(s) on '{}':",
        network.subnets.len(),
        network.name
    );
    for subnet in network.subnets.iter() {
        print_info(subnet, true);
    }
    Ok(())
}

fn info(network: &str, id: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let network = load_network_and_update_subnets(network, config)?;
    let subnet = network
        .get_subnet(id)
        .ok_or_else(|| CliError::dataerr(format!("Subnet '{id}' not found")))?;

    if json {
        println!("{}", serde_json::to_string(&subnet).unwrap());
        return Ok(());
    }

    print_info(subnet, false);
    Ok(())
}

// Print subnet information (when not in JSON mode)
fn print_info(subnet: &AvalancheSubnet, separator: bool) {
    let subnet_id_line = format!("Subnet '{}':", subnet.id);

    if separator {
        // Print a separator of the same length as `subnet_id_line`
        println!("{}", "-".repeat(subnet_id_line.len()));
    }

    // Print ID, number of blockchains, blockchains IDs and names
    println!("{subnet_id_line}");
    println!("  Number of blockchains: {}", subnet.blockchains.len());
    println!("  Blockchains:");
    for blockchain in subnet.blockchains.iter() {
        println!("  - {}:", blockchain.name);
        println!("      ID:      {}", blockchain.id);
        println!("      VM type: {}", blockchain.vm_type);
        println!("      RPC URL: {}", blockchain.rpc_url);
    }
}

// Parse subnet subcommand
pub fn parse(subnet: SubnetCommand, config: Option<&str>, json: bool) -> Result<(), CliError> {
    match subnet.command {
        SubnetCommands::List => list(&subnet.network, config, json),
        SubnetCommands::Info { id } => info(&subnet.network, &id, config, json),
    }
}
