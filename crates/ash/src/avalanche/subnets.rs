// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche subnets and validators

use crate::avalanche::blockchains::AvalancheBlockchain;
use crate::avalanche::{
    avalanche_id_from_string, avalanche_node_id_from_string, AvalancheOutputOwners,
};
use crate::errors::*;
use avalanche_types::{ids::node::Id as NodeId, ids::Id};
use serde::{Deserialize, Serialize};

/// Avalanche Subnet
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheSubnet {
    #[serde(deserialize_with = "avalanche_id_from_string")]
    pub id: Id,
    pub control_keys: Vec<String>,
    pub threshold: u8,
    /// List of the Subnet's blockchains
    pub blockchains: Vec<AvalancheBlockchain>,
    /// List of the Subnet's validators
    #[serde(default)]
    pub validators: Vec<AvalancheSubnetValidator>,
}

impl AvalancheSubnet {
    /// Get a blockchain of the Subnet by its ID
    pub fn get_blockchain(&self, id: &str) -> Result<&AvalancheBlockchain, AshError> {
        self.blockchains
            .iter()
            .find(|&blockchain| blockchain.id.to_string() == id)
            .ok_or(
                AvalancheSubnetError::NotFound {
                    subnet_id: self.id,
                    target_type: "blockchain".to_string(),
                    target_value: id.to_string(),
                }
                .into(),
            )
    }

    /// Get a blockchain of the Subnet by its name
    pub fn get_blockchain_by_name(&self, name: &str) -> Result<&AvalancheBlockchain, AshError> {
        self.blockchains
            .iter()
            .find(|&blockchain| blockchain.name == name)
            .ok_or(
                AvalancheSubnetError::NotFound {
                    subnet_id: self.id,
                    target_type: "blockchain".to_string(),
                    target_value: name.to_string(),
                }
                .into(),
            )
    }

    /// Get a validator of the Subnet by its ID
    pub fn get_validator(&self, id: &str) -> Result<&AvalancheSubnetValidator, AshError> {
        self.validators
            .iter()
            .find(|&validator| validator.node_id.to_string() == id)
            .ok_or(
                AvalancheSubnetError::NotFound {
                    subnet_id: self.id,
                    target_type: "validator".to_string(),
                    target_value: id.to_string(),
                }
                .into(),
            )
    }
}

/// Avalanche Subnet validator
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheSubnetValidator {
    #[serde(rename = "txID", deserialize_with = "avalanche_id_from_string")]
    pub tx_id: Id,
    #[serde(rename = "nodeID", deserialize_with = "avalanche_node_id_from_string")]
    pub node_id: NodeId,
    #[serde(skip)]
    pub subnet_id: Id,
    pub start_time: u64,
    pub end_time: u64,
    pub stake_amount: u64,
    pub weight: u64,
    pub potential_reward: u64,
    pub delegation_fee: f32,
    pub connected: bool,
    pub uptime: f32,
    pub validation_reward_owner: AvalancheOutputOwners,
    pub delegator_count: u32,
    pub delegator_weight: u64,
    pub delegators: Vec<AvalancheSubnetDelegator>,
    pub delegation_reward_owner: AvalancheOutputOwners,
}

/// Avalanche Subnet delegator
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheSubnetDelegator {
    #[serde(rename = "txID", deserialize_with = "avalanche_id_from_string")]
    pub tx_id: Id,
    #[serde(rename = "nodeID", skip)]
    pub node_id: NodeId,
    pub start_time: u64,
    pub end_time: u64,
    pub weight: u64,
    pub stake_amount: u64,
    pub potential_reward: u64,
    pub reward_owner: AvalancheOutputOwners,
}

#[cfg(test)]
mod tests {
    use crate::avalanche::{AvalancheNetwork, AVAX_PRIMARY_NETWORK_ID};
    use std::env;

    const AVAX_FUJI_CCHAIN_ID: &str = "yH8D7ThNJkxmtkuv2jgBa4P1Rn3Qpr4pPr7QYNfcdoS6k6HWp";
    const ASH_TEST_NODE_ID: &str = "NodeID-7Xhw2mDxuDS44j42TCB6U5579esbSt3Lg";

    // Load the test network from the ASH_TEST_CONFIG file
    fn load_test_network() -> AvalancheNetwork {
        let config_path =
            env::var("ASH_TEST_AVAX_CONFIG").unwrap_or("tests/conf/default.yml".to_string());
        AvalancheNetwork::load("fuji", Some(&config_path)).unwrap()
    }

    #[test]
    fn test_avalanche_subnet_get_blockchain() {
        let fuji = load_test_network();
        let subnet = fuji.get_subnet(AVAX_PRIMARY_NETWORK_ID).unwrap();

        let blockchain = subnet.get_blockchain(AVAX_FUJI_CCHAIN_ID).unwrap();
        assert_eq!(blockchain.name, "C-Chain");
    }

    #[test]
    fn test_avalanche_subnet_get_blockchain_by_name() {
        let fuji = load_test_network();
        let subnet = fuji.get_subnet(AVAX_PRIMARY_NETWORK_ID).unwrap();

        let blockchain = subnet.get_blockchain_by_name("C-Chain").unwrap();
        assert_eq!(blockchain.id.to_string(), AVAX_FUJI_CCHAIN_ID);
    }

    #[test]
    fn test_avalanche_subnet_get_validator() {
        let mut fuji = load_test_network();
        fuji.update_subnet_validators(AVAX_PRIMARY_NETWORK_ID)
            .unwrap();

        let subnet = fuji.get_subnet(AVAX_PRIMARY_NETWORK_ID).unwrap();

        let validator = subnet.get_validator(ASH_TEST_NODE_ID).unwrap();
        assert_eq!(validator.node_id.to_string(), ASH_TEST_NODE_ID);
    }
}
