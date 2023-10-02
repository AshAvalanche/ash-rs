// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

pub(crate) mod error;
pub(crate) mod parsing;
pub(crate) mod templating;

use crate::utils::error::CliError;
use clap::{builder::Str, crate_version};
use keyring::Entry;

/// Enrich the version information with whether the command triggers a transaction or not
/// This is used to help use the CLI in a non-interactive way
pub(crate) fn version_tx_cmd(is_tx: bool) -> Str {
    Str::from(format!("{} (tx_cmd={})", crate_version!(), is_tx))
}

/// Store a value in the device keyring
pub(crate) fn store_keyring_value(
    target: &str,
    service: &str,
    value: &str,
) -> Result<(), CliError> {
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
