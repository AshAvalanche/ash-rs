// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the vm subcommand parser

use crate::{
    avalanche::*,
    utils::{error::CliError, parsing::*, templating::*},
};
use async_std::task;
use clap::{Parser, Subcommand};
use colored::Colorize;

/// Interact with Avalanche Warp Messaging
#[derive(Parser)]
#[command()]
pub(crate) struct WarpCommand {
    #[command(subcommand)]
    command: WarpSubcommands,
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
enum WarpSubcommands {
    /// Navigate the Warp: monitor Avalanche Warp Messages sent between chains
    #[command()]
    Navigate {
        /// Source chain ID or name
        source_chain: String,
        /// Block from which to start monitoring
        #[arg(long, short = 'f', default_value = "earliest")]
        from_block: String,
        /// Block at which to stop monitoring
        #[arg(long, short = 't', default_value = "latest")]
        to_block: String,
        /// Show extended information (notably signatures)
        /// This option is only available in non-JSON mode
        #[arg(long, short = 'e')]
        extended: bool,
    },
}

fn navigate(
    network_name: &str,
    source_chain: &str,
    from_block: &str,
    to_block: &str,
    extended: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    // Display warning about the experimental nature of this feature
    eprintln!(
        "{}",
        "Warning: this feature is experimental and may break at any time."
            .yellow()
            .bold()
    );

    let mut network = load_network(network_name, config)?;
    update_network_subnets(&mut network)?;

    // Try loading the blockchain by its ID or by its name depending on whether source_chain is an ID
    let blockchain_id = parse_id(source_chain);
    let blockchain = match blockchain_id {
        Ok(id) => network
            .get_blockchain(id)
            .map_err(|e| CliError::dataerr(format!("Error loading blockchain info: {e}")))?,
        Err(_) => network
            .get_blockchain_by_name(source_chain)
            .map_err(|e| CliError::dataerr(format!("Error loading blockchain info: {e}")))?,
    }
    .clone();
    update_subnet_validators(&mut network, &blockchain.subnet_id.to_string())?;

    let subnet = network
        .get_subnet(blockchain.subnet_id)
        .map_err(|e| CliError::dataerr(format!("Error loading subnet info: {e}")))?;

    let warp_messages =
        task::block_on(async { blockchain.get_warp_messages(from_block, to_block).await })
            .map_err(|e| CliError::dataerr(format!("Error reading warp messages: {e}")))?
            .iter()
            .map(|warp_message| {
                let mut signed_warp_message = warp_message.clone();
                let signatures = subnet
                    .get_warp_message_node_signatures(warp_message, None)
                    .unwrap_or(vec![]);
                for sig in signatures {
                    signed_warp_message.add_node_signature(sig);
                }
                signed_warp_message
            })
            .collect::<Vec<_>>();

    if json {
        println!("{}", serde_json::to_string(&warp_messages).unwrap());
        return Ok(());
    }

    println!("Found {} Warp messages:", warp_messages.len());
    for warp_message in warp_messages {
        println!(
            "{}",
            template_warp_message(&warp_message, &blockchain, extended, true, 0)
        );
    }

    Ok(())
}

// Parse warp subcommand
pub(crate) fn parse(warp: WarpCommand, config: Option<&str>, json: bool) -> Result<(), CliError> {
    match warp.command {
        WarpSubcommands::Navigate {
            source_chain,
            from_block,
            to_block,
            extended,
        } => navigate(
            &warp.network,
            &source_chain,
            &from_block,
            &to_block,
            extended,
            config,
            json,
        ),
    }
}
