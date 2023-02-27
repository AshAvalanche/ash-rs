// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the avalanche subcommand parser

mod network;
mod subnet;

use crate::error::CliError;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Interact with Avalanche Subnets, blockchains and nodes")]
pub struct AvalancheCommand {
    #[command(subcommand)]
    command: AvalancheCommands,
}

#[derive(Subcommand)]
enum AvalancheCommands {
    Subnet(subnet::SubnetCommand),
    Network(network::NetworkCommand),
}

// Parse subnet subcommand
pub fn parse(
    avalanche: AvalancheCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match avalanche.command {
        AvalancheCommands::Subnet(subnet) => subnet::parse(subnet, config, json),
        AvalancheCommands::Network(network) => network::parse(network, config, json),
    }
}
