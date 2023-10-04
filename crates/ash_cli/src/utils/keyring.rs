// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

use crate::utils::error::CliError;
use keyring::Entry;

/// Store a value in the device keyring
pub(crate) fn set_keyring_value(target: &str, service: &str, value: &str) -> Result<(), CliError> {
    Entry::new_with_target(target, service, &whoami::username())
        .map_err(|e| CliError::dataerr(format!("Error storing access token: {e}")))?
        .set_password(value)
        .map_err(|e| CliError::dataerr(format!("Error storing access token: {e}")))
}

/// Get a value from the device keyring
pub(crate) fn get_keyring_value(target: &str, service: &str) -> Result<String, CliError> {
    Entry::new_with_target(target, service, &whoami::username())
        .map_err(|e| CliError::dataerr(format!("Error getting access token: {e}")))?
        .get_password()
        .map_err(|e| CliError::dataerr(format!("Error getting access token: {e}")))
}

/// Remove a value from the device keyring
pub(crate) fn delete_keyring_value(target: &str, service: &str) -> Result<(), CliError> {
    Entry::new_with_target(target, service, &whoami::username())
        .map_err(|e| CliError::dataerr(format!("Error removing access token: {e}")))?
        .delete_password()
        .map_err(|e| CliError::dataerr(format!("Error removing access token: {e}")))
}
