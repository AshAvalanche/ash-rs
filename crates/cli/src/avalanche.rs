// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

mod network;
mod node;
mod subnet;
mod validator;

// Module that contains the avalanche subcommand parser

use crate::utils::error::CliError;
use ash::avalanche::AvalancheNetwork;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Interact with Avalanche Subnets, blockchains and nodes")]
pub(crate) struct AvalancheCommand {
    #[command(subcommand)]
    command: AvalancheSubcommands,
}

#[derive(Subcommand)]
enum AvalancheSubcommands {
    Network(network::NetworkCommand),
    Node(node::NodeCommand),
    Subnet(subnet::SubnetCommand),
    Validator(validator::ValidatorCommand),
}

// Load the network configuation
pub(crate) fn load_network(
    network_name: &str,
    config: Option<&str>,
) -> Result<AvalancheNetwork, CliError> {
    let network = AvalancheNetwork::load(network_name, config)
        .map_err(|e| CliError::dataerr(format!("Error loading network: {e}")))?;
    Ok(network)
}

// Recursively update the Subnets (and their blockchains)
pub(crate) fn update_network_subnets(network: &mut AvalancheNetwork) -> Result<(), CliError> {
    network
        .update_subnets()
        .map_err(|e| CliError::dataerr(format!("Error updating subnets: {e}")))?;
    network
        .update_blockchains()
        .map_err(|e| CliError::dataerr(format!("Error updating blockchains: {e}")))?;
    Ok(())
}

// Update a Subnet's validators
pub(crate) fn update_subnet_validators(
    network: &mut AvalancheNetwork,
    subnet_id: &str,
) -> Result<(), CliError> {
    network
        .update_subnet_validators(subnet_id)
        .map_err(|e| CliError::dataerr(format!("Error updating validators: {e}")))?;
    Ok(())
}

// Parse avalanche subcommand
pub(crate) fn parse(
    avalanche: AvalancheCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match avalanche.command {
        AvalancheSubcommands::Network(network) => network::parse(network, config, json),
        AvalancheSubcommands::Node(node) => node::parse(node, json),
        AvalancheSubcommands::Subnet(subnet) => subnet::parse(subnet, config, json),
        AvalancheSubcommands::Validator(validator) => validator::parse(validator, config, json),
    }
}
