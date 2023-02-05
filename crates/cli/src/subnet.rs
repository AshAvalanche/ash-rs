// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

// Module that contains the subnet subcommand parser

use ash::avalanche::{subnets::AvalancheSubnet, AvalancheNetwork};
use clap::{Parser, Subcommand};
use std::process::exit;

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
    List,
    Info {
        #[arg(long, help = "Subnet ID (CB58)")]
        id: String,
    },
}

// List the network's subnets
fn list(network_name: &str, config: Option<&str>, json: bool) {
    match AvalancheNetwork::load(network_name, config) {
        Ok(mut network) => {
            match network.update_subnets() {
                Ok(_) => {
                    if json {
                        // Serialize the first `limit` subnets to JSON
                        println!(
                            "{}",
                            serde_json::to_string(
                                &network.subnets.iter().collect::<Vec<&AvalancheSubnet>>()
                            )
                            .unwrap()
                        );
                        return;
                    }

                    println!(
                        "Found {} subnet{} on '{}':",
                        network.subnets.len(),
                        if network.subnets.len() == 1 { "" } else { "s" },
                        network.name
                    );

                    // Print the first `limit` subnets
                    for subnet in network.subnets.iter() {
                        print_info(subnet, true);
                    }
                }
                Err(e) => {
                    eprintln!("Error updating subnets: {e}");
                    exit(exitcode::DATAERR);
                }
            }
        }
        Err(e) => {
            eprintln!("Error listing subnets: {e}");
            exit(exitcode::DATAERR);
        }
    }
}

fn info(network: &str, id: &str, config: Option<&str>, json: bool) {
    match AvalancheNetwork::load(network, config) {
        Ok(mut network) => match network.update_subnets() {
            Ok(_) => match network.get_subnet(id) {
                Some(subnet) => {
                    if json {
                        println!("{}", serde_json::to_string(&subnet).unwrap());
                        return;
                    }

                    print_info(subnet, false);
                }
                None => eprintln!("Subnet '{id}' not found"),
            },
            Err(e) => {
                eprintln!("Error updating subnets: {e}");
                exit(exitcode::DATAERR);
            }
        },
        Err(e) => {
            eprintln!("Error loading info: {e}");
            exit(exitcode::DATAERR);
        }
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
pub fn parse(subnet: SubnetCommand, config: Option<&str>, json: bool) {
    match subnet.command {
        SubnetCommands::List => list(&subnet.network, config, json),
        SubnetCommands::Info { id } => info(&subnet.network, &id, config, json),
    }
}
