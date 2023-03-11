// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

pub mod blockchains;
pub mod jsonrpc;
pub mod nodes;
pub mod subnets;

// Module that contains code to interact with Avalanche networks

use crate::avalanche::blockchains::AvalancheBlockchain;
use crate::avalanche::subnets::AvalancheSubnet;
use crate::errors::*;
use crate::{avalanche::jsonrpc::platformvm, conf::AshConfig};
use avalanche_types::ids::{node::Id as NodeId, Id};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Avalanche primary network ID
/// This Subnet contains the P-Chain that is used for all Subnet operations
/// (the P-Chain ID is the same as the primary network ID)
pub const AVAX_PRIMARY_NETWORK_ID: &str = "11111111111111111111111111111111LpoYY";

/// Avalanche network
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheNetwork {
    pub name: String,
    /// List of the network's Subnets
    pub subnets: Vec<AvalancheSubnet>,
}

/// Avalanche output owners
/// See https://docs.avax.network/specs/platform-transaction-serialization#secp256k1-output-owners-output
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheOutputOwners {
    pub locktime: u64,
    pub threshold: u32,
    pub addresses: Vec<String>,
}

/// Deserialize an Avalanche ID from a string
fn avalanche_id_from_string<'de, D>(deserializer: D) -> Result<Id, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Id::from_str(&s).map_err(serde::de::Error::custom)
}

/// Deserialize an Avalanche NodeID from a string
fn avalanche_node_id_from_string<'de, D>(deserializer: D) -> Result<NodeId, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NodeId::from_str(&s).map_err(serde::de::Error::custom)
}

impl AvalancheNetwork {
    /// Load an AvalancheNetwork from the configuration
    pub fn load(network: &str, config: Option<&str>) -> Result<AvalancheNetwork, AshError> {
        let ash_config = AshConfig::load(config)?;
        let avax_network = ash_config
            .avalanche_networks
            .iter()
            .find(|&avax_network| avax_network.name == network)
            .ok_or(ConfigError::NotFound {
                target_type: "network".to_string(),
                target_value: network.to_string(),
            })?;

        // Error if the primary network is not found or if the P-Chain is not found
        let _ = avax_network
            .get_subnet(AVAX_PRIMARY_NETWORK_ID)?
            .get_blockchain(AVAX_PRIMARY_NETWORK_ID)?;

        Ok(avax_network.clone())
    }

    /// Get the P-Chain
    pub fn get_pchain(&self) -> Result<&AvalancheBlockchain, AshError> {
        let pchain = self
            .get_subnet(AVAX_PRIMARY_NETWORK_ID)?
            .get_blockchain(AVAX_PRIMARY_NETWORK_ID)?;
        Ok(pchain)
    }

    /// Get the C-Chain
    pub fn get_cchain(&self) -> Result<&AvalancheBlockchain, AshError> {
        let cchain = self
            .get_subnet(AVAX_PRIMARY_NETWORK_ID)?
            .get_blockchain_by_name("C-Chain")?;
        Ok(cchain)
    }

