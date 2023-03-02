// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the node subcommand parser

use crate::utils::{error::CliError, templating::template_avalanche_node_info};
use ash::avalanche::nodes::AvalancheNode;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Interact with Avalanche nodes")]
pub struct NodeCommand {
    #[command(subcommand)]
    command: NodeCommands,
}

#[derive(Subcommand)]
enum NodeCommands {
    #[command(about = "Show Avalanche node information")]
    Info {
        #[arg(
            long,
            default_value = "127.0.0.1",
            help = "Node's HTTP host (IP address or FQDN)"
        )]
        http_host: String,
        #[arg(long, default_value = "9650", help = "Node's HTTP port")]
        http_port: u16,
    },
}

// Create a new node and update its info
fn create_and_update_info(http_host: &str, http_port: u16) -> Result<AvalancheNode, CliError> {
    let mut node = AvalancheNode {
        http_host: http_host.to_string(),
        http_port,
        ..Default::default()
    };

    node.update_info()
        .map_err(|e| CliError::dataerr(format!("Error updating node info: {e}")))?;

    Ok(node)
}

fn info(http_host: &str, http_port: u16, json: bool) -> Result<(), CliError> {
    let node = create_and_update_info(http_host, http_port)?;

    if json {
        println!("{}", serde_json::to_string(&node).unwrap());
        return Ok(());
    }

    println!("{}", template_avalanche_node_info(&node, 0));

    Ok(())
}

// Parse node subcommand
pub fn parse(node: NodeCommand, json: bool) -> Result<(), CliError> {
    match node.command {
        NodeCommands::Info {
            http_host,
            http_port,
        } => info(&http_host, http_port, json),
    }
}
