// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

pub mod oauth2;

// Module that contains code to interact with the Ash Console

use serde::{Deserialize, Serialize};

use crate::conf::AshConfig;
use crate::console::oauth2::AshConsoleOAuth2Client;
use crate::errors::*;

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AshConsole {
    /// Console URL
    pub api_url: String,
    /// Console OAuth2 client
    pub oauth2: AshConsoleOAuth2Client,
}

impl AshConsole {
    /// Load the Ash Console from the configuration
    pub fn load(config: Option<&str>) -> Result<AshConsole, AshError> {
        let ash_conf = AshConfig::load(config)?;

        match ash_conf.ash_console {
            Some(console) => Ok(console),
            None => Err(ConfigError::NotFound {
                target_type: "console".to_string(),
                target_value: "Ash".to_string(),
            }
            .into()),
        }
    }
}