    /// Update the AvalancheNetwork Subnets by querying an API endpoint
    pub fn update_subnets(&mut self) -> Result<(), AshError> {
        let rpc_url = &self.get_pchain()?.rpc_url;

        let subnets =
            platformvm::get_network_subnets(&rpc_url).map_err(|e| RpcError::GetFailure {
                data_type: "network's Subnets".to_string(),
                target_type: "network".to_string(),
                target_value: self.name.clone(),
                msg: e.to_string(),
            })?;

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

    /// Get a Subnet of the network by its ID
    pub fn get_subnet(&self, id: &str) -> Result<&AvalancheSubnet, AshError> {
        self.subnets
            .iter()
            .find(|&subnet| subnet.id.to_string() == id)
            .ok_or(
                AvalancheNetworkError::NotFound {
                    network: self.name.clone(),
                    target_type: "Subnet".to_string(),
                    target_value: id.to_string(),
                }
                .into(),
            )
    }

    /// Update the AvalancheNetwork blockchains by querying an API endpoint
    /// This function will update the blockchains of all subnets
    pub fn update_blockchains(&mut self) -> Result<(), AshError> {
        let rpc_url = &self.get_pchain()?.rpc_url;

        let blockchains =
            platformvm::get_network_blockchains(&rpc_url).map_err(|e| RpcError::GetFailure {
                data_type: "blockchains".to_string(),
                target_type: "network".to_string(),
                target_value: self.name.clone(),
                msg: e.to_string(),
            })?;

        // For each Subnet, replace the blockchains with the ones returned by the API
        // Skip the primary network, as the P-Chain is not returned by the API
        let primary_subnet = self.get_subnet(AVAX_PRIMARY_NETWORK_ID).unwrap().clone();
        let mut subnets = self
            .subnets
            .iter()
            .filter(|subnet| subnet.id.to_string() != AVAX_PRIMARY_NETWORK_ID)
            .map(|subnet| {
                let mut subnet = subnet.clone();
                subnet.blockchains = blockchains
                    .iter()
                    .filter(|blockchain| blockchain.subnet_id == subnet.id)
                    .cloned()
                    .collect();
                subnet
            })
            .collect::<Vec<_>>();
        subnets.push(primary_subnet);

        self.subnets = subnets;
        Ok(())
    }

    /// Update the validators of a Subnet by querying an API endpoint
    pub fn update_subnet_validators(&mut self, subnet_id: &str) -> Result<(), AshError> {
        let rpc_url = &self.get_pchain()?.rpc_url;

        let validators = platformvm::get_current_validators(&rpc_url, subnet_id).map_err(|e| {
            RpcError::GetFailure {
                data_type: "validators".to_string(),
                target_type: "Subnet".to_string(),
                target_value: subnet_id.to_string(),
                msg: e.to_string(),
            }
        })?;

        // Replace the validators of the Subnet
        let mut subnet = self.get_subnet(subnet_id)?.clone();

        subnet.validators = validators;

        // Get the index of the Subnet
        let subnet_index = self
            .subnets
            .iter()
            .position(|subnet| subnet.id.to_string() == subnet_id)
            .ok_or(AvalancheNetworkError::NotFound {
                network: self.name.clone(),
                target_type: "Subnet".to_string(),
                target_value: subnet_id.to_string(),
            })?;

        // Replace the Subnet
        self.subnets[subnet_index] = subnet;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avalanche::blockchains::AvalancheBlockchain;
    use std::env;

    const AVAX_FUJI_CCHAIN_ID: &str = "yH8D7ThNJkxmtkuv2jgBa4P1Rn3Qpr4pPr7QYNfcdoS6k6HWp";
    const AVAX_FUJI_EVM_ID: &str = "mgj786NP7uDwBCcq6YwThhaN8FLyybkCa4zBWTQbNgmK6k9A6";
    const AVAX_FUJI_DFK_SUBNET_ID: &str = "XHLRR9cvMtCR8KZsjU8nLxg1JbV7aS23AcLVeBMVHLKkSBriS";
    const AVAX_FUJI_DFK_CHAIN_ID: &str = "32sexHqc3tBQsik8h7WP5F2ruL5svqhX5opeTgXCRVX8HpbKF";

    // Load the test network from the ASH_TEST_CONFIG file
    fn load_test_network() -> AvalancheNetwork {
        let config_path =
            env::var("ASH_TEST_AVAX_CONFIG").unwrap_or("tests/conf/default.yml".to_string());
        AvalancheNetwork::load("fuji", Some(&config_path)).unwrap()
    }

    #[test]
    fn test_avalanche_network_load() {
        // Only test the fuji network as the mainnet network is structurally the same
        let fuji = load_test_network();
        assert_eq!(fuji.name, "fuji");
        assert_eq!(fuji.subnets.len(), 1);

        let AvalancheSubnet {
            id,
            control_keys,
            threshold,
            blockchains,
            ..
        } = &fuji.subnets[0];
        assert_eq!(id.to_string(), AVAX_PRIMARY_NETWORK_ID);
        assert_eq!(control_keys.len(), 0);
        assert_eq!(threshold, &0);
        assert_eq!(blockchains.len(), 3);

        let AvalancheBlockchain {
            id,
            name,
            vm_id,
            vm_type,
            ..
        } = &blockchains[1];
        assert_eq!(id.to_string(), AVAX_FUJI_CCHAIN_ID);
        assert_eq!(name, "C-Chain");
        assert_eq!(vm_id.to_string(), AVAX_FUJI_EVM_ID);
        assert_eq!(vm_type, "EVM");

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
    fn test_avalanche_network_get_pchain() {
        let fuji = load_test_network();

        let pchain = fuji.get_pchain().unwrap();

        assert_eq!(pchain.id.to_string(), AVAX_PRIMARY_NETWORK_ID);
        assert_eq!(pchain.name, "P-Chain");
    }

    #[test]
    fn test_avalanche_network_get_subnet() {
        let fuji = load_test_network();

        // Should never fail as AVAX_PRIMARY_NETWORK_ID should always be a valid key
        let primary_subnet = fuji.get_subnet(AVAX_PRIMARY_NETWORK_ID).unwrap();
        assert_eq!(primary_subnet.id.to_string(), AVAX_PRIMARY_NETWORK_ID);
        assert_eq!(primary_subnet.blockchains.len(), 3);

        assert!(fuji.get_subnet("invalid").is_err());
    }

    #[test]
    fn test_avalanche_network_update_subnets() {
        let mut fuji = load_test_network();
        fuji.update_subnets().unwrap();

        // Test that the number of Subnets is greater than 1
        assert!(fuji.subnets.len() > 1);

        // Test that the primary network is still present
        // and that the P-Chain is still present
        let primary_subnet = fuji.get_subnet(AVAX_PRIMARY_NETWORK_ID).unwrap();
        assert_eq!(primary_subnet.id.to_string(), AVAX_PRIMARY_NETWORK_ID);
        assert_eq!(primary_subnet.blockchains.len(), 3);
        assert!(primary_subnet
            .blockchains
            .iter()
            .any(|blockchain| blockchain.id.to_string() == AVAX_PRIMARY_NETWORK_ID));

        // Test that the DFK Subnet is present
        let dfk_subnet = fuji.get_subnet(AVAX_FUJI_DFK_SUBNET_ID).unwrap();
        assert_eq!(dfk_subnet.id.to_string(), AVAX_FUJI_DFK_SUBNET_ID);
    }

    #[test]
    fn test_avalanche_network_update_blockchains() {
        let mut fuji = load_test_network();
        fuji.update_subnets().unwrap();
        fuji.update_blockchains().unwrap();

        // Test that the primary network is still present
        assert!(fuji
            .subnets
            .iter()
            .any(|subnet| subnet.id.to_string() == AVAX_PRIMARY_NETWORK_ID));

        // Test that the DFK Subnet contains the DFK chain
        let dfk_subnet = fuji.get_subnet(AVAX_FUJI_DFK_SUBNET_ID).unwrap();
        assert!(dfk_subnet
            .blockchains
            .iter()
            .any(|blockchain| blockchain.id.to_string() == AVAX_FUJI_DFK_CHAIN_ID));
    }

    #[test]
    fn test_avalanche_network_update_subnet_validators() {
        // The method platform.getCurrentValidators is not available on QuickNode
        // Tempoary workaround: use Ankr public endpoint
        let mut fuji = AvalancheNetwork::load("fuji-ankr", None).unwrap();
        fuji.update_subnets().unwrap();
        fuji.update_subnet_validators(AVAX_PRIMARY_NETWORK_ID)
            .unwrap();

        // Test that the primary network is still present
        assert!(fuji
            .subnets
            .iter()
            .any(|subnet| subnet.id.to_string() == AVAX_PRIMARY_NETWORK_ID));

        // Test that the primary network has validators
        let primary_subnet = fuji.get_subnet(AVAX_PRIMARY_NETWORK_ID).unwrap();
        assert!(primary_subnet.validators.len() > 0);
    }
}
