// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the operation subcommand parser

use crate::{
    console::{create_api_config_with_access_token, load_console},
    utils::{error::CliError, templating::*, version_tx_cmd},
};
use ash_sdk::console;
use async_std::task;
use clap::{Parser, Subcommand};

/// Explore Ash Console operations
#[derive(Parser)]
#[command()]
pub(crate) struct OperationCommand {
    #[command(subcommand)]
    command: OperationSubcommands,
}

#[derive(Subcommand)]
enum OperationSubcommands {
    /// List the Console operations known to the user
    #[command(version = version_tx_cmd(false))]
    List {
        /// Date and time from which to retrieve operations
        /// e.g.: "2021-01-01T00:00:00Z"
        #[arg(long, short = 'f')]
        from: Option<String>,
        /// Date and time until which to retrieve operations
        /// e.g.: "2021-01-01T00:00:00Z"
        #[arg(long, short = 't')]
        to: Option<String>,
        /// Whether to show extended information (e.g. full IDs)
        #[arg(long, short = 'e')]
        extended: bool,
    },
    /// Show information about a Console operation
    #[command(version = version_tx_cmd(false))]
    Info {
        /// Operation ID
        operation_id: String,
        /// Whether to show extended information (e.g. full IDs)
        #[arg(long, short = 'e')]
        extended: bool,
    },
}

// List cloud operations of a project
fn list(
    from: Option<String>,
    to: Option<String>,
    extended: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let response =
        task::block_on(async { console::api::get_all_operations(&api_config, from, to).await })
            .map_err(|e| CliError::dataerr(format!("Error getting operations: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!("{}", template_operations_table(response, extended, 0));

    Ok(())
}

// Show information about a cloud operation
fn info(
    operation_id: &str,
    extended: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let response = task::block_on(async {
        console::api::get_operation_by_id(&api_config, operation_id)
            .await
            .map_err(|e| CliError::dataerr(format!("Error getting operation: {e}")))
    })?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!("{}", template_operations_table(vec![response], extended, 0));

    Ok(())
}

// Parse operation subcommand
pub(crate) fn parse(
    operation: OperationCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match operation.command {
        OperationSubcommands::List { from, to, extended } => list(from, to, extended, config, json),
        OperationSubcommands::Info {
            operation_id,
            extended,
        } => info(&operation_id, extended, config, json),
    }
}
