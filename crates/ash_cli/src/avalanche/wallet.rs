// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the wallet subcommand parser

use crate::utils::error::CliError;
use crate::utils::templating::template_wallet_info;
use crate::{avalanche::*, utils::templating::template_generate_private_key};
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
    #[command()]
    Info {
        /// Private key of the wallet
        #[arg(env = "AVALANCHE_PRIVATE_KEY")]
        private_key: String,
        /// Private key format
        #[arg(long, short = 'f', default_value = "cb58")]
        key_format: PrivateKeyFormat,
    },
    /// Randomly generate a private key (giving access to a wallet)
    #[command()]
    Generate {
        /// Private key format
        #[arg(long, short = 'f', default_value = "cb58")]
        key_format: PrivateKeyFormat,
    },
}

#[derive(Display, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum PrivateKeyFormat {
    Cb58,
    Hex,
}

// Create a wallet from a private key
pub(crate) fn create_wallet(
    network: &AvalancheNetwork,
    private_key: &str,
    key_format: PrivateKeyFormat,
) -> Result<AvalancheWallet, CliError> {
    let wallet = match key_format {
        PrivateKeyFormat::Cb58 => network.create_wallet_from_cb58(private_key),
        PrivateKeyFormat::Hex => network.create_wallet_from_hex(private_key),
    }
    .map_err(|e| CliError::dataerr(format!("Error creating wallet from private key: {e}")))?;

    Ok(wallet)
}

fn info(
    network_name: &str,
    private_key: &str,
    key_format: PrivateKeyFormat,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let network = load_network(network_name, config)?;

    let wallet = create_wallet(&network, private_key, key_format)?;

    let wallet_info: AvalancheWalletInfo = wallet.into();

    if json {
        println!("{}", serde_json::to_string(&wallet_info).unwrap());
        return Ok(());
    }

    println!("{}", template_wallet_info(&wallet_info, 0));

    Ok(())
}

fn generate(key_format: PrivateKeyFormat, json: bool) -> Result<(), CliError> {
    let private_key = generate_private_key()
        .map_err(|e| CliError::dataerr(format!("Error generating private key: {e}")))?;

    let private_key_string = match key_format {
        PrivateKeyFormat::Cb58 => private_key.to_cb58(),
        PrivateKeyFormat::Hex => private_key.to_hex(),
    };

    if json {
        println!(
            "{}",
            serde_json::json!({ "privateKey": private_key_string })
        );
        return Ok(());
    }

    println!(
        "{}",
        template_generate_private_key(&private_key_string, &key_format.to_string(), 0)
    );

    Ok(())
}

// Parse subnet subcommand
pub(crate) fn parse(
    wallet: WalletCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match wallet.command {
        WalletSubcommands::Info {
            private_key,
            key_format,
        } => info(&wallet.network, &private_key, key_format, config, json),
        WalletSubcommands::Generate { key_format } => generate(key_format, json),
    }
}
