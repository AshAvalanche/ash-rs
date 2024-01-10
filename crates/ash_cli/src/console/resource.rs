// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the resource subcommand parser

use crate::{
    console::{
        create_api_config_with_access_token, load_console, project::get_current_project_id_or_name,
    },
    utils::{
        error::CliError,
        file::*,
        prompt::{confirm_action, confirm_restart},
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
        /// Resource YAML/JSON string or file path ('-' for stdin)
        /// e.g.: '{name: my-node, resourceType: avalancheNode, cloudRegionId: region-id, ...}'
        resource: String,
    },
    /// Show information about a resource of the Console project
    #[command(version = version_tx_cmd(false))]
    Info {
        /// Resource ID or name
        resource_id_or_name: String,
        /// Whether to show extended information (e.g. full IDs)
        #[arg(long, short = 'e')]
        extended: bool,
    },
    /// Update a resource of the Console project
    #[command(version = version_tx_cmd(false))]
    Update {
        /// Resource ID or name
        resource_id_or_name: String,
        /// Resource YAML/JSON string or file path ('-' for stdin)
        /// e.g.: '{name: my-node, resourceType: avalancheNode, cloudRegionId: region-id, ...}'
        resource: String,
    },
    /// Delete a resource from the Console project
    #[command(version = version_tx_cmd(false))]
    Delete {
        /// Resource ID or name
        resource_id_or_name: String,
        /// Assume yes to all prompts
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Restart a resource of the Console project
    #[command(version = version_tx_cmd(false))]
    Restart {
        /// Resource ID or name
        resource_id_or_name: String,
        /// Assume yes to all prompts
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

// List resources of a project
fn list(
    project_id_or_name: &str,
    extended: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let resources_response = task::block_on(async {
        console::api::get_all_project_resources(&api_config, project_id_or_name).await
    })
    .map_err(|e| CliError::dataerr(format!("Error getting project resources: {e}")))?;

    let project_response = task::block_on(async {
        console::api::get_project_by_id_or_name(&api_config, project_id_or_name).await
    })
    .map_err(|e| CliError::dataerr(format!("Error getting project: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&resources_response));
        return Ok(());
    }

    println!(
        "Resources of project '{}':\n{}",
        type_colorize(&project_id_or_name),
        template_resources_table(resources_response, project_response, extended, 0)
    );

    Ok(())
}

// Create a resource in a project
pub(crate) fn create(
    project_id_or_name: &str,
    resource: &str,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let resource_str = read_file_or_stdin(resource)?;

    let spinner = spinner_with_message("Creating resource...".to_string());

    // Deserialize the resource JSON
    // TODO: Change to CreateResourceRequest when another resource type is added
    let new_resource: console::api_models::NewAvalancheNodeResource =
        serde_yaml::from_str(&resource_str)
            .map_err(|e| CliError::dataerr(format!("Error parsing resource JSON: {e}")))?;

    let resource_response = task::block_on(async {
        console::api::create_project_resource(&api_config, project_id_or_name, new_resource).await
    })
    .map_err(|e| CliError::dataerr(format!("Error creating resource in the project: {e}")))?;

    let project_response = task::block_on(async {
        console::api::get_project_by_id_or_name(&api_config, project_id_or_name).await
    })
    .map_err(|e| CliError::dataerr(format!("Error getting project: {e}")))?;

    spinner.finish_and_clear();

    if json {
        println!("{}", serde_json::json!(&resource_response));
        return Ok(());
    }

    println!(
        "{}\n{}",
        format!(
            "Resource successfully created in project '{}'!",
            project_id_or_name
        )
        .green(),
        template_resources_table(vec![resource_response], project_response, false, 0)
    );

    Ok(())
}

// Get a project resource information by its ID
fn info(
    project_id_or_name: &str,
    resource_id_or_name: &str,
    extended: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let resource_response = task::block_on(async {
        console::api::get_project_resource_by_id_or_name(
            &api_config,
            project_id_or_name,
            resource_id_or_name,
        )
        .await
    })
    .map_err(|e| CliError::dataerr(format!("Error getting resource: {e}")))?;

    let project_response = task::block_on(async {
        console::api::get_project_by_id_or_name(&api_config, project_id_or_name).await
    })
    .map_err(|e| CliError::dataerr(format!("Error getting project: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&resource_response));
        return Ok(());
    }

    println!(
        "Resource '{}' of project '{}':\n{}",
        type_colorize(&resource_id_or_name),
        type_colorize(&project_id_or_name),
        template_resources_table(vec![resource_response], project_response, extended, 0)
    );

    Ok(())
}

// Update a resource
pub(crate) fn update(
    project_id_or_name: &str,
    resource_id_or_name: &str,
    resource: &str,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let resource_str = read_file_or_stdin(resource)?;

    let spinner = spinner_with_message("Updating resource...".to_string());

    // Deserialize the resource JSON
    // TODO: Change to UpdateResourceByIdRequest when another resource type is added
    let update_resource_request: console::api_models::UpdateAvalancheNodeResource =
        serde_yaml::from_str(&resource_str)
            .map_err(|e| CliError::dataerr(format!("Error parsing resource JSON: {e}")))?;

    let resource_response = task::block_on(async {
        console::api::update_project_resource_by_id_or_name(
            &api_config,
            project_id_or_name,
            resource_id_or_name,
            update_resource_request,
        )
        .await
    })
    .map_err(|e| CliError::dataerr(format!("Error updating resource: {e}")))?;

    let project_response = task::block_on(async {
        console::api::get_project_by_id_or_name(&api_config, project_id_or_name).await
    })
    .map_err(|e| CliError::dataerr(format!("Error getting project: {e}")))?;

    spinner.finish_and_clear();

    if json {
        println!("{}", serde_json::json!(&resource_response));
        return Ok(());
    }

    println!(
        "{}\n{}",
        "Resource updated successfully!".green(),
        template_resources_table(vec![resource_response], project_response, false, 0)
    );

    Ok(())
}

// Delete a resource from a project
fn delete(
    project_id_or_name: &str,
    resource_id_or_name: &str,
    yes: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    // Prompt for confirmation if not using --yes
    if !yes {
        info(
            project_id_or_name,
            resource_id_or_name,
            false,
            config,
            false,
        )?;

        if !confirm_action("resource", None) {
            return Ok(());
        }
    }

    let spinner = spinner_with_message("Deleting resource...".to_string());

    let response = task::block_on(async {
        console::api::delete_project_resource_by_id_or_name(
            &api_config,
            project_id_or_name,
            resource_id_or_name,
        )
        .await
    })
    .map_err(|e| CliError::dataerr(format!("Error removing resource: {e}")))?;

    spinner.finish_and_clear();

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!("{}", "Resource deleted successfully!".green());

    Ok(())
}

// Restart a resource
fn restart(
    project_id_or_name: &str,
    resource_id_or_name: &str,
    yes: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    // Prompt for confirmation if not using --yes
    if !yes {
        info(
            project_id_or_name,
            resource_id_or_name,
            false,
            config,
            false,
        )?;

        if !confirm_restart("resource") {
            return Ok(());
        }
    }

    let spinner = spinner_with_message("Restarting resource...".to_string());

    let response = task::block_on(async {
        console::api::restart_project_resource_by_id_or_name(
            &api_config,
            project_id_or_name,
            resource_id_or_name,
        )
        .await
    })
    .map_err(|e| CliError::dataerr(format!("Error restarting resource: {e}")))?;

    spinner.finish_and_clear();

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
    let mut project_id_or_name = resource.project_id_or_name;

    // Get the current project ID
    if project_id_or_name == "current" {
        project_id_or_name = get_current_project_id_or_name()?;
    };

    match resource.command {
        ResourceSubcommands::List { extended } => list(&project_id_or_name, extended, config, json),
        ResourceSubcommands::Create { resource } => {
            create(&project_id_or_name, &resource, config, json)
        }
        ResourceSubcommands::Info {
            resource_id_or_name,
            extended,
        } => info(
            &project_id_or_name,
            &resource_id_or_name,
            extended,
            config,
            json,
        ),
        ResourceSubcommands::Update {
            resource_id_or_name,
            resource,
        } => update(
            &project_id_or_name,
            &resource_id_or_name,
            &resource,
            config,
            json,
        ),
        ResourceSubcommands::Delete {
            resource_id_or_name,
            yes,
        } => delete(&project_id_or_name, &resource_id_or_name, yes, config, json),
        ResourceSubcommands::Restart {
            resource_id_or_name,
            yes,
        } => restart(&project_id_or_name, &resource_id_or_name, yes, config, json),
    }
}
