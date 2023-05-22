// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the node subcommand parser

use crate::utils::{error::CliError, templating::*};
use ash_sdk::avalanche::nodes::AvalancheNode;
use clap::{Parser, Subcommand};

/// Interact with Avalanche nodes
#[derive(Parser)]
#[command()]
pub(crate) struct NodeCommand {
    #[command(subcommand)]
    command: NodeSubcommands,
    /// Node's HTTP host (IP address or FQDN)
    #[arg(long, short = 'n', default_value = "127.0.0.1", global = true)]
    http_host: String,
    /// Node's HTTP port
    #[arg(long, short = 'p', default_value = "9650", global = true)]
    http_port: u16,
    /// Use HTTPS
    #[arg(long, short = 's', global = true)]
    https: bool,
}

#[derive(Subcommand)]
enum NodeSubcommands {
    /// Show node information
    #[command()]
    Info,
    /// Check if a chain is done bootstrapping on the node
    #[command()]
    IsBootstrapped {
        /// Chain ID or alias
        chain: String,
    },
}

// Create a new node and update its info
fn create_and_update_info(
    http_host: &str,
    http_port: u16,
    https_enabled: bool,
) -> Result<AvalancheNode, CliError> {
    let mut node = AvalancheNode {
        http_host: http_host.to_string(),
        http_port,
        https_enabled,
        ..Default::default()
    };

    node.update_info()
        .map_err(|e| CliError::dataerr(format!("Error updating node info: {e}")))?;

    Ok(node)
}

fn info(http_host: &str, http_port: u16, https_enabled: bool, json: bool) -> Result<(), CliError> {
    let node = create_and_update_info(http_host, http_port, https_enabled)?;

    if json {
        println!("{}", serde_json::to_string(&node).unwrap());
        return Ok(());
    }

    println!("{}", template_avalanche_node_info(&node, 0));

    Ok(())
}

fn is_bootstrapped(
    http_host: &str,
    http_port: u16,
    https_enabled: bool,
    chain: &str,
    json: bool,
) -> Result<(), CliError> {
    let node = AvalancheNode {
        http_host: http_host.to_string(),
        http_port,
        https_enabled,
        ..Default::default()
    };

    let is_bootstrapped = node
        .check_chain_bootstrapping(chain)
        .map_err(|e| CliError::dataerr(format!("Error checking if chain is bootstrapped: {e}")))?;

    if json {
        println!(
            "{}",
            serde_json::json!({ "isBootstrapped": is_bootstrapped })
        );
        return Ok(());
    }

    println!(
        "{}",
        template_chain_is_bootstrapped(&node, chain, is_bootstrapped, 0)
    );

    Ok(())
}

// Parse node subcommand
pub(crate) fn parse(node: NodeCommand, json: bool) -> Result<(), CliError> {
    match node.command {
        NodeSubcommands::Info => info(&node.http_host, node.http_port, node.https, json),
        NodeSubcommands::IsBootstrapped { chain } => {
            is_bootstrapped(&node.http_host, node.http_port, node.https, &chain, json)
        }
    }
}
