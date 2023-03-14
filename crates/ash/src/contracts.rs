// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

pub mod ash_router_http;

// Module that contains code to interact with Ash contracts

use crate::{conf::AshConfig, errors::*};
use serde::{Deserialize, Serialize};

/// Ash contract metadata
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AshContractMetadata {
    pub name: String,
    pub addresses: Vec<AshContractAddress>,
}

/// Ash contract address on a specific network
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AshContractAddress {
    pub network: String,
    pub address: String,
}

impl AshContractMetadata {
    /// Load an AshContract from the configuration
    pub fn load(name: &str, config: Option<&str>) -> Result<AshContractMetadata, AshError> {
        let ash_config = AshConfig::load(config)?;

        let contract = ash_config
            .ash_contracts
            .iter()
            .find(|&contract| contract.name == name)
            .ok_or(ConfigError::NotFound {
                target_type: "Ash contract".to_string(),
                target_value: name.to_string(),
            })?;

        Ok(contract.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conf::AshConfig;

    #[test]
    fn test_load() {
        let config = AshConfig::load(Some("tests/conf/default.yml")).unwrap();

        let test_contract = &config.ash_contracts[0].name;

        let contract = AshContractMetadata::load(test_contract, None).unwrap();
        assert_eq!(&contract.name, test_contract);
        assert_eq!(
            contract.addresses.len(),
            config.ash_contracts[0].addresses.len()
        );
        assert_eq!(
            contract.addresses[0].network,
            config.ash_contracts[0].addresses[0].network
        );
        assert_eq!(
            contract.addresses[0].address,
            config.ash_contracts[0].addresses[0].address
        );
    }
}
