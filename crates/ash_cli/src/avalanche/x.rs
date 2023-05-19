// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the x subcommand parser

use crate::utils::error::CliError;
use crate::utils::templating::template_xchain_transfer;
use crate::{avalanche::wallet::*, avalanche::*, utils::templating::template_xchain_balance};
use async_std::task;
use clap::{Parser, Subcommand};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

/// Interact with Avalanche X-Chain
#[derive(Parser)]
#[command()]
pub(crate) struct XCommand {
    #[command(subcommand)]
    command: XSubcommands,
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
enum XSubcommands {
    /// Get the balance of an address for a given asset
    #[command()]
    Balance {
        /// Address to get the balance of
        address: String,
        /// Asset ID to get the balance of
        #[arg(long, short = 'a', default_value = "AVAX")]
        asset_id: String,
    },
    /// Transfer any amount of a given asset to an address
    #[command()]
    Transfer {
        /// Amount of asset to send (in AVAX equivalent, 1 AVAX = 10^9 nAVAX)
        amount: f64,
        /// Address to send the asset to
        to: String,
        /// Asset ID to send
        #[arg(long, short = 'a', default_value = "AVAX")]
        asset_id: String,
        /// Private key to sign the transaction with
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

fn balance(
    network_name: &str,
    address: &str,
    asset_id: &str,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let network = load_network(network_name, config)?;

    let balance = network.get_xchain_balance(address, asset_id).map_err(|e| {
        CliError::dataerr(format!("Error getting balance for address {address}: {e}"))
    })?;

    if json {
        println!("{}", serde_json::to_string(&balance).unwrap());
        return Ok(());
    }

    println!(
        "{}",
        template_xchain_balance(address, asset_id, &balance, 0)
    );

    Ok(())
}

fn transfer(
    network_name: &str,
    to: &str,
    asset_id: &str,
    amount: f64,
    private_key: &str,
    key_encoding: PrivateKeyEncoding,
    wait: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let network = load_network(network_name, config)?;

    let wallet = create_wallet(&network, private_key, key_encoding)?;

    if wait {
        eprintln!("Waiting for transaction to be accepted...");
    }

    let tx_id = task::block_on(async {
        wallet
            .transfer_avax_xchain(
                to,
                (Decimal::from_f64(amount).unwrap() * Decimal::from_f64(1_000_000_000.0).unwrap())
                    .to_u64()
                    .unwrap(),
                wait,
            )
            .await
    })
    .map_err(|e| {
        CliError::dataerr(format!(
            "Error transferring {amount} of asset '{asset_id}' to '{to}': {e}"
        ))
    })?;

    if json {
        println!("{}", serde_json::json!({ "txID": tx_id.to_string() }));
        return Ok(());
    }

    println!(
        "{}",
        template_xchain_transfer(&tx_id.to_string(), to, asset_id, amount, wait, 0)
    );

    Ok(())
}

// Parse subnet subcommand
pub(crate) fn parse(x: XCommand, config: Option<&str>, json: bool) -> Result<(), CliError> {
    match x.command {
        XSubcommands::Balance { address, asset_id } => {
            balance(&x.network, &address, &asset_id, config, json)
        }
        XSubcommands::Transfer {
            to,
            asset_id,
            amount,
            private_key,
            key_encoding,
            wait,
        } => transfer(
            &x.network,
            &to,
            &asset_id,
            amount,
            &private_key,
            key_encoding,
            wait,
            config,
            json,
        ),
    }
}
