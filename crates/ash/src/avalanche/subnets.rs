// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche subnets and validators

use crate::avalanche::{
    avalanche_id_from_string, avalanche_node_id_from_string, blockchains::AvalancheBlockchain,
    AvalancheOutputOwners,
};
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
    /// Get a Blockchain from the Subnet by its ID
    pub fn get_blockchain(&self, id: &str) -> Option<&AvalancheBlockchain> {
        self.blockchains
            .iter()
            .find(|&blockchain| blockchain.id.to_string() == id)
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
    pub start_time: u64,
    pub end_time: u64,
    pub stake_amount: u64,
    pub weight: u64,
    pub potential_reward: u64,
    pub delegation_fee: f32,
    pub connected: bool,
    pub uptime: f32,
    pub validation_reward_owner: AvalancheOutputOwners,
    pub delegators: Vec<AvalancheSubnetDelegator>,
    pub delegator_count: u32,
    pub delegator_weight: u64,
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
