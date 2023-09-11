// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the node subcommand parser

use crate::utils::{error::CliError, templating::*, version_tx_cmd};
use ash_sdk::avalanche::nodes::{node_id_from_cert_pem, AvalancheNode};
use clap::{Parser, Subcommand};

/// Interact with Avalanche nodes
#[derive(Parser)]
#[command()]
pub(crate) struct NodeCommand {
    #[command(subcommand)]
    command: NodeSubcommands,
}

#[derive(Subcommand)]
enum NodeSubcommands {
    /// Show node information
    #[command(version = version_tx_cmd(false))]
    Info {
        /// Node's HTTP host (IP address or FQDN)
        #[arg(long, short = 'n', default_value = "127.0.0.1", global = true)]
        http_host: String,
        /// Node's HTTP port
        #[arg(long, short = 'p', default_value = "9650", global = true)]
        http_port: u16,
        /// Use HTTPS
        #[arg(long, short = 's', global = true)]
        https: bool,
    },
    /// Check if a chain is done bootstrapping on the node
    #[command(version = version_tx_cmd(false))]
    IsBootstrapped {
        /// Node's HTTP host (IP address or FQDN)
        #[arg(long, short = 'n', default_value = "127.0.0.1", global = true)]
        http_host: String,
        /// Node's HTTP port
        #[arg(long, short = 'p', default_value = "9650", global = true)]
        http_port: u16,
        /// Use HTTPS
        #[arg(long, short = 's', global = true)]
        https: bool,
        /// Chain ID or alias
        chain: String,
    },
    /// Get the node ID from the PEM-encoded X509 certificate
    #[command(version = version_tx_cmd(false))]
    IdFromCert {
        /// PEM-encoded X509 certificate string
        #[arg(long, short = 'p', group = "cert")]
        pem_str: Option<String>,
        /// Path to the PEM-encoded X509 certificate file
        #[arg(long, short = 'f', group = "cert")]
        pem_file: Option<String>,
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

fn id_from_cert(
    cert_str: Option<String>,
    cert_file: Option<String>,
    json: bool,
) -> Result<(), CliError> {
    let cert_pem =
        match (cert_str, cert_file) {
            (Some(cert_str), None) => cert_str,
            (None, Some(cert_file)) => std::fs::read_to_string(cert_file)
                .map_err(|e| CliError::dataerr(format!("Error reading certificate file: {e}")))?,
            _ => return Err(CliError::dataerr(
                "Error when parsing arguments: either 'cert-str' or 'cert-file' must be provided"
                    .to_string(),
            )),
        };

    let node_id = node_id_from_cert_pem(&cert_pem)
        .map_err(|e| CliError::dataerr(format!("Error getting node ID from certificate: {e}")))?;

    if json {
        println!("{}", serde_json::json!({ "nodeID": node_id }));
        return Ok(());
    }

    println!("Node ID: {}", type_colorize(&node_id.to_string()));

    Ok(())
}

// Parse node subcommand
pub(crate) fn parse(node: NodeCommand, json: bool) -> Result<(), CliError> {
    match node.command {
        NodeSubcommands::Info {
            http_host,
            http_port,
            https,
        } => info(&http_host, http_port, https, json),
        NodeSubcommands::IsBootstrapped {
            http_host,
            http_port,
            https,
            chain,
        } => is_bootstrapped(&http_host, http_port, https, &chain, json),
        NodeSubcommands::IdFromCert { pem_str, pem_file } => id_from_cert(pem_str, pem_file, json),
    }
}
