// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the conf subcommand parser

use crate::utils::{error::CliError, version_tx_cmd};
use ash_sdk::conf::AshConfig;
use clap::{Parser, Subcommand};

#[derive(Parser)]
/// Interact with Ash configuration files
#[command()]
pub(crate) struct ConfCommand {
    #[command(subcommand)]
    command: ConfSubcommands,
}

#[derive(Subcommand)]
enum ConfSubcommands {
    /// Initialize an Ash config file
    #[command(version = version_tx_cmd(false))]
    Init {
        #[arg(from_global)]
        config: String,
        /// Overwrite existing config file
        #[arg(long)]
        force: bool,
    },
}

// Initialize an Ash config file
fn init(config: String, force: bool) -> Result<(), CliError> {
    AshConfig::dump_default(&config, force)
        .map_err(|e| CliError::cantcreat(format!("Error initializing config file: {e}")))?;

    println!("Config file initialized at '{config}'");
    Ok(())
}

// Parse conf subcommand
pub(crate) fn parse(conf: ConfCommand) -> Result<(), CliError> {
    match conf.command {
        ConfSubcommands::Init { config, force } => init(config, force),
    }
}
