// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

pub mod blockchains;
pub mod jsonrpc;
pub mod subnets;

// Module that contains code to interact with Avalanche networks

use crate::{avalanche::jsonrpc::platformvm, avalanche::subnets::AvalancheSubnet, conf::AshConfig};
use avalanche_types::ids::Id;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Avalanche primary network ID
/// This subnet contains the P-Chain that is used for all subnet operations
/// (the P-Chain ID is the same as the primary network ID)
pub const AVAX_PRIMARY_NETWORK_ID: &str = "11111111111111111111111111111111LpoYY";

/// Avalanche network
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheNetwork {
    pub name: String,
    /// List of the network's Subnets
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

impl AvalancheNetwork {
    /// Load an AvalancheNetwork from the configuration
    pub fn load(network: &str, config: Option<&str>) -> Result<AvalancheNetwork, String> {
        let ash_config = AshConfig::load(config).map_err(|e| e.to_string())?;
        let avax_network = ash_config
            .avalanche_networks
            .iter()
            .find(|&avax_network| avax_network.name == network)
            .ok_or(format!(
                "Avalanche network '{network}' not found in configuration"
            ))?;

        // Error if the primary network is not found
        let primary_subnet = avax_network
            .subnets
            .iter()
            .find(|&subnet| subnet.id.to_string() == AVAX_PRIMARY_NETWORK_ID)
            .ok_or(format!(
                "Primary network (ID: '{AVAX_PRIMARY_NETWORK_ID}') not found in configuration"
            ))?;

        // Error if the P-Chain is not found
        let _ = primary_subnet
            .get_blockchain(AVAX_PRIMARY_NETWORK_ID)
            .ok_or(format!(
                "P-Chain (ID: '{AVAX_PRIMARY_NETWORK_ID}') not found in configuration",
            ))?;

        Ok(avax_network.clone())
    }

    /// Update the AvalancheNetwork subnets by querying an API endpoint
    pub fn update_subnets(&mut self) -> Result<(), String> {
        // Get the P-Chain RPC URL
        let rpc_url = &self
            .get_subnet(AVAX_PRIMARY_NETWORK_ID)
            .ok_or(format!(
                "Primary network (ID: '{AVAX_PRIMARY_NETWORK_ID}') not found in configuration"
            ))?
            .get_blockchain(AVAX_PRIMARY_NETWORK_ID)
            .ok_or(format!(
                "P-Chain (ID: '{AVAX_PRIMARY_NETWORK_ID}') not found in configuration",
            ))?
            .rpc_url;

        let subnets = platformvm::get_network_subnets(rpc_url).map_err(|e| e.to_string())?;

        // Replace the primary network with the pre-configured one
        // This is done to ensure that the P-Chain is kept in the blockchains list
        // (it is not returned by the API)
        let primary_subnet = self.get_subnet(AVAX_PRIMARY_NETWORK_ID).unwrap().clone();
        let mut subnets = subnets
            .into_iter()
            .filter(|subnet| subnet.id.to_string() != AVAX_PRIMARY_NETWORK_ID)
            .collect::<Vec<_>>();
        subnets.push(primary_subnet);

        self.subnets = subnets;
        Ok(())
    }

    /// Get a Subnet from the network by its ID
    pub fn get_subnet(&self, id: &str) -> Option<&AvalancheSubnet> {
        self.subnets
            .iter()
            .find(|&subnet| subnet.id.to_string() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blockchains::AvalancheBlockchain;

    const AVAX_MAINNET_CCHAIN_ID: &str = "2q9e4r6Mu3U68nU1fYjgbR6JvwrRx36CohpAX5UQxse55x1Q5";
    const AVAX_MAINNET_EVM_ID: &str = "mgj786NP7uDwBCcq6YwThhaN8FLyybkCa4zBWTQbNgmK6k9A6";
    const AVAX_MAINNET_CCHAIN_RPC: &str = "https://api.avax.network/ext/bc/C/rpc";

    #[test]
    fn test_avalanche_network_load() {
        // Only test the mainnet network as the fuji network is the same structurally
        let mainnet = AvalancheNetwork::load("mainnet", None).unwrap();
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
        } = &blockchains[1];
        assert_eq!(id.to_string(), AVAX_MAINNET_CCHAIN_ID);
        assert_eq!(name, "C-Chain");
        assert_eq!(vm_id.to_string(), AVAX_MAINNET_EVM_ID);
        assert_eq!(vm_type, "EVM");
        assert_eq!(rpc_url, AVAX_MAINNET_CCHAIN_RPC);

        assert!(AvalancheNetwork::load("invalid", None).is_err());
    }

    #[test]
    fn test_avalanche_network_load_no_primary() {
        // Load the wrong.yml file which doesn't have the primary network
        // This should fail as the primary network is required
        assert!(
            AvalancheNetwork::load("no-primary-network", Some("tests/conf/wrong.yml")).is_err()
        );
    }

    #[test]
    fn test_avalanche_network_load_no_pchain() {
        // Load the wrong.yml file which doesn't have the P-Chain
        // This should fail as the P-Chain is required
        assert!(AvalancheNetwork::load("no-pchain", Some("tests/conf/wrong.yml")).is_err());
    }

    #[test]
    fn test_avalanche_network_get_subnet() {
        let mainnet = AvalancheNetwork::load("mainnet", None).unwrap();

        // Should never fail as AVAX_PRIMARY_NETWORK_ID should always be a valid key
        let mainnet_subnet = mainnet.get_subnet(AVAX_PRIMARY_NETWORK_ID).unwrap();
        assert_eq!(mainnet_subnet.id.to_string(), AVAX_PRIMARY_NETWORK_ID);
        assert_eq!(mainnet_subnet.blockchains.len(), 3);

        assert!(mainnet.get_subnet("invalid").is_none());
    }

    #[test]
    fn test_avalanche_network_update_subnets() {
        let mut mainnet = AvalancheNetwork::load("mainnet", None).unwrap();
        mainnet.update_subnets().unwrap();

        // Test that the number of subnets is greater than 1
        assert!(mainnet.subnets.len() > 1);

        // Test that the primary network is still present
        // and that the P-Chain is still present
        let primary_subnet = mainnet.get_subnet(AVAX_PRIMARY_NETWORK_ID).unwrap();
        assert_eq!(primary_subnet.id.to_string(), AVAX_PRIMARY_NETWORK_ID);
        assert_eq!(primary_subnet.blockchains.len(), 3);
        assert!(primary_subnet
            .blockchains
            .iter()
            .any(|blockchain| blockchain.id.to_string() == AVAX_PRIMARY_NETWORK_ID));
    }
}
