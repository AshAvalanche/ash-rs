// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

// Module that contains code to interact with the lib configuration

use crate::avalanche::AvalancheNetwork;
use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const DEFAULT_CONF: &str = include_str!("../conf/default.yml");

/// Ash lib configuration
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AshConfig {
    /// List of known Avalanche networks
    pub avalanche_networks: Vec<AvalancheNetwork>,
}

impl AshConfig {
    /// Load the Ash lib configuration from config files
    /// The default config file is located at `conf/avalanche.yml`
    /// A custom config can be provided with the config_file parameter
    pub fn load(config_file: Option<&str>) -> Result<AshConfig, ConfigError> {
        let ash_conf = Config::builder();

        match config_file {
            Some(config) => ash_conf.add_source(File::with_name(config)),
            None => ash_conf.add_source(File::from_str(DEFAULT_CONF, FileFormat::Yaml)),
        }
        .add_source(Environment::with_prefix("ASH"))
        .build()?
        .try_deserialize()
    }

    /// Dump the Ash lib default configuration to a file in YAML format
    pub fn dump_default(config_file: &str, force: bool) -> Result<(), String> {
        let ash_conf = Self::load(None).unwrap();

        // If the config file already exists, return an error unless force is set to true
        match (Path::new(config_file).exists(), force) {
            (true, false) => Err(format!("Configuration file '{config_file}' already exists")),
            _ => {
                fs::write(config_file, serde_yaml::to_string(&ash_conf).unwrap()).map_err(|e| {
                    format!("Failed to write default configuration to {config_file}: {e}")
                })?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avalanche::{
        blockchains::AvalancheBlockchain, subnets::AvalancheSubnet, AVAX_PRIMARY_NETWORK_ID,
    };

    const AVAX_PCHAIN_ID: &str = AVAX_PRIMARY_NETWORK_ID;
    const AVAX_MAINNET_CCHAIN_ID: &str = "2q9e4r6Mu3U68nU1fYjgbR6JvwrRx36CohpAX5UQxse55x1Q5";
    const AVAX_MAINNET_EVM_ID: &str = "mgj786NP7uDwBCcq6YwThhaN8FLyybkCa4zBWTQbNgmK6k9A6";
    const AVAX_MAINNET_CCHAIN_RPC: &str = "https://api.avax.network/ext/bc/C/rpc";

    #[test]
    fn test_ash_config_load() {
        // Only test the mainnet network as the fuji network is the same structurally
        let ash_config = AshConfig::load(None).unwrap();

        // Test the default configuration for avalanche_networks
        assert_eq!(ash_config.avalanche_networks.len(), 2);

        let mainnet = ash_config
            .avalanche_networks
            .iter()
            .find(|&network| network.name == "mainnet")
            .unwrap();
        assert_eq!(mainnet.name, "mainnet");
        assert_eq!(mainnet.subnets.len(), 1);

        let AvalancheSubnet {
            id,
            control_keys,
            threshold,
            blockchains,
        } = &mainnet.subnets[0];
        assert_eq!(id.to_string(), AVAX_PRIMARY_NETWORK_ID);
        assert_eq!(control_keys.len(), 0);
        assert_eq!(threshold, &0);
        assert_eq!(blockchains.len(), 3);

        let AvalancheBlockchain {
            id,
            name,
            vm_id,
            vm_type,
            rpc_url,
            ..
        } = &blockchains[1];
        assert_eq!(id.to_string(), AVAX_MAINNET_CCHAIN_ID);
        assert_eq!(name, "C-Chain");
        assert_eq!(vm_id.to_string(), AVAX_MAINNET_EVM_ID);
        assert_eq!(vm_type, "EVM");
        assert_eq!(rpc_url, AVAX_MAINNET_CCHAIN_RPC);
    }

    #[test]
    fn test_ash_config_load_custom() {
        let ash_config = AshConfig::load(Some("tests/conf/custom.yml")).unwrap();

        // Test the custom configuration for avalanche_networks
        assert_eq!(ash_config.avalanche_networks.len(), 1);

        // The configuration should contain the custom network
        let custom = ash_config
            .avalanche_networks
            .iter()
            .find(|&network| network.name == "custom")
            .unwrap();

        assert_eq!(custom.name, "custom");
        assert_eq!(custom.subnets.len(), 1);

        let AvalancheSubnet {
            id,
            control_keys,
            threshold,
            blockchains,
        } = &custom.subnets[0];
        assert_eq!(id.to_string(), AVAX_PRIMARY_NETWORK_ID);
        assert_eq!(control_keys.len(), 0);
        assert_eq!(threshold, &0);
        assert_eq!(blockchains.len(), 3);

        let AvalancheBlockchain {
            id,
            name,
            vm_type,
            rpc_url,
            ..
        } = &blockchains[0];
        assert_eq!(id.to_string(), AVAX_PCHAIN_ID);
        assert_eq!(name, "P-Chain");
        assert_eq!(vm_type, "PVM");
        assert_eq!(rpc_url, "https://api.ash.center/ext/bc/P");
    }

    #[test]
    fn test_ash_config_dump_default() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_file_path = temp_dir.path().join("ash.yml");
        let config_file = config_file_path.to_str().unwrap();
        let ash_config = AshConfig::load(None).unwrap();

        // Dump the default configuration to a file
        AshConfig::dump_default(config_file, false).unwrap();

        // Load the dumped configuration
        let dumped_config = AshConfig::load(Some(config_file)).unwrap();

        // Compare the dumped configuration with the default configuration
        assert_eq!(ash_config.avalanche_networks.len(), 2);
        assert_eq!(dumped_config.avalanche_networks.len(), 2);

        let mainnet = ash_config
            .avalanche_networks
            .iter()
            .find(|&network| network.name == "mainnet")
            .unwrap();
        let dumped_mainnet = dumped_config
            .avalanche_networks
            .iter()
            .find(|&network| network.name == "mainnet")
            .unwrap();

        assert_eq!(mainnet.name, dumped_mainnet.name);
        assert_eq!(mainnet.subnets.len(), dumped_mainnet.subnets.len());
    }
}
