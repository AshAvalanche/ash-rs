// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the region subcommand parser

use crate::{
    console::project::get_current_project_id_or_name,
    console::{create_api_config_with_access_token, load_console},
    utils::{error::CliError, file::*, prompt::confirm_action, templating::*, version_tx_cmd},
};
use ash_sdk::console;
use async_std::task;
use clap::{Parser, Subcommand};
use colored::Colorize;

/// Interact with Ash Console projects' cloud regions
#[derive(Parser)]
#[command()]
pub(crate) struct RegionCommand {
    #[command(subcommand)]
    command: RegionSubcommands,
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
enum RegionSubcommands {
    /// Show the list of available regions for each cloud provider
    #[command(version = version_tx_cmd(false))]
    Available,
    /// List the cloud regions of the Console project
    #[command(version = version_tx_cmd(false))]
    List {
        /// Whether to show extended information (e.g. full IDs)
        #[arg(long, short = 'e')]
        extended: bool,
    },
    /// Add a cloud region to the Console project
    #[command(version = version_tx_cmd(false))]
    Add {
        /// Cloud region YAML/JSON string or file path ('-' for stdin)
        /// e.g.: '{cloudProvider: aws, region: us-east-1, cloudCredentialsSecretId: secret-id}'
        region: String,
    },
    /// Show information about a cloud region of the Console project
    #[command(version = version_tx_cmd(false))]
    Info {
        /// Region name
        /// e.g.: "aws/us-east-1"
        region_name: String,
        /// Whether to show extended information (e.g. full IDs)
        #[arg(long, short = 'e')]
        extended: bool,
    },
    /// Remove a cloud region from the Console project
    #[command(version = version_tx_cmd(false))]
    Remove {
        /// Region name
        /// e.g.: "aws/us-east-1"
        region_name: String,
        /// Assume yes to all prompts
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

// List available cloud regions of a provider
fn available(config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let response =
        task::block_on(async { console::api::get_available_cloud_regions(&api_config).await })
            .map_err(|e| {
                CliError::dataerr(format!("Error getting available cloud regions: {e}"))
            })?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!(
        "Available cloud regions:\n{}",
        template_available_regions_table(response, 0)
    );

    Ok(())
}

// List cloud regions of a project
fn list(
    project_id_or_name: &str,
    extended: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let response = task::block_on(async {
        console::api::get_all_project_cloud_regions(&api_config, project_id_or_name).await
    })
    .map_err(|e| CliError::dataerr(format!("Error getting project cloud regions: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!(
        "Cloud regions of project '{}':\n{}",
        type_colorize(&project_id_or_name),
        template_regions_table(response, extended, 0)
    );

    Ok(())
}

// Add a cloud region to a project
fn add(
    project_id_or_name: &str,
    region: &str,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let region_str = read_file_or_stdin(region)?;

    // Deserialize the region JSON
    let new_region: console::api_models::NewCloudRegion = serde_yaml::from_str(&region_str)
        .map_err(|e| CliError::dataerr(format!("Error parsing cloud region JSON: {e}")))?;

    let response = task::block_on(async {
        console::api::add_project_cloud_region(&api_config, project_id_or_name, new_region).await
    })
    .map_err(|e| CliError::dataerr(format!("Error adding cloud region to the project: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!(
        "{}\n{}",
        format!(
            "Cloud region successfully added to project '{}'!",
            project_id_or_name
        )
        .green(),
        template_regions_table(vec![response], false, 0)
    );

    Ok(())
}

// Get a project cloud region information by its ID
fn info(
    project_id_or_name: &str,
    region_name: &str,
    extended: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let response = task::block_on(async {
        console::api::get_project_cloud_region_by_name(
            &api_config,
            project_id_or_name,
            &region_name.replace('/', "_"),
        )
        .await
    })
    .map_err(|e| CliError::dataerr(format!("Error getting cloud region: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!(
        "Region '{}' of project '{}':\n{}",
        type_colorize(&region_name),
        type_colorize(&project_id_or_name),
        template_regions_table(vec![response], extended, 0)
    );

    Ok(())
}

// Remove a cloud region from a project
fn remove(
    project_id_or_name: &str,
    region_name: &str,
    yes: bool,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    // Prompt for confirmation if not using --yes
    if !yes {
        info(project_id_or_name, region_name, false, config, false)?;

        if !confirm_action("region", Some("remove")) {
            return Ok(());
        }
    }

    let response = task::block_on(async {
        console::api::remove_project_cloud_region_by_name(
            &api_config,
            project_id_or_name,
            &region_name.replace('/', "_"),
        )
        .await
    })
    .map_err(|e| CliError::dataerr(format!("Error removing cloud region: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!("{}", "Cloud region removed successfully!".green());

    Ok(())
}

// Parse region subcommand
pub(crate) fn parse(
    region: RegionCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    let mut project_id_or_name = region.project_id_or_name;

    // Get the current project ID for the subcommands that require it
    match region.command {
        RegionSubcommands::Available {} => (),
        _ => {
            if project_id_or_name == "current" {
                project_id_or_name = get_current_project_id_or_name()?;
            }
        }
    }

    match region.command {
        RegionSubcommands::Available => available(config, json),
        RegionSubcommands::List { extended } => list(&project_id_or_name, extended, config, json),
        RegionSubcommands::Add { region } => add(&project_id_or_name, &region, config, json),
        RegionSubcommands::Info {
            region_name,
            extended,
        } => info(&project_id_or_name, &region_name, extended, config, json),
        RegionSubcommands::Remove { region_name, yes } => {
            remove(&project_id_or_name, &region_name, yes, config, json)
        }
    }
}
