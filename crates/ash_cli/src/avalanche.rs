// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

mod blockchain;
mod network;
mod node;
mod subnet;
mod validator;
mod vm;
mod wallet;
mod x;

// Module that contains the avalanche subcommand parser

use crate::utils::{error::CliError, parsing::*};
use ash_sdk::avalanche::AvalancheNetwork;
use clap::{Parser, Subcommand};

#[derive(Parser)]
/// Interact with Avalanche Subnets, blockchains and nodes
#[command(visible_alias = "avax")]
pub(crate) struct AvalancheCommand {
    #[command(subcommand)]
    command: AvalancheSubcommands,
}

#[derive(Subcommand)]
enum AvalancheSubcommands {
    Blockchain(blockchain::BlockchainCommand),
    Network(network::NetworkCommand),
    Node(node::NodeCommand),
    Subnet(subnet::SubnetCommand),
    Validator(validator::ValidatorCommand),
    Vm(vm::VmCommand),
    Wallet(wallet::WalletCommand),
    X(x::XCommand),
}

// Load the network configuation
fn load_network(network_name: &str, config: Option<&str>) -> Result<AvalancheNetwork, CliError> {
    let network = AvalancheNetwork::load(network_name, config)
        .map_err(|e| CliError::dataerr(format!("Error loading network: {e}")))?;
    Ok(network)
}

// Recursively update the Subnets (and their blockchains)
fn update_network_subnets(network: &mut AvalancheNetwork) -> Result<(), CliError> {
    network
        .update_subnets()
        .map_err(|e| CliError::dataerr(format!("Error updating subnets: {e}")))?;
    network
        .update_blockchains()
        .map_err(|e| CliError::dataerr(format!("Error updating blockchains: {e}")))?;
    Ok(())
}

// Update a Subnet's validators
fn update_subnet_validators(
    network: &mut AvalancheNetwork,
    subnet_id: &str,
) -> Result<(), CliError> {
    network
        .update_subnet_validators(parse_id(subnet_id)?)
        .map_err(|e| CliError::dataerr(format!("Error updating validators: {e}")))?;
    Ok(())
}

// Update a Subnet's pending validators
fn update_subnet_pending_validators(
    network: &mut AvalancheNetwork,
    subnet_id: &str,
) -> Result<(), CliError> {
    network
        .update_subnet_pending_validators(parse_id(subnet_id)?)
        .map_err(|e| CliError::dataerr(format!("Error updating pending validators: {e}")))?;
    Ok(())
}

// Parse avalanche subcommand
pub(crate) fn parse(
    avalanche: AvalancheCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match avalanche.command {
        AvalancheSubcommands::Blockchain(blockchain) => blockchain::parse(blockchain, config, json),
        AvalancheSubcommands::Network(network) => network::parse(network, config, json),
        AvalancheSubcommands::Node(node) => node::parse(node, json),
        AvalancheSubcommands::Subnet(subnet) => subnet::parse(subnet, config, json),
        AvalancheSubcommands::Validator(validator) => validator::parse(validator, config, json),
        AvalancheSubcommands::Vm(vm) => vm::parse(vm, json),
        AvalancheSubcommands::X(x) => x::parse(x, config, json),
        AvalancheSubcommands::Wallet(wallet) => wallet::parse(wallet, config, json),
    }
}
