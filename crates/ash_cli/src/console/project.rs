// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the project subcommand parser

use crate::{
    console::{create_api_config_with_access_token, load_console},
    utils::{
        error::CliError, prompt::confirm_deletion, state::CliState, templating::*, version_tx_cmd,
    },
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
    /// List Console projects
    #[command(version = version_tx_cmd(false))]
    List {
        /// Whether to show extended information (e.g. full IDs)
        #[arg(long, short = 'e')]
        extended: bool,
    },
    /// Create a new Console project
    #[command(version = version_tx_cmd(false))]
    Create {
        /// Project JSON string
        /// e.g.: '{"name": "My project", "network": "local"}'
        project: String,
    },
    /// Show Console project information
    #[command(version = version_tx_cmd(false))]
    Info {
        /// Project ID
        project_id: String,
        /// Whether to show extended information (e.g. full IDs)
        #[arg(long, short = 'e')]
        extended: bool,
    },
    /// Update a Console project
    #[command(version = version_tx_cmd(false))]
    Update {
        /// Project ID
        project_id: String,
        /// Project JSON string
        project: String,
    },
    /// Delete a Console project
    #[command(version = version_tx_cmd(false))]
    Delete {
        /// Project ID
        project_id: String,
        /// Assume yes to all prompts
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Show the current Console project
    #[command(version = version_tx_cmd(false))]
    Show,
    /// Select the current Console project
    /// This project will be used by default in other commands
    #[command(version = version_tx_cmd(false))]
    Select {
        /// Project ID
        project_id: String,
    },
}

// Get the current project ID
pub(crate) fn get_current_project_id() -> Result<String, CliError> {
    let state = CliState::load()?;

    if let Some(project_id) = state.current_project {
        Ok(project_id)
    } else {
        Err(CliError::dataerr(
            "No current project set. Use `ash project select` to set one.".to_string(),
        ))
    }
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
        template_projects_table(vec![response.clone()], false, 0)
    );

    // Set the new project as the current one
    let mut state = CliState::load()?;
    state.current_project = Some(response.id.unwrap_or_default().to_string());
    state.save()?;

    println!(
        "{}",
        format!(
            "Switched to project '{}' ({})!",
            response.name.unwrap_or_default(),
            response.id.unwrap_or_default().to_string()
        )
        .green()
    );

    Ok(())
}

// Get a project information by its ID
fn info(
    project_id: &str,
    extended: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let response =
        task::block_on(async { console::api::get_project_by_id(&api_config, project_id).await })
            .map_err(|e| CliError::dataerr(format!("Error getting project: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!("{}", template_projects_table(vec![response], extended, 0));

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
        info(project_id, false, config, false)?;

        if !confirm_deletion("project", None) {
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

// Show the current project
fn show(config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let state = CliState::load()?;

    if let Some(project_id) = state.current_project {
        let response = task::block_on(async {
            console::api::get_project_by_id(&api_config, &project_id).await
        });

        match response {
            Ok(current_project) => {
                if json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "currentProject": {
                                "id": current_project.id,
                                "name": current_project.name,
                            }
                        })
                    );
                    return Ok(());
                }

                println!(
                    "Current project: '{}' ({})",
                    type_colorize(&current_project.name.unwrap_or_default()),
                    type_colorize(&current_project.id.unwrap_or_default())
                );
            }
            Err(_) => {
                eprintln!(
                    "{}",
                    format!(
                        "The selected project '{}' does not exist anymore.",
                        project_id,
                    )
                    .red()
                );
                println!("Use `ash project select` to set a new one.");
            }
        }
    } else {
        if json {
            println!("{}", serde_json::json!({ "currentProject": null }));
            return Ok(());
        }

        println!("No current project set. Use `ash project select` to set one.");
    }

    Ok(())
}

// Select the current project
fn select(project_id: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let current_project =
        task::block_on(async { console::api::get_project_by_id(&api_config, project_id).await })
            .map_err(|e| CliError::dataerr(format!("Error getting project: {e}")))?;

    let mut state = CliState::load()?;
    state.current_project = Some(project_id.to_string());
    state.save()?;

    if json {
        println!(
            "{}",
            serde_json::json!({
                "currentProject": {
                    "id": current_project.id,
                    "name": current_project.name,
                }
            })
        );
        return Ok(());
    }

    println!(
        "{}",
        format!(
            "Switched to project '{}' ({})!",
            current_project.name.unwrap_or_default(),
            current_project.id.unwrap_or_default()
        )
        .green()
    );

    Ok(())
}

// Parse project subcommand
pub(crate) fn parse(
    project: ProjectCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match project.command {
        ProjectSubcommands::List { extended } => list(extended, config, json),
        ProjectSubcommands::Info {
            project_id,
            extended,
        } => info(&project_id, extended, config, json),
        ProjectSubcommands::Create { project } => create(&project, config, json),
        ProjectSubcommands::Update {
            project_id,
            project,
        } => update(&project_id, &project, config, json),
        ProjectSubcommands::Delete { project_id, yes } => delete(&project_id, yes, config, json),
        ProjectSubcommands::Show => show(config, json),
        ProjectSubcommands::Select { project_id } => select(&project_id, config, json),
    }
}
