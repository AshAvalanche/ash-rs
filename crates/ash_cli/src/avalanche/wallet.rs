// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the wallet subcommand parser

use crate::{
    avalanche::*,
    utils::{error::CliError, templating::*, version_tx_cmd},
};
use ash_sdk::avalanche::wallets::{generate_private_key, AvalancheWallet, AvalancheWalletInfo};
use clap::{Parser, Subcommand, ValueEnum};
use std::fmt::Display;

/// Interact with Avalanche wallets
#[derive(Parser)]
#[command()]
pub(crate) struct WalletCommand {
    #[command(subcommand)]
    command: WalletSubcommands,
    /// Avalanche network
    #[arg(
        long,
        short = 'n',
        default_value = "fuji",
        global = true,
        env = "AVALANCHE_NETWORK"
    )]
    network: String,
}

#[derive(Subcommand)]
enum WalletSubcommands {
    /// Get information about a wallet (linked to a private key)
    #[command(version = version_tx_cmd(false))]
    Info {
        /// Private key of the wallet
        #[arg(env = "AVALANCHE_PRIVATE_KEY")]
        private_key: String,
        /// Private key format
        #[arg(long, short = 'e', default_value = "cb58")]
        key_encoding: PrivateKeyEncoding,
    },
    /// Randomly generate a private key (giving access to a wallet)
    #[command(version = version_tx_cmd(false))]
    Generate,
}

#[derive(Display, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum PrivateKeyEncoding {
    Cb58,
    Hex,
}

// Create a wallet from a private key
pub(crate) fn create_wallet(
    network: &AvalancheNetwork,
    private_key: &str,
    key_encoding: PrivateKeyEncoding,
) -> Result<AvalancheWallet, CliError> {
    let wallet = match key_encoding {
        PrivateKeyEncoding::Cb58 => network.create_wallet_from_cb58(private_key),
        PrivateKeyEncoding::Hex => network.create_wallet_from_hex(private_key),
    }
    .map_err(|e| CliError::dataerr(format!("Error creating wallet from private key: {e}")))?;

    Ok(wallet)
}

fn info(
    network_name: &str,
    private_key: &str,
    key_encoding: PrivateKeyEncoding,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let network = load_network(network_name, config)?;

    let wallet = create_wallet(&network, private_key, key_encoding)?;

    let wallet_info: AvalancheWalletInfo = wallet.into();

    if json {
        println!("{}", serde_json::to_string(&wallet_info).unwrap());
        return Ok(());
    }

    println!("{}", template_wallet_info(&wallet_info, 0));

    Ok(())
}

fn generate(json: bool) -> Result<(), CliError> {
    let private_key = generate_private_key()
        .map_err(|e| CliError::dataerr(format!("Error generating private key: {e}")))?;

    if json {
        println!(
            "{}",
            serde_json::json!({ "cb58": private_key.to_cb58(), "hex": private_key.to_hex() })
        );
        return Ok(());
    }

    println!(
        "{}",
        template_generate_private_key(&private_key.to_cb58(), &private_key.to_hex(), 0)
    );

    Ok(())
}

// Parse wallet subcommand
pub(crate) fn parse(
    wallet: WalletCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match wallet.command {
        WalletSubcommands::Info {
            private_key,
            key_encoding,
        } => info(&wallet.network, &private_key, key_encoding, config, json),
        WalletSubcommands::Generate => generate(json),
    }
}
