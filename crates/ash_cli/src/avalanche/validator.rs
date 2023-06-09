// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the validator subcommand parser

use crate::{
    avalanche::{wallet::*, *},
    utils::{error::CliError, parsing::*, templating::*},
};
use ash_sdk::avalanche::{subnets::AvalancheSubnetType, AVAX_PRIMARY_NETWORK_ID};
use async_std::task;
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
    /// Add a validator to a Subnet
    #[command()]
    Add {
        /// Validator NodeID
        id: String,
        /// Validator weight (permissioned Subnet) or stake in AVAX (elastic Subnet)
        stake_or_weight: u64,
        /// Start time of the validation (YYYY-MM-DDTHH:MM:SSZ)
        #[arg(long, short = 'S')]
        start_time: String,
        /// End time of the validation (YYYY-MM-DDTHH:MM:SSZ)
        #[arg(long, short = 'E')]
        end_time: String,
        /// Delegation fee (percentage)
        #[arg(long, short = 'f', default_value = "2")]
        delegation_fee: u32,
        /// Private key to sign the transaction with
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
    /// List the Subnet's validators
    #[command()]
    List {
        /// List pending validators
        #[arg(long, short = 'p')]
        pending: bool,
    },
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
    pending: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut network = load_network(network_name, config)?;
    update_network_subnets(&mut network)?;
    let subnet;
    let validators;
    let first_line;

    match pending {
        true => {
            update_subnet_pending_validators(&mut network, subnet_id)?;
            subnet = network
                .get_subnet(parse_id(subnet_id)?)
                .map_err(|e| CliError::dataerr(format!("Error listing validators: {e}")))?;
            validators = subnet.pending_validators.clone();
            first_line = format!(
                "Found {} pending validator(s) on Subnet '{}':",
                type_colorize(&subnet.pending_validators.len()),
                type_colorize(&subnet_id)
            )
        }
        false => {
            update_subnet_validators(&mut network, subnet_id)?;
            subnet = network
                .get_subnet(parse_id(subnet_id)?)
                .map_err(|e| CliError::dataerr(format!("Error listing validators: {e}")))?;
            validators = subnet.validators.clone();
            first_line = format!(
                "Found {} validator(s) on Subnet '{}':",
                type_colorize(&subnet.validators.len()),
                type_colorize(&subnet_id)
            )
        }
    }

    if json {
        println!("{}", serde_json::to_string(&validators).unwrap());
        return Ok(());
    }

    println!("{}", first_line);
    for validator in validators.iter() {
        println!(
            "{}",
            template_validator_info(validator, subnet, true, true, 2)
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
        template_validator_info(validator, subnet, false, true, 0)
    );

    Ok(())
}

fn add(
    network_name: &str,
    subnet_id: &str,
    id: &str,
    stake_or_weight: u64,
    start_time: String,
    end_time: String,
    delegation_fee: u32,
    private_key: String,
    key_encoding: PrivateKeyEncoding,
    wait: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let node_id_parsed = parse_node_id(id)?;
    let start_time_parsed = parse_datetime(&start_time)?;
    let end_time_parsed = parse_datetime(&end_time)?;

    let mut network = load_network(network_name, config)?;
    update_network_subnets(&mut network)?;

    let subnet = network
        .get_subnet(parse_id(subnet_id)?)
        .map_err(|e| CliError::dataerr(format!("Error loading Subnet info: {e}")))?;
    let wallet = create_wallet(&network, &private_key, key_encoding)?;

    if wait {
        eprintln!("Waiting for transaction to be accepted...");
    }

    let validator = match subnet.subnet_type {
        AvalancheSubnetType::PrimaryNetwork => task::block_on(async {
            subnet
                .add_avalanche_validator(
                    &wallet,
                    node_id_parsed,
                    // Multiply by 1 billion to convert from AVAX to nAVAX
                    stake_or_weight * 1_000_000_000,
                    start_time_parsed,
                    end_time_parsed,
                    delegation_fee,
                    wait,
                )
                .await
        }),
        AvalancheSubnetType::Permissioned => task::block_on(async {
            subnet
                .add_validator_permissioned(
                    &wallet,
                    node_id_parsed,
                    stake_or_weight,
                    start_time_parsed,
                    end_time_parsed,
                    wait,
                )
                .await
        }),
        AvalancheSubnetType::Elastic => {
            return Err(CliError::dataerr(
                "Adding a validator to an elastic Subnet is not yet supported".to_string(),
            ));
        }
    }
    .map_err(|e| CliError::dataerr(format!("Error adding validator: {e}")))?;

    if json {
        println!("{}", serde_json::to_string(&validator).unwrap());
        return Ok(());
    }

    println!("{}", template_validator_add(&validator, subnet, wait));

    Ok(())
}

// Parse validator subcommand
pub(crate) fn parse(
    validator: ValidatorCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match validator.command {
        ValidatorSubcommands::Add {
            id,
            stake_or_weight,
            start_time,
            end_time,
            delegation_fee,
            private_key,
            key_encoding,
            wait,
        } => add(
            &validator.network,
            &validator.subnet_id,
            &id,
            stake_or_weight,
            start_time,
            end_time,
            delegation_fee,
            private_key,
            key_encoding,
            wait,
            config,
            json,
        ),
        ValidatorSubcommands::Info { id } => {
            info(&validator.network, &validator.subnet_id, &id, config, json)
        }
        ValidatorSubcommands::List { pending } => list(
            &validator.network,
            &validator.subnet_id,
            pending,
            config,
            json,
        ),
    }
}
