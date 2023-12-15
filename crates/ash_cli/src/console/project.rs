// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the project subcommand parser

use crate::{
    console::{create_api_config_with_access_token, load_console},
    utils::{
        error::CliError, file::*, prompt::confirm_action, state::CliState, templating::*,
        version_tx_cmd,
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
        /// Project YAML/JSON string or file path ('-' for stdin)
        /// e.g.: '{name: my-project, network: local}'
        project: String,
    },
    /// Show Console project information
    #[command(version = version_tx_cmd(false))]
    Info {
        /// Project ID or name
        project_id_or_name: String,
        /// Whether to show extended information (e.g. full IDs)
        #[arg(long, short = 'e')]
        extended: bool,
    },
    /// Update a Console project
    #[command(version = version_tx_cmd(false))]
    Update {
        /// Project ID or name
        project_id_or_name: String,
        /// Project YAML/JSON string or file path ('-' for stdin)
        project: String,
    },
    /// Delete a Console project
    #[command(version = version_tx_cmd(false))]
    Delete {
        /// Project ID or name
        project_id_or_name: String,
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
        /// Project ID or name
        project_id_or_name: String,
    },
}

// Get the current project ID or name
pub(crate) fn get_current_project_id_or_name() -> Result<String, CliError> {
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
pub(crate) fn create(project: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let project_str = read_file_or_stdin(project)?;

    // Deserialize the project JSON
    let new_project: console::api_models::NewProject = serde_yaml::from_str(&project_str)
        .map_err(|e| CliError::dataerr(format!("Error parsing project JSON: {e}")))?;

    let response =
        task::block_on(async { console::api::create_project(&api_config, new_project).await })
            .map_err(|e| CliError::dataerr(format!("Error creating project: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
    } else {
        println!(
            "{}\n{}",
            "Project created successfully!".green(),
            template_projects_table(vec![response.clone()], false, 0)
        );
    }

    // Set the new project as the current one
    let mut state = CliState::load()?;
    state.current_project = Some(response.name.clone().unwrap_or_default());
    state.save()?;

    let switch_message = format!(
        "Switched to project '{}' ({})!",
        response.name.unwrap_or_default(),
        response.id.unwrap_or_default()
    )
    .green();

    if json {
        eprint!("{}", switch_message);
    } else {
        println!("{}", switch_message);
    }

    Ok(())
}

// Get a project information by its ID
fn info(
    project_id_or_name: &str,
    extended: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let response = task::block_on(async {
        console::api::get_project_by_id_or_name(&api_config, project_id_or_name).await
    })
    .map_err(|e| CliError::dataerr(format!("Error getting project: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!("{}", template_projects_table(vec![response], extended, 0));

    Ok(())
}

// Update a project
pub(crate) fn update(
    project_id_or_name: &str,
    project: &str,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let project_str = read_file_or_stdin(project)?;

    // Deserialize the project JSON
    let update_project_request: console::api_models::UpdateProject =
        serde_yaml::from_str(&project_str)
            .map_err(|e| CliError::dataerr(format!("Error parsing project JSON: {e}")))?;

    let response = task::block_on(async {
        console::api::update_project_by_id_or_name(
            &api_config,
            project_id_or_name,
            update_project_request,
        )
        .await
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
fn delete(
    project_id_or_name: &str,
    yes: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    // Prompt for confirmation if not using --yes
    if !yes {
        info(project_id_or_name, false, config, false)?;

        if !confirm_action("project", None) {
            return Ok(());
        }
    }

    let response = task::block_on(async {
        console::api::delete_project_by_id_or_name(&api_config, project_id_or_name).await
    })
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
            console::api::get_project_by_id_or_name(&api_config, &project_id).await
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
fn select(project_id_or_name: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let current_project = task::block_on(async {
        console::api::get_project_by_id_or_name(&api_config, project_id_or_name).await
    })
    .map_err(|e| CliError::dataerr(format!("Error getting project: {e}")))?;

    let mut state = CliState::load()?;
    state.current_project = Some(project_id_or_name.to_string());
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
            project_id_or_name,
            extended,
        } => info(&project_id_or_name, extended, config, json),
        ProjectSubcommands::Create { project } => create(&project, config, json),
        ProjectSubcommands::Update {
            project_id_or_name,
            project,
        } => update(&project_id_or_name, &project, config, json),
        ProjectSubcommands::Delete {
            project_id_or_name,
            yes,
        } => delete(&project_id_or_name, yes, config, json),
        ProjectSubcommands::Show => show(config, json),
        ProjectSubcommands::Select { project_id_or_name } => {
            select(&project_id_or_name, config, json)
        }
    }
}
