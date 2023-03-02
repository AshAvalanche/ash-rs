// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the avalanche subcommand parser

mod network;
mod node;
mod subnet;

use crate::utils::error::CliError;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Interact with Avalanche Subnets, blockchains and nodes")]
pub struct AvalancheCommand {
    #[command(subcommand)]
    command: AvalancheCommands,
}

#[derive(Subcommand)]
enum AvalancheCommands {
    Network(network::NetworkCommand),
    Node(node::NodeCommand),
    Subnet(subnet::SubnetCommand),
}

// Parse subnet subcommand
pub fn parse(
    avalanche: AvalancheCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match avalanche.command {
        AvalancheCommands::Network(network) => network::parse(network, config, json),
        AvalancheCommands::Node(node) => node::parse(node, json),
        AvalancheCommands::Subnet(subnet) => subnet::parse(subnet, config, json),
    }
}
