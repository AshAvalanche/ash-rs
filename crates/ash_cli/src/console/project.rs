// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the project subcommand parser

use crate::{
    console::{create_api_config_with_access_token, load_console},
    utils::{error::CliError, prompt::confirm_deletion, templating::*, version_tx_cmd},
};
use ash_sdk::console;
use async_std::task;
use clap::{Parser, Subcommand};
use colored::Colorize;

/// Interact with Ash Console projects
#[derive(Parser)]
#[command()]
pub(crate) struct ProjectCommand {
    #[command(subcommand)]
    command: ProjectSubcommands,
}

#[derive(Subcommand)]
enum ProjectSubcommands {
    /// List Ash Console projects
    #[command(version = version_tx_cmd(false))]
    List {
        /// Whether to show extended information (e.g. full IDs)
        #[arg(long, short = 'e')]
        extended: bool,
    },
    /// Create a new Ash Console project
    #[command(version = version_tx_cmd(false))]
    Create {
        /// Secret JSON string
        /// e.g.: '{"name": "My project", "network": "local"}'
        secret: String,
    },
    /// Get an Ash Console project
    #[command(version = version_tx_cmd(false))]
    Get {
        /// Secret ID
        secret_id: String,
        /// Whether to show extended information (e.g. full IDs)
        #[arg(long, short = 'e')]
        extended: bool,
    },
    /// Update an Ash Console project
    #[command(version = version_tx_cmd(false))]
    Update {
        /// Secret ID
        secret_id: String,
        /// Secret JSON string
        secret: String,
    },
    /// Delete an Ash Console project
    #[command(version = version_tx_cmd(false))]
    Delete {
        /// Secret ID
        secret_id: String,
        /// Assume yes to all prompts
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

// List projects
fn list(extended: bool, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let response = task::block_on(async { console::api::get_all_projects(&api_config).await })
        .map_err(|e| CliError::dataerr(format!("Error getting projects: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!("{}", template_projects_table(response, extended, 0));

    Ok(())
}

// Get a project by its ID
fn get(extended: bool, config: Option<&str>, project_id: &str, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let response =
        task::block_on(async { console::api::get_project_by_id(&api_config, project_id).await })
            .map_err(|e| CliError::dataerr(format!("Error getting secret: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!("{}", template_projects_table(vec![response], extended, 0));

    Ok(())
}

// Create a new project
fn create(project: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    // Deserialize the project JSON
    let new_project: console::api_models::NewProject = serde_json::from_str(project)
        .map_err(|e| CliError::dataerr(format!("Error parsing project JSON: {e}")))?;

    let response =
        task::block_on(async { console::api::create_project(&api_config, new_project).await })
            .map_err(|e| CliError::dataerr(format!("Error creating project: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!(
        "{}\n{}",
        "Project created successfully!".green(),
        template_projects_table(vec![response], false, 0)
    );

    Ok(())
}

// Update a project
fn update(
    project_id: &str,
    project: &str,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    // Deserialize the project JSON
    let update_project_request: console::api_models::UpdateProject = serde_json::from_str(project)
        .map_err(|e| CliError::dataerr(format!("Error parsing project JSON: {e}")))?;

    let response = task::block_on(async {
        console::api::update_project_by_id(&api_config, project_id, update_project_request).await
    })
    .map_err(|e| CliError::dataerr(format!("Error updating project: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!(
        "{}\n{}",
        "Project updated successfully!".green(),
        template_projects_table(vec![response], false, 0)
    );

    Ok(())
}

// Delete a project
fn delete(project_id: &str, yes: bool, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    // Prompt for confirmation if not using --yes
    if !yes {
        get(false, config, project_id, false)?;

        if !confirm_deletion("project") {
            return Ok(());
        }
    }

    let response =
        task::block_on(async { console::api::delete_project_by_id(&api_config, project_id).await })
            .map_err(|e| CliError::dataerr(format!("Error deleting project: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!("{}", "Project deleted successfully!".green());

    Ok(())
}

// Parse secret subcommand
pub(crate) fn parse(
    network: ProjectCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match network.command {
        ProjectSubcommands::List { extended } => list(extended, config, json),
        ProjectSubcommands::Get {
            secret_id,
            extended,
        } => get(extended, config, &secret_id, json),
        ProjectSubcommands::Create { secret } => create(&secret, config, json),
        ProjectSubcommands::Update { secret_id, secret } => {
            update(&secret_id, &secret, config, json)
        }
        ProjectSubcommands::Delete { secret_id, yes } => delete(&secret_id, yes, config, json),
    }
}
