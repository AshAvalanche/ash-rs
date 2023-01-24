// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots
pub mod blockchains;
pub mod subnets;

// Module that contains code to interact with Avalanche networks

use avalanche_types::ids::Id;
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use subnets::AvalancheSubnet;

/// Global Avalanche configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct AvalancheConfig {
    networks: Vec<AvalancheNetwork>,
}

/// Avalanche network
#[derive(Debug, Serialize, Deserialize)]
pub struct AvalancheNetwork {
    pub name: String,
    /// Map of <Subnet ID, AvalancheSubnet>
    pub subnets: Vec<AvalancheSubnet>,
}

/// Deserialize an Avalanche ID from a string
fn avalanche_id_from_string<'de, D>(deserializer: D) -> Result<Id, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Id::from_str(&s).map_err(serde::de::Error::custom)
}

/// Load the Avalanche global configuration from the config files
/// The default config file is located at `conf/avalanche.yml`
/// A custom config can be provided with the config parameter
fn load_config(config: Option<&str>) -> Result<AvalancheConfig, ConfigError> {
    let ash_conf = Config::builder()
        .add_source(File::with_name("conf/avalanche_networks.yml"))
        .add_source(Environment::with_prefix("ASH"));

    match config {
        Some(config) => ash_conf.add_source(File::with_name(config)),
        None => ash_conf,
    }
    .build()?
    .try_deserialize()
}

impl AvalancheNetwork {
    /// Load an AvalancheNetwork from the global configuration
    pub fn load(network: &str) -> Result<AvalancheNetwork, String> {
        let config = load_config(None).map_err(|e| e.to_string())?;
        let avax_network = config
            .networks
            .iter()
            .find(|&avax_network| avax_network.name == network)
            .ok_or(format!("Avalanche network '{}' not found", network))?;

        Ok(avax_network.clone())
    }

    /// Get a Subnet from the network by its ID
    pub fn get_subnet(&self, id: &str) -> Option<&AvalancheSubnet> {
        self.subnets
            .iter()
            .find(|&subnet| subnet.id.to_string() == id)
    }
}

/// Implement the Clone trait for AvalancheNetwork
impl Clone for AvalancheNetwork {
    fn clone(&self) -> AvalancheNetwork {
        AvalancheNetwork {
            name: self.name.clone(),
            subnets: self.subnets.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use blockchains::AvalancheBlockchain;

    const AVAX_PRIMARY_NETWORK_ID: &str = "11111111111111111111111111111111LpoYY";
    const AVAX_MAINNET_CCHAIN_ID: &str = "2q9e4r6Mu3U68nU1fYjgbR6JvwrRx36CohpAX5UQxse55x1Q5";
    const AVAX_MAINNET_CCHAIN_RPC: &str = "https://api.avax.network/ext/bc/C/rpc";

    #[test]
    fn test_avalanche_network_load() {
        // Only test the mainnet network as the fuji network is the same structurally
        let mainnet = AvalancheNetwork::load("mainnet").unwrap();
        assert_eq!(mainnet.name, "mainnet");
        assert_eq!(mainnet.subnets.len(), 1);

        // Should never fail as AVAX_PRIMARY_NETWORK_ID should always be a valid key
        let AvalancheSubnet { id, blockchains } = &mainnet.subnets[0];
        assert_eq!(id.to_string(), AVAX_PRIMARY_NETWORK_ID);
        assert_eq!(blockchains.len(), 1);

        // Should never fail as AVAX_MAINNET_CCHAIN_ID should always be a valid key
        let AvalancheBlockchain {
            name,
            id,
            vm_type,
            rpc_url,
        } = &blockchains[0];
        assert_eq!(name, "C-Chain");
        assert_eq!(vm_type, "EVM");
        assert_eq!(id.to_string(), AVAX_MAINNET_CCHAIN_ID);
        assert_eq!(rpc_url, AVAX_MAINNET_CCHAIN_RPC);

        assert!(AvalancheNetwork::load("invalid").is_err());
    }

    #[test]
    fn test_avalanche_network_get_subnet() {
        let mainnet = AvalancheNetwork::load("mainnet").unwrap();

        // Should never fail as AVAX_PRIMARY_NETWORK_ID should always be a valid key
        let mainnet_subnet = mainnet.get_subnet(AVAX_PRIMARY_NETWORK_ID).unwrap();
        assert_eq!(mainnet_subnet.id.to_string(), AVAX_PRIMARY_NETWORK_ID);
        assert_eq!(mainnet_subnet.blockchains.len(), 1);

        assert!(mainnet.get_subnet("invalid").is_none());
    }

    #[test]
    fn test_load_networks() {
        // Only test the mainnet network as the fuji network is the same structurally
        let networks = load_config(None).unwrap().networks;
        assert_eq!(networks.len(), 2);

        let mainnet = networks
            .iter()
            .find(|&network| network.name == "mainnet")
            .unwrap();
        assert_eq!(mainnet.name, "mainnet");
        assert_eq!(mainnet.subnets.len(), 1);

        let AvalancheSubnet { id, blockchains } = &mainnet.subnets[0];
        assert_eq!(id.to_string(), AVAX_PRIMARY_NETWORK_ID);
        assert_eq!(blockchains.len(), 1);

        let AvalancheBlockchain {
            name,
            id,
            vm_type,
            rpc_url,
        } = &blockchains[0];
        assert_eq!(name, "C-Chain");
        assert_eq!(id.to_string(), AVAX_MAINNET_CCHAIN_ID);
        assert_eq!(vm_type, "EVM");
        assert_eq!(rpc_url, AVAX_MAINNET_CCHAIN_RPC);
    }
}
