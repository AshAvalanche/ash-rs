// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the node subcommand parser

use crate::error::CliError;
use ash::nodes::AshNode;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Interact with Ash nodes")]
pub struct NodeCommand {
    #[command(subcommand)]
    command: NodeCommands,
}

#[derive(Subcommand)]
enum NodeCommands {
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

    println!("Node '{id}':");
    println!("  Node ID (CB58): {}", node_info.id.cb58);
    println!("  Node ID (hex): {}", node_info.id.hex);
    Ok(())
}

// Parse node subcommand
pub fn parse(node: NodeCommand, json: bool) -> Result<(), CliError> {
    match node.command {
        NodeCommands::Info { id } => info(&id, json),
    }
}
