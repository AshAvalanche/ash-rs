// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the blueprint subcommand parser

use crate::{
    console::{
        create_api_config_with_access_token, load_console, project, region, resource, secret,
    },
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
/// Allows to manage multiple entities at once, e.g. a project with a region and a resource
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Blueprint {
    #[serde(default)]
    pub secrets: Vec<console::api_models::CreateSecretRequest>,
    #[serde(default)]
    pub projects: Vec<BlueprintProject>,
}

/// Blueprint project object
/// Allows to manage a project with its regions and resources
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BlueprintProject {
    pub project: console::api_models::NewProject,
    #[serde(default)]
    pub regions: Vec<console::api_models::NewCloudRegion>,
    #[serde(default)]
    pub resources: Vec<console::api_models::NewAvalancheNodeResource>,
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
        /// Assume yes to all prompts
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

// Add regions to a project (do nothing if the region already exists)
fn add_project_regions(
    project_name: &str,
    regions: Vec<console::api_models::NewCloudRegion>,
    config: Option<&str>,
    api_config: &console::api_config::Configuration,
) -> Result<(), CliError> {
    for region in regions {
        let region_name = format!(
            "{}/{}",
            serde_json::to_value(region.cloud_provider.unwrap_or_default())
                .unwrap()
                .as_str()
                .unwrap(),
            region.region.clone().unwrap_or_default()
        );
        let response = task::block_on(async {
            console::api::get_project_cloud_region_by_name(
                api_config,
                project_name,
                &region_name.replace("/", "_"),
            )
            .await
        });
        match response {
            Ok(_) => {
                println!(
                    "Region already exists: {}",
                    type_colorize(&format!("{}:{}", project_name, region_name))
                )
            }
            Err(_) => {
                println!(
                    "Adding region: {}",
                    type_colorize(&format!("{}:{}", project_name, region_name))
                );
                region::add(
                    project_name,
                    &serde_json::to_string(&region).unwrap(),
                    config,
                    false,
                )?;
            }
        }
    }
    Ok(())
}

// Add or update resources to a project
fn add_update_project_resources(
    project_name: &str,
    resources: Vec<console::api_models::NewAvalancheNodeResource>,
    config: Option<&str>,
    api_config: &console::api_config::Configuration,
) -> Result<(), CliError> {
    for resource in resources {
        let response = task::block_on(async {
            console::api::get_project_resource_by_id_or_name(
                api_config,
                project_name,
                &resource.name,
            )
            .await
        });
        match response {
            Ok(_) => {
                println!(
                    "Updating resource: {}",
                    type_colorize(&format!("{}:{}", project_name, resource.name))
                );
                resource::update(
                    project_name,
                    &resource.name,
                    &serde_json::to_string(&resource).unwrap(),
                    config,
                    false,
                )?;
            }
            Err(_) => {
                println!(
                    "Adding resource: {}",
                    type_colorize(&format!("{}:{}", project_name, resource.name))
                );
                resource::create(
                    project_name,
                    &serde_json::to_string(&resource).unwrap(),
                    config,
                    false,
                )?;
            }
        }
    }
    Ok(())
}

// Create all entities in a blueprint
fn create_from_blueprint(
    blueprint: Blueprint,
    config: Option<&str>,
    api_config: &console::api_config::Configuration,
) -> Result<(), CliError> {
    for secret in blueprint.secrets {
        println!("Creating secret: {}", type_colorize(&secret.name));
        secret::create(&serde_json::to_string(&secret).unwrap(), config, false)?;
    }
    for project in blueprint.projects {
        println!("Creating project: {}", type_colorize(&project.project.name));
        project::create(
            &serde_json::to_string(&project.project).unwrap(),
            config,
            false,
        )?;
        add_project_regions(&project.project.name, project.regions, config, api_config)?;
        add_update_project_resources(&project.project.name, project.resources, config, api_config)?;
    }
    Ok(())
}

// Update all entities in a blueprint
fn update_from_blueprint(
    blueprint: Blueprint,
    config: Option<&str>,
    api_config: &console::api_config::Configuration,
) -> Result<(), CliError> {
    for secret in blueprint.secrets {
        println!("Updating secret: {}", type_colorize(&secret.name));
        secret::update(
            &secret.name,
            &serde_json::to_string(&secret).unwrap(),
            config,
            false,
        )?;
    }
    for project in blueprint.projects {
        println!("Updating project: {}", type_colorize(&project.project.name));
        project::update(
            &project.project.name,
            &serde_json::to_string(&project.project).unwrap(),
            config,
            false,
        )?;
        add_project_regions(&project.project.name, project.regions, config, api_config)?;
        add_update_project_resources(&project.project.name, project.resources, config, api_config)?;
    }
    Ok(())
}

// Apply the blueprint
fn apply(blueprint: String, yes: bool, config: Option<&str>) -> Result<(), CliError> {
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
    for project in apply_blueprint.projects {
        // Check if project exists
        let response = task::block_on(async {
            console::api::get_project_by_id_or_name(&api_config, &project.project.name).await
        });
        // Create project if it does not exist and update if it does
        match response {
            Ok(_) => {
                to_update.projects.push(project.clone());
            }
            Err(_) => {
                to_create.projects.push(project.clone());
            }
        }
    }

    // Print a summary of the actions to be taken
    println!("{}", template_blueprint_summary(&to_create, &to_update));
    // Ask for confirmation
    if !yes {
        if !confirm_action("blueprint", Some("apply")) {
            return Ok(());
        }
    }

    if to_create != Blueprint::default() {
        println!("{}", "Creating entities...".bold());
        create_from_blueprint(to_create, config, &api_config)?;
    } else {
        println!(
            "{} {}",
            "Creating entities:".bold(),
            "Nothing to create".green()
        );
    }
    if to_update != Blueprint::default() {
        println!("{}", "Updating entities...".bold());
        update_from_blueprint(to_update, config, &api_config)?;
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
        BlueprintSubcommands::Apply { blueprint, yes } => apply(blueprint, yes, config)?,
    }
    Ok(())
}
