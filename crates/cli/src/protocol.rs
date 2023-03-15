// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

mod node;

// Module that contains the protocol subcommand parser

use crate::utils::error::CliError;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Interact with the Ash protocol")]
pub(crate) struct ProtocolCommand {
    #[command(subcommand)]
    command: ProtocolSubcommands,
    #[arg(
        long,
        help = "Avalanche network",
        default_value = "mainnet",
        global = true
    )]
    network: String,
}

#[derive(Subcommand)]
enum ProtocolSubcommands {
    Node(node::NodeCommand),
}

// Parse protocol subcommand
pub(crate) fn parse(
    protocol: ProtocolCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match protocol.command {
        ProtocolSubcommands::Node(node) => node::parse(node, config, json),
    }
}
