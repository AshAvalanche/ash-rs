// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the resource subcommand parser

use crate::{
    console::project::get_current_project_id,
    console::{create_api_config_with_access_token, load_console},
    utils::{
        error::CliError,
        prompt::{confirm_deletion, confirm_restart},
        templating::*,
        version_tx_cmd,
    },
};
use ash_sdk::console;
use async_std::task;
use clap::{Parser, Subcommand};
use colored::Colorize;

/// Interact with Ash Console projects' resources
#[derive(Parser)]
#[command()]
pub(crate) struct ResourceCommand {
    #[command(subcommand)]
    command: ResourceSubcommands,
    /// Console project ID.
    /// Defaults to the current project
    #[arg(
        long,
        short = 'p',
        default_value = "current",
        global = true,
        env = "ASH_CONSOLE_PROJECT"
    )]
    project: String,
}

#[derive(Subcommand)]
enum ResourceSubcommands {
    /// List the resources of the Console project
    #[command(version = version_tx_cmd(false))]
    List {
        /// Whether to show extended information (e.g. full IDs)
        #[arg(long, short = 'e')]
        extended: bool,
    },
    /// Create a resource in the Console project
    #[command(version = version_tx_cmd(false))]
    Create {
        /// Resource JSON string
        /// e.g.: '{"name": "my-node", "resourceType": "avalancheNode", "cloudRegionId": "region-id", ...}'
        resource: String,
    },
    /// Show information about a resource of the Console project
    #[command(version = version_tx_cmd(false))]
    Info {
        /// Resource ID
        resource_id: String,
        /// Whether to show extended information (e.g. full IDs)
        #[arg(long, short = 'e')]
        extended: bool,
    },
    /// Update a resource of the Console project
    #[command(version = version_tx_cmd(false))]
    Update {
        /// Resource ID
        resource_id: String,
        /// Resource JSON string
        /// e.g.: '{"name": "my-node", "resourceType": "avalancheNode", "cloudRegionId": "region-id", ...}'
        resource: String,
    },
    /// Delete a resource from the Console project
    #[command(version = version_tx_cmd(false))]
    Delete {
        /// Resource ID
        resource_id: String,
        /// Assume yes to all prompts
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Restart a resource of the Console project
    #[command(version = version_tx_cmd(false))]
    Restart {
        /// Resource ID
        resource_id: String,
        /// Assume yes to all prompts
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

// List resources of a project
fn list(
    project_id: &str,
    extended: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let response = task::block_on(async {
        console::api::get_all_project_resources(&api_config, project_id).await
    })
    .map_err(|e| CliError::dataerr(format!("Error getting project resources: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!(
        "Resources of project '{}':\n{}",
        type_colorize(&project_id),
        template_resources_table(response, extended, 0)
    );

    Ok(())
}

// Create a resource in a project
fn create(
    project_id: &str,
    resource: &str,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    // Deserialize the resource JSON
    // TODO: Change to CreateResourceRequest when another resource type is added
    let new_resource: console::api_models::NewAvalancheNodeResource =
        serde_json::from_str(resource)
            .map_err(|e| CliError::dataerr(format!("Error parsing resource JSON: {e}")))?;

    let response = task::block_on(async {
        console::api::create_project_resource(&api_config, project_id, new_resource).await
    })
    .map_err(|e| CliError::dataerr(format!("Error creating resource in the project: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!(
        "{}\n{}",
        format!("Resource successfully created in project '{}'!", project_id).green(),
        template_resources_table(vec![response], false, 0)
    );

    Ok(())
}

// Get a project resource information by its ID
fn info(
    project_id: &str,
    resource_id: &str,
    extended: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let response = task::block_on(async {
        console::api::get_project_resource_by_id(&api_config, project_id, resource_id).await
    })
    .map_err(|e| CliError::dataerr(format!("Error getting resource: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!(
        "Resource '{}' of project '{}':\n{}",
        type_colorize(&resource_id),
        type_colorize(&project_id),
        template_resources_table(vec![response], extended, 0)
    );

    Ok(())
}

// Update a resource
fn update(
    project_id: &str,
    resource_id: &str,
    resource: &str,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    // Deserialize the resource JSON
    // TODO: Change to UpdateResourceByIdRequest when another resource type is added
    let update_resource_request: console::api_models::UpdateAvalancheNodeResource =
        serde_json::from_str(resource)
            .map_err(|e| CliError::dataerr(format!("Error parsing resource JSON: {e}")))?;

    let response = task::block_on(async {
        console::api::update_project_resource_by_id(
            &api_config,
            project_id,
            resource_id,
            update_resource_request,
        )
        .await
    })
    .map_err(|e| CliError::dataerr(format!("Error updating resource: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!(
        "{}\n{}",
        "Resource updated successfully!".green(),
        template_resources_table(vec![response], false, 0)
    );

    Ok(())
}

// Delete a resource from a project
fn delete(
    project_id: &str,
    resource_id: &str,
    yes: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    // Prompt for confirmation if not using --yes
    if !yes {
        info(project_id, resource_id, false, config, false)?;

        if !confirm_deletion("resource", None) {
            return Ok(());
        }
    }

    let response = task::block_on(async {
        console::api::delete_project_resource_by_id(&api_config, project_id, resource_id).await
    })
    .map_err(|e| CliError::dataerr(format!("Error removing resource: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!("{}", "Resource deleted successfully!".green());

    Ok(())
}

// Restart a resource
fn restart(
    project_id: &str,
    resource_id: &str,
    yes: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    // Prompt for confirmation if not using --yes
    if !yes {
        info(project_id, resource_id, false, config, false)?;

        if !confirm_restart("resource") {
            return Ok(());
        }
    }

    let response = task::block_on(async {
        console::api::restart_project_resource_by_id(&api_config, project_id, resource_id).await
    })
    .map_err(|e| CliError::dataerr(format!("Error restarting resource: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!("{}", "Resource restarted successfully!".green());

    Ok(())
}

// Parse resource subcommand
pub(crate) fn parse(
    resource: ResourceCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut project_id = resource.project;

    // Get the current project ID for the subcommands that require it
    match resource.command {
        _ => {
            if project_id == "current" {
                project_id = get_current_project_id()?;
            }
        }
    }

    match resource.command {
        ResourceSubcommands::List { extended } => list(&project_id, extended, config, json),
        ResourceSubcommands::Create { resource } => create(&project_id, &resource, config, json),
        ResourceSubcommands::Info {
            resource_id,
            extended,
        } => info(&project_id, &resource_id, extended, config, json),
        ResourceSubcommands::Update {
            resource_id,
            resource,
        } => update(&project_id, &resource_id, &resource, config, json),
        ResourceSubcommands::Delete { resource_id, yes } => {
            delete(&project_id, &resource_id, yes, config, json)
        }
        ResourceSubcommands::Restart { resource_id, yes } => {
            restart(&project_id, &resource_id, yes, config, json)
        }
    }
}
