// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the helper subcommand parser

use crate::{
    console::{
        create_api_config_with_access_token, load_console, project::get_current_project_id_or_name,
    },
    utils::{error::CliError, templating::*, version_tx_cmd},
};
use ash_sdk::console::{self, api_models::project};
use async_std::task;
use clap::{Parser, Subcommand};

/// Ash Console helper
#[derive(Parser)]
#[command()]
pub(crate) struct HelperCommand {
    #[command(subcommand)]
    command: HelperSubcommands,
    /// Console project ID or name
    /// Defaults to the current project
    #[arg(
        long,
        short = 'p',
        default_value = "current",
        global = true,
        env = "ASH_CONSOLE_PROJECT"
    )]
    project_id_or_name: String,
}

#[derive(Subcommand)]
enum HelperSubcommands {
    /// Show helpful information to stake on an Avalanche node
    #[command(version = version_tx_cmd(false))]
    Stake {
        /// Node resource ID or name
        node_resource_id_or_name: String,
    },
}

// Show helpful information to stake on an Avalanche node
fn staking_helper(
    project_id_or_name: &str,
    node_resource_id_or_name: &str,
    config: Option<&str>,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let spinner = spinner_with_message("Fetching node information...".to_string());

    let project_response = task::block_on(async {
        console::api::get_project_by_id_or_name(&api_config, project_id_or_name)
            .await
            .map_err(|e| CliError::dataerr(format!("Error getting project: {e}")))
    })?;

    if project_response.network.unwrap() == console::api_models::project::Network::Local {
        return Err(CliError::dataerr(
            "Staking is not supported on local (devnet) projects".to_string(),
        ));
    }

    let node_response = task::block_on(async {
        console::api::get_project_resource_by_id_or_name(
            &api_config,
            project_id_or_name,
            node_resource_id_or_name,
        )
        .await
        .map_err(|e| CliError::dataerr(format!("Error getting node resource: {e}")))
    })?;

    let node_id_secret_id = match *node_response.resource_type.unwrap() {
        console::api_models::ResourceType::AvalancheNode => {
            node_response.node_id_secret_id.unwrap()
        }
        _ => {
            return Err(CliError::dataerr(
                "Resource is not an `avalancheNode`!".to_string(),
            ))
        }
    };

    let node_id_secret_response = task::block_on(async {
        console::api::get_secret_by_id_or_name(&api_config, &node_id_secret_id)
            .await
            .map_err(|e| CliError::dataerr(format!("Error getting node ID secret: {e}")))
    })?;

    let node_id = node_id_secret_response.node_id.unwrap();

    spinner.finish_and_clear();

    println!(
        "To stake on your node '{}':\n1. Navigate to the Core 'Stake/Validate' wizard at https://{}core.app/stake/validate/\n2. Select the staking amount\n3. Provide the following Node ID: {}\n4. Complete the 'Stake/Validate' form and sign the transaction!",
        type_colorize(&node_resource_id_or_name),
        if project_response.network.unwrap() == project::Network::Testnet {
            "test.".to_string()
        } else { "".to_string() },
        type_colorize(&node_id)
    );

    Ok(())
}

// Parse helper subcommand
pub(crate) fn parse(operation: HelperCommand, config: Option<&str>) -> Result<(), CliError> {
    let mut project_id_or_name = operation.project_id_or_name;

    // Get the current project ID
    if project_id_or_name == "current" {
        project_id_or_name = get_current_project_id_or_name()?;
    }

    match operation.command {
        HelperSubcommands::Stake {
            node_resource_id_or_name,
        } => staking_helper(&project_id_or_name, &node_resource_id_or_name, config),
    }
}
