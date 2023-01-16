// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

// Module that contains the Ash CLI node subcommand parser

use ash::core::node::AshNode;
use clap::{Parser, Subcommand};
use serde_json;

#[derive(Parser)]
#[command(about = "Interact with Ash nodes", long_about = None)]
pub struct NodeCommand {
    #[command(subcommand)]
    command: NodeCommands,
}

#[derive(Subcommand)]
enum NodeCommands {
    #[command(about = "Show node information", long_about = None)]
    Info {
        #[arg(long, help = "Node ID (CB58 or hex string)")]
        id: String,
    },
}

// Display node information
fn info(id: &str, json: bool) {
    match AshNode::from_string(id) {
        Ok(node) => {
            let node_info = node.info();

            if json {
                println!("{}", serde_json::to_string(&node_info).unwrap());
                return;
            }

            println!("Node information for {}", id);
            println!("  Node ID (CB58): {}", node_info.id.cb58);
            println!("  Node ID (hex): {}", node_info.id.hex);
        }
        Err(e) => println!("Error loading info: {}", e),
    }
}

// Parse node subcommand
pub fn parse(node: &NodeCommand, json: bool) {
    match &node.command {
        NodeCommands::Info { id } => info(&id, json),
    }
}
