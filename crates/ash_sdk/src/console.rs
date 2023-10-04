// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

pub mod oauth2;
pub use ash_api::apis::configuration as api_config;
pub use ash_api::apis::default_api as api;
pub use ash_api::models as api_models;

// Module that contains code to interact with the Ash Console

use ash_api::apis::configuration::Configuration;
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

    /// Create a new Ash Console API configuration with the given access token
    pub fn create_api_config_with_access_token(&self, access_token: &str) -> Configuration {
        let mut config = Configuration::new();
        config.base_path = self.api_url.clone();
        config.oauth_access_token = Some(access_token.to_string());
        config
    }
}
