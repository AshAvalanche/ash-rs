// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the subnet subcommand parser

use crate::avalanche::{load_network_and_update_subnets, update_subnet_validators};
use crate::utils::{error::CliError, templating::*};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Interact with Avalanche Subnets")]
pub(crate) struct SubnetCommand {
    #[command(subcommand)]
    command: SubnetSubcommands,
    #[arg(
        long,
        help = "Avalanche network",
        default_value = "mainnet",
        global = true
    )]
    network: String,
}

#[derive(Subcommand)]
enum SubnetSubcommands {
    #[command(about = "List the network's Subnets")]
    List,
    #[command(about = "Show Subnet information")]
    Info {
        #[arg(long, help = "Subnet ID")]
        id: String,
    },
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
        type_colorize(&network.subnets.len()),
        type_colorize(&network.name)
    );
    for subnet in network.subnets.iter() {
        println!("{}", template_subnet_info(subnet, true, 0));
    }
    Ok(())
}

fn info(network_name: &str, id: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut network = load_network_and_update_subnets(network_name, config)?;
    update_subnet_validators(&mut network, id)?;

    let subnet = network
        .get_subnet(id)
        .map_err(|e| CliError::dataerr(format!("Error loading Subnet info: {e}")))?;

    if json {
        println!("{}", serde_json::to_string(&subnet).unwrap());
        return Ok(());
    }

    println!("{}", template_subnet_info(subnet, false, 0));
    Ok(())
}

// Parse subnet subcommand
pub(crate) fn parse(
    subnet: SubnetCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match subnet.command {
        SubnetSubcommands::Info { id } => info(&subnet.network, &id, config, json),
        SubnetSubcommands::List => list(&subnet.network, config, json),
    }
}
