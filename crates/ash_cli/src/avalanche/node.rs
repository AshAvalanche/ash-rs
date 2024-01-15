// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the node subcommand parser

use crate::utils::{error::CliError, templating::*, version_tx_cmd};
use ash_sdk::avalanche::nodes::{
    generate_node_bls_key, generate_node_id, node_id_from_cert_pem, AvalancheNode, BlsPrivateKey,
};
use clap::{Parser, Subcommand};
use std::{fs, path};

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
        #[arg(long, short = 'c', group = "cert")]
        pem_str: Option<String>,
        /// Path to the PEM-encoded X509 certificate file
        #[arg(long, short = 'f', group = "cert")]
        pem_file: Option<String>,
    },
    /// Generate a new node ID with its certificate and key files
    #[command(version = version_tx_cmd(false))]
    GenerateId {
        /// Path to the output directory where to create the cert and key files
        #[arg(long, short = 'o', global = true)]
        output_dir: Option<String>,
    },
    /// Get the BLS proof of possession (and public key) from the private key
    #[command(version = version_tx_cmd(false))]
    PopFromBlsKey {
        /// Hex-encoded BLS private key string (with the leading '0x')
        #[arg(long, short = 'k', group = "key")]
        key_str: Option<String>,
        /// Path to the BLS private key file
        #[arg(long, short = 'f', group = "key")]
        key_file: Option<String>,
    },
    /// Generate a new BLS private key along with its proof of possession (and public key)
    #[command(version = version_tx_cmd(false))]
    GenerateBlsKey {
        /// Path to the output directory where to create the private key file
        #[arg(long, short = 'o', global = true)]
        output_dir: Option<String>,
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

fn generate_id(output_dir: Option<String>, json: bool) -> Result<(), CliError> {
    let (node_id, cert_pem, key_pem) = generate_node_id(vec![])
        .map_err(|e| CliError::dataerr(format!("Error generating node ID: {e}")))?;

    if let Some(dir) = &output_dir {
        let output_path = path::Path::new(dir);

        // Create output directory if it doesn't exist
        if !output_path.exists() {
            fs::create_dir_all(output_path)
                .map_err(|e| CliError::dataerr(format!("Error creating output directory: {e}")))?;
        }

        // Write cert and key files
        let cert_file = output_path.join("node.crt");
        let key_file = output_path.join("node.key");
        fs::write(cert_file, &cert_pem)
            .map_err(|e| CliError::dataerr(format!("Error writing cert file: {e}")))?;
        fs::write(key_file, &key_pem)
            .map_err(|e| CliError::dataerr(format!("Error writing key file: {e}")))?;
    }

    if json {
        println!(
            "{}",
            serde_json::json!({ "nodeID": node_id,
                "cert": match &output_dir {
                    Some(output_dir) => format!("{}/node.crt", output_dir),
                    None => cert_pem
                },
                "key": match &output_dir {
                    Some(output_dir) => format!("{}/node.key", output_dir),
                    None => key_pem
                }
            })
        );
        return Ok(());
    }

    println!("Node ID: {}", type_colorize(&node_id.to_string()));

    if output_dir.is_some() {
        println!(
            "Certificate and key files written to '{}/node.crt' and '{}/node.key'",
            output_dir.as_ref().unwrap(),
            output_dir.as_ref().unwrap()
        );
    } else {
        println!(
            "Certificate:\n{}\nKey:\n{}",
            type_colorize(&cert_pem),
            type_colorize(&key_pem)
        );
    }

    Ok(())
}

fn pop_from_bls_key(
    key_str: Option<String>,
    key_file: Option<String>,
    json: bool,
) -> Result<(), CliError> {
    let bls_key = match (key_str, key_file) {
        (Some(key), None) => hex::decode(&key[2..])
            .map_err(|e| CliError::dataerr(format!("Error decoding BLS key: {e}")))?,
        (None, Some(key_file)) => fs::read(key_file)
            .map_err(|e| CliError::dataerr(format!("Error reading BLS key file: {e}")))?,
        _ => {
            return Err(CliError::dataerr(
                "Error when parsing arguments: either 'key' or 'key-file' must be provided"
                    .to_string(),
            ))
        }
    };

    let bls_pop = BlsPrivateKey::from_bytes(&bls_key)
        .map_err(|e| CliError::dataerr(format!("Error generating BLS proof of possession: {e}")))?
        .to_proof_of_possession();

    if json {
        println!(
            "{}",
            serde_json::json!(
                { "proofOfPossession": bls_pop })
        );
        return Ok(());
    }

    println!(
        "BLS public key: {}\nBLS proof of possession: {}",
        type_colorize(&format!("0x{}", hex::encode(&bls_pop.public_key))),
        type_colorize(&format!("0x{}", hex::encode(&bls_pop.proof_of_possession)))
    );

    Ok(())
}

fn generate_bls_key(output_dir: Option<String>, json: bool) -> Result<(), CliError> {
    let (bls_key, bls_pop) = generate_node_bls_key()
        .map_err(|e| CliError::dataerr(format!("Error generating BLS key: {e}")))?;

    if let Some(dir) = &output_dir {
        let output_path = path::Path::new(dir);

        // Create output directory if it doesn't exist
        if !output_path.exists() {
            fs::create_dir_all(output_path)
                .map_err(|e| CliError::dataerr(format!("Error creating output directory: {e}")))?;
        }

        // Write key file
        let key_file = output_path.join("bls.key");
        fs::write(key_file, bls_key.to_bytes())
            .map_err(|e| CliError::dataerr(format!("Error writing key file: {e}")))?;
    }

    if json {
        println!(
            "{}",
            serde_json::json!({
                "privateKey": match &output_dir {
                    Some(output_dir) => format!("{}/bls.key", output_dir),
                    None => format!("0x{}", hex::encode(bls_key.to_bytes()))
                },
                "proofOfPossession": bls_pop
            })
        );
        return Ok(());
    }

    println!(
        "BLS public key: {}\nBLS proof of possession: {}",
        type_colorize(&format!("0x{}", hex::encode(&bls_pop.public_key))),
        type_colorize(&format!("0x{}", hex::encode(&bls_pop.proof_of_possession)))
    );

    if output_dir.is_some() {
        println!(
            "BLS private key file written to '{}/bls.key'",
            output_dir.as_ref().unwrap()
        );
    } else {
        println!(
            "BLS private key: {}",
            type_colorize(&format!("0x{}", hex::encode(bls_key.to_bytes())))
        );
    }

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
        NodeSubcommands::GenerateId { output_dir } => generate_id(output_dir, json),
        NodeSubcommands::PopFromBlsKey { key_str, key_file } => {
            pop_from_bls_key(key_str, key_file, json)
        }
        NodeSubcommands::GenerateBlsKey { output_dir } => generate_bls_key(output_dir, json),
    }
}
