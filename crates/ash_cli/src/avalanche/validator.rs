// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the validator subcommand parser

use crate::{
    avalanche::{wallet::*, *},
    utils::{error::CliError, parsing::*, templating::*, version_tx_cmd},
};
use ash_sdk::avalanche::{
    nodes::ProofOfPossession, subnets::AvalancheSubnetType, AVAX_PRIMARY_NETWORK_ID,
};
use async_std::task;
use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};
use std::fmt::Display;

/// Node signer format
#[derive(Display, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum SignerFormat {
    Str,
    Json,
}

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
    #[command(version = version_tx_cmd(true))]
    Add {
        /// Validator NodeID
        id: String,
        /// Validator weight (permissioned Subnet) or stake in AVAX (elastic Subnet)
        stake_or_weight: u64,
        /// Start time of the validation (YYYY-MM-DDTHH:MM:SSZ), defaults to now
        #[arg(long, short = 'S')]
        start_time: Option<String>,
        /// End time of the validation (YYYY-MM-DDTHH:MM:SSZ)
        #[arg(long, short = 'E')]
        end_time: String,
        /// Delegation fee (percentage), defaults to 2%
        #[arg(long, short = 'f', default_value = "2")]
        delegation_fee: u32,
        /// Private key to sign the transaction with
        #[arg(long, short = 'p', env = "AVALANCHE_PRIVATE_KEY")]
        private_key: String,
        /// Private key encoding (cb58 or hex)
        #[arg(
            long,
            short = 'e',
            default_value = "cb58",
            env = "AVALANCHE_KEY_ENCODING"
        )]
        key_encoding: PrivateKeyEncoding,
        /// Signer (BLS public key and PoP) in "public_key:PoP" or JSON format
        /// (e.g. '{"publicKey":"public_key","proofOfPossession":"pop"}')
        #[arg(long, short = 'B')]
        signer: Option<String>,
        /// Signer format (str or json)
        #[arg(long, short = 'F', default_value = "str")]
        signer_format: SignerFormat,
        /// Whether to wait for transaction acceptance
        #[arg(long, short = 'w')]
        wait: bool,
    },
    /// List the Subnet's validators
    #[command(version = version_tx_cmd(false))]
    List {
        /// List pending validators
        #[arg(long, short = 'p')]
        pending: bool,
    },
    /// Show validator information
    #[command(version = version_tx_cmd(false))]
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

    update_subnet_validators(&mut network, subnet_id)?;
    subnet = network
        .get_subnet(parse_id(subnet_id)?)
        .map_err(|e| CliError::dataerr(format!("Error listing validators: {e}")))?;
    validators = subnet.validators.clone();
    format!(
        "Found {} validators on Subnet '{}':",
        type_colorize(&subnet.validators.len()),
        type_colorize(&subnet_id)
    );

    if json {
        println!("{}", serde_json::to_string(&validators).unwrap());
        return Ok(());
    }

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
    start_time: Option<String>,
    end_time: String,
    delegation_fee: u32,
    private_key: &str,
    key_encoding: PrivateKeyEncoding,
    signer: Option<String>,
    signer_format: SignerFormat,
    wait: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let node_id_parsed = parse_node_id(id)?;
    let start_time_parsed = match start_time {
        Some(start_time) => parse_datetime(&start_time)?,
        None => Utc::now(),
    };
    let end_time_parsed = parse_datetime(&end_time)?;
    let signer_parsed = match signer.clone() {
        Some(signer_str) => match signer_format {
            SignerFormat::Str => {
                let parts: Vec<&str> = signer_str.split(':').collect();
                if parts.len() != 2 {
                    return Err(CliError::dataerr(
                        "Signer must be in the format 'public_key:PoP'".to_string(),
                    ));
                }
                serde_json::from_value::<ProofOfPossession>(serde_json::json!({
                    "publicKey": parts[0],
                    "proofOfPossession": parts[1]
                }))
                .map_err(|e| CliError::dataerr(format!("Error parsing signer: {e}")))?
            }
            SignerFormat::Json => serde_json::from_str(&signer_str)
                .map_err(|e| CliError::dataerr(format!("Error parsing signer: {e}")))?,
        },
        None => ProofOfPossession::default(),
    };

    let mut network = load_network(network_name, config)?;
    update_network_subnets(&mut network)?;

    let subnet = network
        .get_subnet(parse_id(subnet_id)?)
        .map_err(|e| CliError::dataerr(format!("Error loading Subnet info: {e}")))?;
    let wallet = create_wallet(&network, private_key, key_encoding)?;

    if wait {
        eprintln!("Waiting for transaction to be accepted...");
    }

    let validator = match subnet.subnet_type {
        AvalancheSubnetType::PrimaryNetwork => task::block_on(async {
            subnet
                .add_validator_permissionless(
                    &wallet,
                    node_id_parsed,
                    subnet.id,
                    // Multiply by 1 billion to convert from AVAX to nAVAX
                    stake_or_weight * 1_000_000_000,
                    start_time_parsed,
                    end_time_parsed,
                    delegation_fee,
                    match signer {
                        Some(_) => Some(signer_parsed),
                        None => None,
                    },
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
            signer,
            signer_format,
            wait,
        } => add(
            &validator.network,
            &validator.subnet_id,
            &id,
            stake_or_weight,
            start_time,
            end_time,
            delegation_fee,
            &private_key,
            key_encoding,
            signer,
            signer_format,
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
