// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the blueprint subcommand parser

use crate::{
    console::{create_api_config_with_access_token, load_console, secret},
    utils::{
        error::CliError, file::read_file_or_stdin, prompt::confirm_action, templating::*,
        version_tx_cmd,
    },
};
use ash_sdk::console;
use async_std::task;
use clap::{Parser, Subcommand};
use colored::Colorize;
use serde::{Deserialize, Serialize};

/// Blueprint object
/// Allows to create multiple entities at once, e.g. a project with a region and a resource
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Blueprint {
    pub secrets: Vec<console::api_models::CreateSecretRequest>,
}

/// Interact with Ash Console entities
#[derive(Parser)]
#[command()]
pub(crate) struct BlueprintCommand {
    #[command(subcommand)]
    command: BlueprintSubcommands,
}

#[derive(Subcommand)]
enum BlueprintSubcommands {
    /// Apply a blueprint
    #[command(version = version_tx_cmd(false))]
    Apply {
        /// Blueprint YAML/JSON string or file path ('-' for stdin)
        blueprint: String,
    },
}

// Create all entities in a blueprint
fn create_from_blueprint(blueprint: Blueprint, config: Option<&str>) -> Result<(), CliError> {
    for secret in blueprint.secrets {
        println!("Creating secret: {}", type_colorize(&secret.name));
        secret::create(&serde_json::to_string(&secret).unwrap(), config, false)?;
    }
    Ok(())
}

// Update all entities in a blueprint
fn update_from_blueprint(blueprint: Blueprint, config: Option<&str>) -> Result<(), CliError> {
    for secret in blueprint.secrets {
        println!("Updating secret: {}", type_colorize(&secret.name));
        secret::update(
            &secret.name,
            &serde_json::to_string(&secret).unwrap(),
            config,
            false,
        )?;
    }
    Ok(())
}

// Apply the blueprint
fn apply(blueprint: String, config: Option<&str>) -> Result<(), CliError> {
    let blueprint_str = read_file_or_stdin(&blueprint)?;
    let apply_blueprint: Blueprint = serde_yaml::from_str(&blueprint_str)
        .map_err(|e| CliError::dataerr(format!("Could not parse blueprint file: {e}")))?;

    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let mut to_create = Blueprint::default();
    let mut to_update = Blueprint::default();

    for secret in apply_blueprint.secrets {
        // Check if secret exists
        let response = task::block_on(async {
            console::api::get_secret_by_id_or_name(&api_config, &secret.name).await
        });
        // Create secret if it does not exist and update if it does
        match response {
            Ok(_) => {
                to_update.secrets.push(secret.clone());
            }
            Err(_) => {
                to_create.secrets.push(secret.clone());
            }
        }
    }

    // Print a summary of the actions to be taken
    println!("{}", template_blueprint_summary(&to_create, &to_update));
    // Ask for confirmation
    if !confirm_action("blueprint", Some("apply")) {
        return Ok(());
    }

    if to_create != Blueprint::default() {
        println!("{}", "Creating entities...".bold());
        create_from_blueprint(to_create, config)?;
    } else {
        println!(
            "{} {}",
            "Creating entities:".bold(),
            "Nothing to create".green()
        );
    }
    if to_update != Blueprint::default() {
        println!("{}", "Updating entities...".bold());
        update_from_blueprint(to_update, config)?;
    } else {
        println!(
            "{} {}",
            "Updating entities:".bold(),
            "Nothing to update".green()
        );
    }

    Ok(())
}

// Parse the blueprint subcommand
pub(crate) fn parse(
    blueprint_command: BlueprintCommand,
    config: Option<&str>,
) -> Result<(), CliError> {
    match blueprint_command.command {
        BlueprintSubcommands::Apply { blueprint } => apply(blueprint, config)?,
    }
    Ok(())
}
