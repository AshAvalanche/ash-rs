// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the secret subcommand parser

use crate::{
    console::{create_api_config_with_access_token, load_console},
    utils::{error::CliError, templating::*, version_tx_cmd},
};
use ash_sdk::console;
use async_std::task;
use clap::{Parser, Subcommand};
use colored::Colorize;

/// Interact with Ash Console secrets
#[derive(Parser)]
#[command()]
pub(crate) struct SecretCommand {
    #[command(subcommand)]
    command: SecretSubcommands,
}

#[derive(Subcommand)]
enum SecretSubcommands {
    /// List Ash Console secrets
    #[command(version = version_tx_cmd(false))]
    List {
        /// Whether to show extended information (e.g. full IDs)
        #[arg(long, short = 'e')]
        extended: bool,
    },
    /// Create a new Ash Console secret
    #[command(version = version_tx_cmd(false))]
    Create {
        /// Secret JSON string
        /// e.g.: '{"name": "My secret", "secretType": "generic", "content": "MyS3cr3tC0nt3nt"}'
        secret: String,
    },
    /// Get an Ash Console secret
    #[command(version = version_tx_cmd(false))]
    Get {
        /// Secret ID
        secret_id: String,
        /// Whether to show extended information (e.g. full IDs)
        #[arg(long, short = 'e')]
        extended: bool,
    },
    /// Update an Ash Console secret
    #[command(version = version_tx_cmd(false))]
    Update {
        /// Secret ID
        secret_id: String,
        /// Secret JSON string
        secret: String,
    },
    /// Delete an Ash Console secret
    #[command(version = version_tx_cmd(false))]
    Delete {
        /// Secret ID
        secret_id: String,
    },
}

fn get_secret_response_to_secret(
    get_all_secrets_response: &console::api_models::GetAllSecrets200ResponseInner,
) -> console::api_models::Secret {
    serde_json::from_value::<console::api_models::Secret>(serde_json::json!(
        get_all_secrets_response
    ))
    .unwrap()
}

// List secrets
fn list(extended: bool, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let response = task::block_on(async { console::api::get_all_secrets(&api_config).await })
        .map_err(|e| CliError::dataerr(format!("Error getting secrets: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    let secrets = response.iter().map(get_secret_response_to_secret).collect();

    println!("{}", template_secrets_table(secrets, extended, 0));

    Ok(())
}

// Get a secret by its ID
fn get(extended: bool, config: Option<&str>, secret_id: &str, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let response =
        task::block_on(async { console::api::get_secret_by_id(&api_config, secret_id).await })
            .map_err(|e| CliError::dataerr(format!("Error getting secret: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!(
        "{}",
        template_secrets_table(vec![get_secret_response_to_secret(&response)], extended, 0)
    );

    Ok(())
}

// Create a new secret
fn create(secret: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    // Deserialize the secret JSON
    let create_secret_request: console::api_models::CreateSecretRequest =
        serde_json::from_str(secret)
            .map_err(|e| CliError::dataerr(format!("Error parsing secret JSON: {e}")))?;

    let response = task::block_on(async {
        console::api::create_secret(&api_config, create_secret_request).await
    })
    .map_err(|e| CliError::dataerr(format!("Error creating secret: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!(
        "{}\n{}",
        "Secret created successfully!".green(),
        template_secrets_table(vec![get_secret_response_to_secret(&response)], false, 0)
    );

    Ok(())
}

// Update a secret
fn update(secret_id: &str, secret: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    // Deserialize the secret JSON
    let update_secret_request: console::api_models::UpdateSecretByIdRequest =
        serde_json::from_str(secret)
            .map_err(|e| CliError::dataerr(format!("Error parsing secret JSON: {e}")))?;

    let response = task::block_on(async {
        console::api::update_secret_by_id(&api_config, secret_id, update_secret_request).await
    })
    .map_err(|e| CliError::dataerr(format!("Error updating secret: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!(
        "{}\n{}",
        "Secret updated successfully!".green(),
        template_secrets_table(vec![get_secret_response_to_secret(&response)], false, 0)
    );

    Ok(())
}

// Delete a secret
fn delete(secret_id: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    // TODO: Add confirmation prompt when inquire used for interactive mode

    let response =
        task::block_on(async { console::api::delete_secret_by_id(&api_config, secret_id).await })
            .map_err(|e| CliError::dataerr(format!("Error deleting secret: {e}")))?;

    if json {
        println!("{}", serde_json::json!(&response));
        return Ok(());
    }

    println!("{}", "Secret deleted successfully!".green());

    Ok(())
}

// Parse secret subcommand
pub(crate) fn parse(
    network: SecretCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match network.command {
        SecretSubcommands::List { extended } => list(extended, config, json),
        SecretSubcommands::Get {
            secret_id,
            extended,
        } => get(extended, config, &secret_id, json),
        SecretSubcommands::Create { secret } => create(&secret, config, json),
        SecretSubcommands::Update { secret_id, secret } => {
            update(&secret_id, &secret, config, json)
        }
        SecretSubcommands::Delete { secret_id } => delete(&secret_id, config, json),
    }
}
