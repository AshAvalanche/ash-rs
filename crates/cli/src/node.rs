// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the node subcommand parser

use crate::utils::{error::CliError, templating::template_ash_node_info};
use ash::nodes::AshNode;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Interact with Ash nodes")]
pub(crate) struct NodeCommand {
    #[command(subcommand)]
    command: NodeSubcommands,
}

#[derive(Subcommand)]
enum NodeSubcommands {
    #[command(about = "Show node information")]
    Info {
        #[arg(long, help = "Node ID (CB58 or hex string)")]
        id: String,
    },
}

// Display node information
fn info(id: &str, json: bool) -> Result<(), CliError> {
    let node_info = AshNode::from_string(id)
        .map_err(|e| CliError::dataerr(format!("Error loading info: {e}")))?
        .info();

    if json {
        println!("{}", serde_json::to_string(&node_info).unwrap());
        return Ok(());
    }

    println!("{}", template_ash_node_info(&node_info, false, 0));

    Ok(())
}

// Parse node subcommand
pub(crate) fn parse(node: NodeCommand, json: bool) -> Result<(), CliError> {
    match node.command {
        NodeSubcommands::Info { id } => info(&id, json),
    }
}
