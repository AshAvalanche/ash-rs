// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the node subcommand parser

use crate::avalanche::load_network;
use crate::utils::{error::CliError, templating::*};
use ash::protocol::{nodes::AshNode, AshProtocol};
use async_std::task;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Interact with Ash nodes")]
pub(crate) struct NodeCommand {
    #[command(subcommand)]
    command: NodeSubcommands,
    #[arg(
        long,
        help = "Avalanche network",
        default_value = "mainnet",
        global = true
    )]
    network: String,
}

#[derive(Subcommand)]
enum NodeSubcommands {
    #[command(about = "Show node information")]
    Info {
        #[arg(long, help = "Node ID (CB58 or hex string)")]
        id: String,
    },
    #[command(about = "List the Ash protocol's nodes")]
    List,
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

// List nodes registered on the protocol
fn list(network_name: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let network = load_network(network_name, config)?;
    let mut protocol = AshProtocol::new(&network, config)
        .map_err(|e| CliError::dataerr(format!("Error loading protocol: {e}")))?;

    task::block_on(async {
        protocol
            .update_nodes()
            .await
            .map_err(|e| CliError::dataerr(format!("Error updating nodes: {e}")))?;
        Ok::<(), CliError>(())
    })?;

    if json {
        println!(
            "{}",
            serde_json::to_string(
                &protocol
                    .nodes
                    .iter()
                    .map(|node| node.info())
                    .collect::<Vec<_>>()
            )
            .unwrap()
        );
        return Ok(());
    }

    println!(
        "{} node(s) registed to the Ash protocol on '{}':",
        type_colorize(&protocol.nodes.len()),
        type_colorize(&network.name)
    );
    for node in protocol.nodes.iter() {
        println!("{}", template_ash_node_info(&node.info(), true, 0));
    }

    Ok(())
}

// Parse node subcommand
pub(crate) fn parse(node: NodeCommand, config: Option<&str>, json: bool) -> Result<(), CliError> {
    match node.command {
        NodeSubcommands::Info { id } => info(&id, json),
        NodeSubcommands::List => list(&node.network, config, json),
    }
}
