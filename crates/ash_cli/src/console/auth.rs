// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the auth subcommand parser

use crate::{
    console::{
        load_console, KEYRING_ACCESS_TOKEN_SERVICE, KEYRING_REFRESH_TOKEN_SERVICE, KEYRING_TARGET,
    },
    utils::{
        error::CliError, get_keyring_value, store_keyring_value, templating::*, version_tx_cmd,
    },
};
use ash_sdk::console::AshConsole;
use clap::{Parser, Subcommand};
use colored::Colorize;
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
/// Authenticate with the Ash Console
pub(crate) struct AuthCommand {
    #[command(subcommand)]
    command: AuthSubcommands,
}

#[derive(Subcommand)]
enum AuthSubcommands {
    /// Login to the Ash Console. Credentials are stored in the device keyring.
    #[command(version = version_tx_cmd(false))]
    Login,
    /// Refresh the Ash Console access token
    #[command(version = version_tx_cmd(false))]
    RefreshToken,
    /// Show the current access token
    #[command(version = version_tx_cmd(false))]
    ShowToken,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    exp: usize,
    iat: usize,
    auth_time: usize,
    jti: String,
    iss: String,
    sub: String,
    typ: String,
    azp: String,
    session_state: String,
    sid: String,
    #[serde(rename = "preferred_username")]
    username: String,
    name: String,
    given_name: String,
    family_name: String,
    email: String,
}

// Refresh the user access token to the Ash Console
pub(crate) fn refresh_keyring_access_token(console: &AshConsole) -> Result<(), CliError> {
    // Get the refresh token from the keyring
    let refresh_token = get_keyring_value(KEYRING_TARGET, KEYRING_REFRESH_TOKEN_SERVICE)?;

    // Exchange the refresh token for a new access token
    let access_token = console
        .oauth2
        .refresh_access_token(&refresh_token)
        .map_err(|e| CliError::dataerr(format!("Error refreshing access token: {e}")))?;

    // Store the access token in the keyring
    store_keyring_value(
        KEYRING_TARGET,
        KEYRING_ACCESS_TOKEN_SERVICE,
        &access_token.secret().to_string(),
    )?;

    Ok(())
}

// Get the current access token from the keyring
pub(crate) fn get_keyring_access_token() -> Result<String, CliError> {
    get_keyring_value(KEYRING_TARGET, KEYRING_ACCESS_TOKEN_SERVICE)
}

// Decode the access token to get its token data
pub(crate) fn decode_access_token(access_token: &str) -> Result<TokenData<Claims>, CliError> {
    let mut token_validation = Validation::default();
    token_validation.insecure_disable_signature_validation();
    token_validation.validate_exp = false;

    let token_data = decode::<Claims>(
        access_token,
        &DecodingKey::from_secret("secret".as_ref()),
        &token_validation,
    )
    .map_err(|e| CliError::dataerr(format!("Error decoding access token: {e}")))?;

    Ok(token_data)
}

// Get an access token. If the access token is expired, refresh it.
#[allow(dead_code)]
pub(crate) fn get_access_token(console: &AshConsole) -> Result<String, CliError> {
    // Get the access token from the keyring
    let access_token = get_keyring_access_token()?;

    // Decode the access token to get its token data
    let token_data = decode_access_token(&access_token)?;

    // If the access token is expired, refresh it
    if token_data.claims.exp < (chrono::Utc::now().timestamp() as usize) {
        refresh_keyring_access_token(console)?;
        return get_keyring_access_token();
    }

    Ok(access_token)
}

// Login to the Ash Console
#[allow(clippy::unnecessary_to_owned)]
fn login(config: Option<&str>) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    eprintln!("Logging in to Ash Console at {}", console.api_url);

    console.oauth2.init();

    // Generate the authorization URL and user code
    let device_authorization = console
        .oauth2
        .generate_device_authorization_response(None)
        .map_err(|e| CliError::dataerr(format!("Error generating authorization URL: {e}")))?;

    println!(
        "Please open the following URL in your browser:\n{}\nand enter the code: {}",
        type_colorize(&device_authorization.verification_uri().to_string()),
        type_colorize(&device_authorization.user_code().secret().to_string())
    );

    let (access_token, refresh_token) = console
        .oauth2
        .exchange_device_code(&device_authorization)
        .map_err(|e| CliError::dataerr(format!("Error getting access token: {e}")))?;

    // Store the access token and refresh token in the keyring
    store_keyring_value(
        KEYRING_TARGET,
        KEYRING_ACCESS_TOKEN_SERVICE,
        &access_token.secret().to_string(),
    )?;
    store_keyring_value(
        KEYRING_TARGET,
        KEYRING_REFRESH_TOKEN_SERVICE,
        &refresh_token.secret().to_string(),
    )?;

    println!("\nLogin successful! The credentials have been stored in your device keyring.");

    Ok(())
}

// Refresh the Ash Console access token
fn refresh_access_token(config: Option<&str>) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    eprintln!(
        "Refreshing access token for Ash Console at {}",
        console.api_url
    );

    console.oauth2.init();

    refresh_keyring_access_token(&console)?;

    println!("\nAccess token refreshed successfully!");

    Ok(())
}

// Show the current access token
fn show_access_token(config: Option<&str>, json: bool) -> Result<(), CliError> {
    let console = load_console(config)?;

    let access_token = get_keyring_access_token()?;

    eprintln!(
        "Showing access token for Ash Console at {}",
        console.api_url
    );

    let token_data = decode_access_token(&access_token)?;

    if json {
        println!(
            "{}",
            serde_json::json!({ "accessToken": access_token, "tokenHeader": token_data.header, "tokenClaims": token_data.claims })
        );
        return Ok(());
    }

    println!(
        "Access token ({}):\n{}",
        match token_data.claims.exp < (chrono::Utc::now().timestamp() as usize) {
            true => "expired".red(),
            false => "valid".green(),
        },
        type_colorize(&access_token.to_string()),
    );

    Ok(())
}

// Parse console subcommand
pub(crate) fn parse(auth: AuthCommand, config: Option<&str>, json: bool) -> Result<(), CliError> {
    match auth.command {
        AuthSubcommands::Login => login(config),
        AuthSubcommands::RefreshToken => refresh_access_token(config),
        AuthSubcommands::ShowToken => show_access_token(config, json),
    }
}
