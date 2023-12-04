// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

use crate::utils::error::CliError;
use colored::Colorize;
use keyring::{Entry, Error};
use std::{fs, path::Path};

/// Store a value in the device keyring
/// Returns true if the value was stored in the keyring, false if it was stored in a plain text file
pub(crate) fn set_keyring_value(
    target: &str,
    service: &str,
    value: &str,
    fallback_files_dir: &str,
) -> Result<bool, CliError> {
    let new_entry = Entry::new_with_target(target, service, &whoami::username())
        .map_err(|e| CliError::dataerr(format!("Error storing access token: {e}")))?
        .set_password(value);

    match new_entry {
        Ok(_) => Ok(true),
        Err(Error::PlatformFailure(_)) => {
            eprintln!("{}", "Your platform does not support keyring storage. Falling back to plain text storage.".red());
            write_plaintext_file(service, value, fallback_files_dir)?;
            Ok(false)
        }
        Err(e) => Err(CliError::dataerr(format!(
            "Error storing access token: {e}"
        ))),
    }
}

/// Get a value from the device keyring
pub(crate) fn get_keyring_value(
    target: &str,
    service: &str,
    fallback_files_dir: &str,
) -> Result<String, CliError> {
    let new_entry = Entry::new_with_target(target, service, &whoami::username())
        .map_err(|e| CliError::dataerr(format!("Error getting access token: {e}")))?
        .get_password();

    match new_entry {
        Ok(entry) => Ok(entry),
        Err(Error::PlatformFailure(_)) => read_plaintext_file(service, fallback_files_dir),
        Err(e) => Err(CliError::dataerr(format!(
            "Error getting access token: {e}"
        ))),
    }
}

/// Remove a value from the device keyring
/// Returns true if the value was removed from the keyring, false if it was removed from a plain text file
pub(crate) fn delete_keyring_value(
    target: &str,
    service: &str,
    fallback_files_dir: &str,
) -> Result<bool, CliError> {
    let new_entry = Entry::new_with_target(target, service, &whoami::username())
        .map_err(|e| CliError::dataerr(format!("Error removing access token: {e}")))?
        .delete_password();

    match new_entry {
        Ok(_) => Ok(true),
        Err(Error::PlatformFailure(_)) => {
            delete_plaintext_file(service, fallback_files_dir)?;
            Ok(false)
        }
        Err(e) => Err(CliError::dataerr(format!(
            "Error removing access token: {e}"
        ))),
    }
}

/// Store a value in a plain text file
/// This is used as a fallback if the device does not support keyring storage
fn write_plaintext_file(
    service: &str,
    value: &str,
    fallback_files_dir: &str,
) -> Result<(), CliError> {
    let plaintext_dir = shellexpand::tilde(fallback_files_dir).to_string();
    let plaintext_dir_path = Path::new(&plaintext_dir);

    if !plaintext_dir_path.exists() {
        fs::create_dir_all(plaintext_dir_path)
            .map_err(|e| CliError::dataerr(format!("Error creating output directory: {e}")))?;
    }

    let plaintext_file_path = plaintext_dir_path.join(service);

    fs::write(plaintext_file_path, value)
        .map_err(|e| CliError::dataerr(format!("Error writing plain text file: {e}")))?;

    Ok(())
}

/// Get a value from a plain text file
/// This is used as a fallback if the device does not support keyring storage
fn read_plaintext_file(service: &str, fallback_files_dir: &str) -> Result<String, CliError> {
    let plaintext_dir = shellexpand::tilde(fallback_files_dir).to_string();
    let plaintext_dir_path = Path::new(&plaintext_dir);

    if !plaintext_dir_path.exists() {
        CliError::dataerr(format!(
            "Plain text storage directory does not exist: {plaintext_dir}"
        ));
    }

    let plaintext_file_path = plaintext_dir_path.join(service);

    fs::read_to_string(plaintext_file_path)
        .map_err(|e| CliError::dataerr(format!("Error reading plain text file: {e}")))
}

/// Delete a plain text file
/// This is used as a fallback if the device does not support keyring storage
fn delete_plaintext_file(service: &str, fallback_files_dir: &str) -> Result<(), CliError> {
    let plaintext_dir = shellexpand::tilde(fallback_files_dir).to_string();
    let plaintext_dir_path = Path::new(&plaintext_dir);

    if !plaintext_dir_path.exists() {
        CliError::dataerr(format!(
            "Plain text storage directory does not exist: {plaintext_dir}"
        ));
    }

    let plaintext_file_path = plaintext_dir_path.join(service);

    fs::remove_file(plaintext_file_path)
        .map_err(|e| CliError::dataerr(format!("Error removing plain text file: {e}")))
}
