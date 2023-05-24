// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

mod network;
mod node;
mod subnet;
mod validator;
mod wallet;
mod x;

// Module that contains the avalanche subcommand parser

use crate::utils::error::CliError;
use ash_sdk::{
    avalanche::AvalancheNetwork,
    ids::{node::Id as NodeId, Id},
};
use clap::{Parser, Subcommand};
use std::str::FromStr;

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
    X(x::XCommand),
    Wallet(wallet::WalletCommand),
}

// Parse an ID from a string
fn parse_id(id: &str) -> Result<Id, CliError> {
    let id = Id::from_str(id).map_err(|e| CliError::dataerr(format!("Error parsing ID: {e}")))?;
    Ok(id)
}

// Parse a node ID from a string
fn parse_node_id(id: &str) -> Result<NodeId, CliError> {
    let id = NodeId::from_str(id)
        .map_err(|e| CliError::dataerr(format!("Error parsing NodeID: {e}")))?;
    Ok(id)
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
        AvalancheSubcommands::X(x) => x::parse(x, config, json),
        AvalancheSubcommands::Wallet(wallet) => wallet::parse(wallet, config, json),
    }
}
