// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

// Module that contains subnet subcommand parser

use ash::avalanche::{AvalancheBlockchain, AvalancheNetwork, AvalancheSubnet};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Interact with Avalanche Subnets", long_about = None)]
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
    #[command(about = "List the network's subnets", long_about = None)]
    List {
        #[arg(
            long,
            help = "Limit the number of subnets to list",
            default_value = "10"
        )]
        limit: u32,
    },
    Info {
        #[arg(long, help = "Subnet ID (CB58)")]
        id: String,
    },
}

// List the network's subnets
fn list(network: &str, limit: u32, json: bool) {
    match AvalancheNetwork::new(network) {
        Ok(network) => {
            if json {
                // Serialize the first `limit` subnets to JSON
                println!(
                    "{}",
                    serde_json::to_string(
                        &network
                            .subnets
                            .values()
                            .into_iter()
                            .take(limit as usize)
                            .collect::<Vec<&AvalancheSubnet>>()
                    )
                    .unwrap()
                );
                return;
            }

            println!(
                "Found {} subnets on '{}':",
                network.subnets.len(),
                network.name
            );

            // Print the first `limit` subnets
            for subnet in network.subnets.values().take(limit as usize) {
                print_info(subnet, true);
            }
        }
        Err(e) => println!("Error listing subnets: {}", e),
    }
}

fn info(network: &str, id: &str, json: bool) {
    match AvalancheNetwork::new(network) {
        Ok(network) => match network.subnets.get(id) {
            Some(subnet) => {
                if json {
                    println!("{}", serde_json::to_string(&subnet).unwrap());
                    return;
                }

                print_info(subnet, false);
            }
            None => println!("Subnet '{}' not found", id),
        },
        Err(e) => println!("Error loading info: {}", e),
    }
}

// Print subnet information (when not in JSON mode)
fn print_info(subnet: &AvalancheSubnet, separator: bool) {
    let subnet_id_line = format!("Subnet '{}':", subnet.id);

    if separator {
        // Print a separator of the same length as `subnet_id_line`
        println!("{}", "-".repeat(subnet_id_line.len()));
    }

    // Print ID, number of blockchains, blockchains IDs and names
    println!("{}", subnet_id_line);
    println!("  Number of blockchains: {}", subnet.blockchains.len());
    println!("  Blockchains:");
    for blockchain in subnet.blockchains.values() {
        match blockchain {
            AvalancheBlockchain::Evm { name, id, .. } => {
                println!("  - {} (ID='{}')", name, id)
            }
        }
    }
}

// Parse subnet subcommand
pub fn parse(subnet: &SubnetCommand, json: bool) {
    match &subnet.command {
        SubnetCommands::List { limit } => list(&subnet.network, *limit, json),
        SubnetCommands::Info { id } => info(&subnet.network, id, json),
    }
}
