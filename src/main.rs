// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

use ash::node::AshNode;
use clap::{Parser, Subcommand};
use serde_json;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Ash CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,
    #[arg(long, help = "Output in JSON format")]
    json: bool,
}

#[derive(Subcommand)]
enum CliCommands {
    Node(Node),
}

#[derive(Parser)]
#[command(about = "Interact with Ash nodes", long_about = None)]
struct Node {
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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        CliCommands::Node(node) => match &node.command {
            NodeCommands::Info { id } => match AshNode::from_string(id) {
                Ok(node) => {
                    let node_info = node.info();

                    if cli.json {
                        println!("{}", serde_json::to_string(&node_info).unwrap());
                        return;
                    }

                    println!("Node information for {}", id);
                    println!("  Node ID (CB58): {}", node_info.id.cb58);
                    println!("  Node ID (hex): {}", node_info.id.hex);
                }
                Err(e) => println!("Error loading info: {}", e),
            },
        },
    }
}
