// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

mod auth;

// Module that contains the console subcommand parser

use crate::utils::error::CliError;
use ash_sdk::console::AshConsole;
use clap::{Parser, Subcommand};

#[derive(Parser)]
/// Interact with the Ash Console
pub(crate) struct ConsoleCommand {
    #[command(subcommand)]
    command: ConsoleSubcommands,
}

#[derive(Subcommand)]
enum ConsoleSubcommands {
    Auth(auth::AuthCommand),
}

const KEYRING_TARGET: &str = "ash-console";
const KEYRING_ACCESS_TOKEN_SERVICE: &str = "access_token";
const KEYRING_REFRESH_TOKEN_SERVICE: &str = "refresh_token";

// Load the console configuation
fn load_console(config: Option<&str>) -> Result<AshConsole, CliError> {
    AshConsole::load(config).map_err(|e| CliError::dataerr(format!("Error loading console: {e}")))
}

// Parse console subcommand
pub(crate) fn parse(
    console: ConsoleCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match console.command {
        ConsoleSubcommands::Auth(auth) => auth::parse(auth, config, json),
    }
}
