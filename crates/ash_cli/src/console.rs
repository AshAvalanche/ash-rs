// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

mod auth;
pub(crate) mod blueprint;
mod operation;
mod project;
mod region;
mod resource;
mod secret;

// Module that contains the console subcommand parser

use crate::utils::error::CliError;
use ash_sdk::console::{api_config::Configuration, AshConsole};
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
    Blueprint(blueprint::BlueprintCommand),
    Operation(operation::OperationCommand),
    Project(project::ProjectCommand),
    Region(region::RegionCommand),
    Resource(resource::ResourceCommand),
    Secret(secret::SecretCommand),
}

const KEYRING_TARGET: &str = "ash-console";
const KEYRING_ACCESS_TOKEN_SERVICE: &str = "access_token";
const KEYRING_REFRESH_TOKEN_SERVICE: &str = "refresh_token";
const KEYRING_FALLBACK_FILES_DIR: &str = "~/.ash-console/tokens";

// Load the console configuation
fn load_console(config: Option<&str>) -> Result<AshConsole, CliError> {
    AshConsole::load(config).map_err(|e| CliError::dataerr(format!("Error loading console: {e}")))
}

// Create a new Ash Console API configuration with the current access token
fn create_api_config_with_access_token(
    console: &mut AshConsole,
) -> Result<Configuration, CliError> {
    let access_token = auth::get_access_token(console)?;

    Ok(console.create_api_config_with_access_token(&access_token))
}

// Parse console subcommand
pub(crate) fn parse(
    console: ConsoleCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match console.command {
        ConsoleSubcommands::Auth(auth) => auth::parse(auth, config, json),
        ConsoleSubcommands::Blueprint(blueprint) => blueprint::parse(blueprint, config),
        ConsoleSubcommands::Operation(operation) => operation::parse(operation, config, json),
        ConsoleSubcommands::Project(project) => project::parse(project, config, json),
        ConsoleSubcommands::Region(region) => region::parse(region, config, json),
        ConsoleSubcommands::Resource(resource) => resource::parse(resource, config, json),
        ConsoleSubcommands::Secret(secret) => secret::parse(secret, config, json),
    }
}
