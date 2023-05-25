// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the subnet subcommand parser

use crate::{
    avalanche::{wallet::*, *},
    utils::{error::CliError, templating::*},
};
use ash_sdk::avalanche::subnets::AvalancheSubnet;
use async_std::task;
use clap::{Parser, Subcommand};

/// Interact with Avalanche Subnets
#[derive(Parser)]
#[command()]
pub(crate) struct SubnetCommand {
    #[command(subcommand)]
    command: SubnetSubcommands,
    /// Avalanche network
    #[arg(
        long,
        short = 'n',
        default_value = "mainnet",
        global = true,
        env = "AVALANCHE_NETWORK"
    )]
    network: String,
}

#[derive(Subcommand)]
enum SubnetSubcommands {
    /// List the network's Subnets
    #[command()]
    List,
    /// Show Subnet information
    #[command()]
    Info {
        /// Subnet ID
        id: String,
    },
    /// Create a new Subnet
    #[command()]
    Create {
        #[arg(long, short = 'p', env = "AVALANCHE_PRIVATE_KEY")]
        private_key: String,
        /// Private key format
        #[arg(
            long,
            short = 'e',
            default_value = "cb58",
            env = "AVALANCHE_KEY_ENCODING"
        )]
        key_encoding: PrivateKeyEncoding,
        /// Whether to wait for transaction acceptance
        #[arg(long, short = 'w')]
        wait: bool,
    },
}

// List the network's Subnets
fn list(network_name: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut network = load_network(network_name, config)?;
    update_network_subnets(&mut network)?;

    if json {
        println!("{}", serde_json::to_string(&network.subnets).unwrap());
        return Ok(());
    }

    println!(
        "Found {} Subnet(s) on network '{}':",
        type_colorize(&network.subnets.len()),
        type_colorize(&network.name)
    );
    for subnet in network.subnets.iter() {
        println!("{}", template_subnet_info(subnet, true, 0));
    }

    Ok(())
}

fn info(network_name: &str, id: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut network = load_network(network_name, config)?;
    update_network_subnets(&mut network)?;
    update_subnet_validators(&mut network, id)?;

    let subnet = network
        .get_subnet(parse_id(id)?)
        .map_err(|e| CliError::dataerr(format!("Error loading Subnet info: {e}")))?;

    if json {
        println!("{}", serde_json::to_string(&subnet).unwrap());
        return Ok(());
    }

    println!("{}", template_subnet_info(subnet, false, 0));

    Ok(())
}

fn create(
    network_name: &str,
    private_key: &str,
    key_encoding: PrivateKeyEncoding,
    wait: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let network = load_network(network_name, config)?;
    let wallet = create_wallet(&network, private_key, key_encoding)?;

    if wait {
        eprintln!("Waiting for transaction to be accepted...");
    }

    let subnet = task::block_on(async { AvalancheSubnet::create(&wallet, wait).await })
        .map_err(|e| CliError::dataerr(format!("Error creating Subnet: {e}")))?;

    if json {
        println!("{}", serde_json::to_string(&subnet).unwrap());
        return Ok(());
    }

    println!("{}", template_subnet_creation(&subnet, wait));

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
        SubnetSubcommands::Create {
            private_key,
            key_encoding,
            wait,
        } => create(
            &subnet.network,
            &private_key,
            key_encoding,
            wait,
            config,
            json,
        ),
    }
}
