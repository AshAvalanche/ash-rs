// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the validator subcommand parser

use crate::{
    avalanche::*,
    utils::{error::CliError, templating::*},
};
use ash_sdk::avalanche::AVAX_PRIMARY_NETWORK_ID;
use clap::{Parser, Subcommand};

/// Interact with Avalanche validators
#[derive(Parser)]
#[command()]
pub(crate) struct ValidatorCommand {
    #[command(subcommand)]
    command: ValidatorSubcommands,
    /// Avalanche network
    #[arg(
        long,
        short = 'n',
        default_value = "mainnet",
        global = true,
        env = "AVALANCHE_NETWORK"
    )]
    network: String,
    /// Avalanche Subnet ID
    #[arg(
        long,
        short = 's',
        default_value = AVAX_PRIMARY_NETWORK_ID,
        global = true
    )]
    subnet_id: String,
}

#[derive(Subcommand)]
enum ValidatorSubcommands {
    /// List the Subnet's validators
    #[command()]
    List,
    /// Show validator information
    #[command()]
    Info {
        /// Validator NodeID
        id: String,
    },
}

// List the Subnet's validators
fn list(
    network_name: &str,
    subnet_id: &str,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut network = load_network(network_name, config)?;
    update_network_subnets(&mut network)?;
    update_subnet_validators(&mut network, subnet_id)?;

    let subnet = network
        .get_subnet(parse_id(subnet_id)?)
        .map_err(|e| CliError::dataerr(format!("Error listing validators: {e}")))?;

    if json {
        println!("{}", serde_json::to_string(&subnet.validators).unwrap());
        return Ok(());
    }

    println!(
        "Found {} validator(s) on Subnet '{}':",
        type_colorize(&subnet.validators.len()),
        type_colorize(&subnet_id)
    );
    for validator in subnet.validators.iter() {
        println!(
            "{}",
            template_validator_info(validator, subnet, true, 2, true)
        );
    }

    Ok(())
}

fn info(
    network_name: &str,
    subnet_id: &str,
    id: &str,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut network = load_network(network_name, config)?;
    update_network_subnets(&mut network)?;
    update_subnet_validators(&mut network, subnet_id)?;

    let subnet = network
        .get_subnet(parse_id(subnet_id)?)
        .map_err(|e| CliError::dataerr(format!("Error loading Subnet info: {e}")))?;

    let validator = subnet
        .get_validator(parse_node_id(id)?)
        .map_err(|e| CliError::dataerr(format!("Error loading Subnet info: {e}")))?;

    if json {
        println!("{}", serde_json::to_string(&validator).unwrap());
        return Ok(());
    }
    println!(
        "{}",
        template_validator_info(validator, subnet, false, 0, true)
    );

    Ok(())
}

// Parse validator subcommand
pub(crate) fn parse(
    validator: ValidatorCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match validator.command {
        ValidatorSubcommands::Info { id } => {
            info(&validator.network, &validator.subnet_id, &id, config, json)
        }
        ValidatorSubcommands::List => list(&validator.network, &validator.subnet_id, config, json),
    }
}
