// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the blockchain subcommand parser

use crate::{
    avalanche::{wallet::*, *},
    utils::{error::CliError, parsing::*, templating::*},
};
use ash_sdk::avalanche::{
    blockchains::AvalancheBlockchain,
    vms::{subnet_evm::AVAX_SUBNET_EVM_ID, AvalancheVmType},
};
use async_std::task;
use clap::{Parser, Subcommand};

/// Interact with Avalanche blockchains
#[derive(Parser)]
#[command()]
pub(crate) struct BlockchainCommand {
    #[command(subcommand)]
    command: BlockchainSubcommands,
    /// Avalanche network
    #[arg(
        long,
        short = 'n',
        default_value = "mainnet",
        global = true,
        env = "AVALANCHE_NETWORK"
    )]
    network: String,
}

#[derive(Subcommand)]
enum BlockchainSubcommands {
    /// Create a new blockchain
    #[command()]
    Create {
        /// Blockchain name
        name: String,
        /// Blockchain VM type
        #[arg(long, short = 't', default_value = "SubnetEVM")]
        vm_type: AvalancheVmType,
        /// Blockchain VM ID
        #[arg(
            long,
            short = 'i',
            default_value = AVAX_SUBNET_EVM_ID,
        )]
        vm_id: String,
        /// Blockchain genesis data string (hex encoded)
        #[arg(long, short = 'g', group = "genesis")]
        genesis_str: Option<String>,
        /// Path to a JSON file containing the blockchain genesis data (generated with `ash avalanche vm encode-genesis`)
        #[arg(long, short = 'f', group = "genesis")]
        genesis_file: Option<String>,
        /// Subnet ID to create the blockchain on
        #[arg(long, short = 's')]
        subnet_id: String,
        /// Private key to sign the transaction with (must be a control key)
        #[arg(long, short = 'p', env = "AVALANCHE_PRIVATE_KEY")]
        private_key: String,
        /// Private key format
        #[arg(
            long,
            short = 'e',
            default_value = "cb58",
            env = "AVALANCHE_KEY_ENCODING"
        )]
        key_encoding: PrivateKeyEncoding,
        /// Whether to wait for transaction acceptance
        #[arg(long, short = 'w')]
        wait: bool,
    },
}

fn create(
    network_name: &str,
    subnet_id: &str,
    name: &str,
    vm_type: AvalancheVmType,
    vm_id: &str,
    genesis_data: Option<String>,
    genesis_file: Option<String>,
    private_key: &str,
    key_encoding: PrivateKeyEncoding,
    wait: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    // Check how genesis data is provided
    // If a file is provided, load it and parse the genesis data
    let genesis_hex = match genesis_file {
        Some(path) => {
            let genesis_json = std::fs::read_to_string(path)
                .map_err(|e| CliError::dataerr(format!("Error reading genesis file: {e}")))?;
            let genesis_obj: serde_json::Value = serde_json::from_str(&genesis_json)
                .map_err(|e| CliError::dataerr(format!("Error parsing genesis file: {e}")))?;
            genesis_obj
                .get("genesisBytes")
                .ok_or_else(|| {
                    CliError::dataerr(
                        "Error parsing genesis file: it should contain a 'genesisBytes' field"
                            .to_string(),
                    )
                })?
                .as_str()
                .ok_or_else(|| {
                    CliError::dataerr(
                        "Error parsing genesis file: the 'genesisBytes' field should be a string"
                            .to_string(),
                    )
                })?
                .to_string()
        }
        None => match genesis_data {
            Some(data) => data,
            None => {
                return Err(CliError::dataerr(
                    "Error when parsing arguments: either 'genesis-str' or a 'genesis-file' must be provided".to_string(),
                ))
            }
        },
    };

    let network = load_network(network_name, config)?;
    let wallet = create_wallet(&network, private_key, key_encoding)?;
    let subnet_id_parsed = parse_id(subnet_id)?;
    let vm_id_parsed = parse_id(vm_id)?;
    let genesis_bytes = hex::decode(genesis_hex.trim_start_matches("0x"))
        .map_err(|e| CliError::dataerr(format!("Error decoding genesis data: {e}")))?;

    if wait {
        eprintln!("Waiting for transaction to be accepted...");
    }

    let blockchain = task::block_on(async {
        AvalancheBlockchain::create(
            &wallet,
            subnet_id_parsed,
            name,
            vm_type,
            vm_id_parsed,
            genesis_bytes,
            wait,
        )
        .await
    })
    .map_err(|e| CliError::dataerr(format!("Error creating blockchain: {e}")))?;

    if json {
        println!("{}", serde_json::to_string(&blockchain).unwrap());
        return Ok(());
    }

    println!("{}", template_blockchain_creation(&blockchain, wait));

    Ok(())
}

// Parse blockchain subcommand
pub(crate) fn parse(
    subnet: BlockchainCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match subnet.command {
        BlockchainSubcommands::Create {
            name,
            vm_type,
            vm_id,
            genesis_str,
            genesis_file,
            subnet_id,
            private_key,
            key_encoding,
            wait,
        } => create(
            &subnet.network,
            &subnet_id,
            &name,
            vm_type,
            &vm_id,
            genesis_str,
            genesis_file,
            &private_key,
            key_encoding,
            wait,
            config,
            json,
        ),
    }
}
