// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use crate::utils::error::CliError;

pub const ASH_CLI_STATE_FILE: &str = "~/.local/state/ash/state.json";

/// Ash CLI state to be stored in a JSON file
#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CliState {
    pub(crate) current_project: Option<String>,
}

impl CliState {
    /// Load the CLI state from the state file
    pub(crate) fn load() -> Result<Self, CliError> {
        let state_file = shellexpand::tilde(ASH_CLI_STATE_FILE).to_string();
        let state_file = Path::new(&state_file);

        if !state_file.exists() {
            return Ok(Self::default());
        }

        let state_file = fs::File::open(state_file)
            .map_err(|e| CliError::dataerr(format!("Error opening state file: {e}")))?;
        let state: Self = serde_json::from_reader(state_file)
            .map_err(|e| CliError::dataerr(format!("Error parsing state file: {e}")))?;

        Ok(state)
    }

    /// Save the CLI state to the state file
    pub(crate) fn save(&self) -> Result<(), CliError> {
        let state_file = shellexpand::tilde(ASH_CLI_STATE_FILE).to_string();
        let state_file = Path::new(&state_file);

        // Create the state file parent directory if it doesn't exist
        if let Some(parent) = state_file.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| CliError::dataerr(format!("Error creating state file: {e}")))?;
            }
        }

        let state_file = fs::File::create(state_file)
            .map_err(|e| CliError::dataerr(format!("Error creating state file: {e}")))?;
        serde_json::to_writer_pretty(state_file, self)
            .map_err(|e| CliError::dataerr(format!("Error writing state file: {e}")))?;

        Ok(())
    }
}
