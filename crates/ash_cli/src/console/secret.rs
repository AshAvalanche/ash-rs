// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the secret subcommand parser

use crate::{
    console::{create_api_config_with_access_token, load_console},
    utils::{error::CliError, file::*, prompt::confirm_deletion, templating::*, version_tx_cmd},
};
use ash_sdk::console;
use async_std::task;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

/// Interact with Ash Console secrets
#[derive(Parser)]
#[command()]
pub(crate) struct SecretCommand {
    #[command(subcommand)]
    command: SecretSubcommands,
}

#[derive(Subcommand)]
enum SecretSubcommands {
    /// List Console secrets
    #[command(version = version_tx_cmd(false))]
    List {
        /// Whether to show extended information (e.g. full IDs)
        #[arg(long, short = 'e')]
        extended: bool,
    },
    /// Create a new Console secret
    #[command(version = version_tx_cmd(false))]
    Create {
        /// Secret YAML/JSON string or file path ('-' for stdin)
        /// e.g.: '{name: my-secret, secretType: generic, content: "MyS3cr3tC0nt3nt"}'
        secret: String,
    },
    /// Show Console secret information
    #[command(version = version_tx_cmd(false))]
    Info {
        /// Secret ID
        secret_id: String,
        /// Whether to show extended information (e.g. full IDs)
        #[arg(long, short = 'e')]
        extended: bool,
    },
    /// Update a Console secret
    #[command(version = version_tx_cmd(false))]
    Update {
        /// Secret ID
        secret_id: String,
        /// Secret YAML/JSON string or file path ('-' for stdin)
        secret: String,
    },
    /// Delete a Console secret
    #[command(version = version_tx_cmd(false))]
    Delete {
        /// Secret ID
        secret_id: String,
        /// Assume yes to all prompts
        #[arg(long, short = 'y')]
        yes: bool,
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

// For a given nodeId secret, load the cert and key files if their values are paths
// Both cert and key must be either paths to PEM files or Base64-encoded strings
fn load_node_id_tls_cert_key(
    node_id_secret: &mut console::api_models::CreateSecretRequest,
) -> Result<(), CliError> {
    let node_cert = match node_id_secret.node_cert {
        Some(ref cert) => cert,
        None => {
            return Err(CliError::dataerr(
                "Error parsing node ID secret JSON: nodeCert field is missing".to_string(),
            ))
        }
    };
    let node_key = match node_id_secret.node_key {
        Some(ref key) => key,
        None => {
            return Err(CliError::dataerr(
                "Error parsing node ID secret JSON: nodeKey field is missing".to_string(),
            ))
        }
    };

    let cert_path = PathBuf::from(&shellexpand::tilde(node_cert).to_string());
    let key_path = PathBuf::from(&shellexpand::tilde(node_key).to_string());

    node_id_secret.node_cert = if cert_path.exists() {
        Some(read_file_base64(cert_path)?)
    } else {
        Some(node_cert.clone())
    };

    node_id_secret.node_key = if key_path.exists() {
        Some(read_file_base64(key_path)?)
    } else {
        Some(node_key.clone())
    };

    Ok(())
}

// For a given googleCredentials secret, load the private key file if its value is a path
// The private key must be either a path to a PEM file or a Base64-encoded string
fn load_google_credentials_private_key(
    google_credentials_secret: &mut console::api_models::CreateSecretRequest,
) -> Result<(), CliError> {
    let private_key = match google_credentials_secret.private_key {
        Some(ref key) => key,
        None => {
            return Err(CliError::dataerr(
                "Error parsing googleCredentials secret JSON: privateKey field is missing"
                    .to_string(),
            ))
        }
    };

    let key_path = PathBuf::from(&shellexpand::tilde(private_key).to_string());

    google_credentials_secret.private_key = if key_path.exists() {
        Some(read_file_base64(key_path)?)
    } else {
        Some(private_key.clone())
    };

    Ok(())
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

// Create a new secret
#[allow(clippy::single_match)]
fn create(secret: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let secret_str = read_file_or_stdin(secret)?;

    // Deserialize the secret JSON
    let mut create_secret_request: console::api_models::CreateSecretRequest =
        serde_yaml::from_str(&secret_str)
            .map_err(|e| CliError::dataerr(format!("Error parsing secret JSON: {e}")))?;

    // Apply special secret type logic
    match *create_secret_request.secret_type {
        console::api_models::SecretType::NodeId => {
            // Load the cert and key files if their values are paths
            load_node_id_tls_cert_key(&mut create_secret_request)?;
        }
        console::api_models::SecretType::GoogleCredentials => {
            // Load the private key file if its value is a path
            load_google_credentials_private_key(&mut create_secret_request)?;
        }
        _ => {}
    }

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

// Get a secret information by its ID
fn info(extended: bool, config: Option<&str>, secret_id: &str, json: bool) -> Result<(), CliError> {
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

// Update a secret
fn update(secret_id: &str, secret: &str, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let secret_str = read_file_or_stdin(secret)?;

    // Deserialize the secret JSON
    let update_secret_request: console::api_models::UpdateSecretByIdRequest =
        serde_yaml::from_str(&secret_str)
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
fn delete(secret_id: &str, yes: bool, config: Option<&str>, json: bool) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    // Prompt for confirmation if not using --yes
    if !yes {
        info(false, config, secret_id, false)?;

        if !confirm_deletion("secret", None) {
            return Ok(());
        }
    }

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
    secret: SecretCommand,
    config: Option<&str>,
    json: bool,
) -> Result<(), CliError> {
    match secret.command {
        SecretSubcommands::List { extended } => list(extended, config, json),
        SecretSubcommands::Info {
            secret_id,
            extended,
        } => info(extended, config, &secret_id, json),
        SecretSubcommands::Create { secret } => create(&secret, config, json),
        SecretSubcommands::Update { secret_id, secret } => {
            update(&secret_id, &secret, config, json)
        }
        SecretSubcommands::Delete { secret_id, yes } => delete(&secret_id, yes, config, json),
    }
}
