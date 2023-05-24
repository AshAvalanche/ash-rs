// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche Subnets and validators

use crate::avalanche::{
    blockchains::AvalancheBlockchain, jsonrpc::platformvm::SubnetStringControlKeys, txs::p,
    wallets::AvalancheWallet, AvalancheOutputOwners, AVAX_PRIMARY_NETWORK_ID,
};
use crate::errors::*;
use avalanche_types::{
    ids::{node::Id as NodeId, Id},
    jsonrpc::platformvm::{ApiPrimaryDelegator, ApiPrimaryValidator},
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Avalanche Subnet types
#[derive(Default, Debug, Display, Clone, Serialize, Deserialize, PartialEq)]
pub enum AvalancheSubnetType {
    PrimaryNetwork,
    #[default]
    Permissioned,
    /// Also named "PoS" in the Avalanche documentation
    Elastic,
}

/// Avalanche Subnet
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheSubnet {
    pub id: Id,
    #[serde(default)]
    pub subnet_type: AvalancheSubnetType,
    // We do not use ShortIds here to avoid having to retransform the control keys to addresses later
    #[serde(default)]
    pub control_keys: Vec<String>,
    #[serde(default)]
    pub threshold: u32,
    /// List of the Subnet's blockchains
    pub blockchains: Vec<AvalancheBlockchain>,
    /// List of the Subnet's validators
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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
                    subnet_id: self.id.to_string(),
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
                    subnet_id: self.id.to_string(),
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
                    subnet_id: self.id.to_string(),
                    target_type: "validator".to_string(),
                    target_value: id.to_string(),
                }
                .into(),
            )
    }

    /// Create a new Subnet
    /// TODO: Add control keys and threshold as parameters
    /// See: https://github.com/ava-labs/avalanche-types-rs/pull/76
    pub async fn create(
        wallet: &AvalancheWallet,
        check_acceptance: bool,
    ) -> Result<Self, AshError> {
        let tx_id = p::create_subnet(wallet, check_acceptance).await?;

        Ok(Self {
            id: tx_id,
            control_keys: vec![wallet.pchain_wallet.p_address.clone()],
            threshold: 1,
            subnet_type: AvalancheSubnetType::Permissioned,
            ..Default::default()
        })
    }
}

impl From<SubnetStringControlKeys> for AvalancheSubnet {
    fn from(subnet: SubnetStringControlKeys) -> Self {
        Self {
            id: subnet.id,
            // Based on Avalanche documentation at https://docs.avax.network/apis/avalanchego/apis/p-chain#platformgetsubnets
            // "If the Subnet is a PoS (= elastic) Subnet, then threshold will be 0 and controlKeys will be empty."
            // The Primary Network is an elastic Subnet and its ID is hardcoded
            subnet_type: match subnet.threshold {
                0 => match subnet.id.to_string().as_str() {
                    AVAX_PRIMARY_NETWORK_ID => AvalancheSubnetType::PrimaryNetwork,
                    _ => AvalancheSubnetType::Elastic,
                },
                _ => AvalancheSubnetType::Permissioned,
            },
            control_keys: subnet.control_keys,
            threshold: subnet.threshold,
            ..Default::default()
        }
    }
}

/// Avalanche Subnet validator
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheSubnetValidator {
    #[serde(rename = "txID")]
    pub tx_id: Id,
    #[serde(rename = "nodeID")]
    pub node_id: NodeId,
    #[serde(skip)]
    pub subnet_id: Id,
    pub start_time: u64,
    pub end_time: u64,
    pub stake_amount: Option<u64>,
    pub weight: Option<u64>,
    pub potential_reward: Option<u64>,
    pub connected: bool,
    pub uptime: Option<f32>,
    pub validation_reward_owner: Option<AvalancheOutputOwners>,
    pub delegator_count: Option<u64>,
    pub delegator_weight: Option<u64>,
    pub delegators: Option<Vec<AvalancheSubnetDelegator>>,
    pub delegation_fee: Option<f32>,
    pub delegation_reward_owner: Option<AvalancheOutputOwners>,
}

impl AvalancheSubnetValidator {
    pub fn from_api_primary_validator(validator: &ApiPrimaryValidator, subnet_id: Id) -> Self {
        Self {
            tx_id: validator.tx_id,
            node_id: validator.node_id,
            subnet_id,
            start_time: validator.start_time,
            end_time: validator.end_time,
            stake_amount: validator.stake_amount,
            weight: validator.weight,
            potential_reward: validator.potential_reward,
            connected: validator.connected,
            uptime: validator.uptime,
            validation_reward_owner: validator
                .validation_reward_owner
                .clone()
                .map(AvalancheOutputOwners::from),
            delegator_count: validator.delegator_count,
            delegator_weight: validator.delegator_weight,
            delegators: validator.delegators.clone().map(|delegators| {
                delegators
                    .into_iter()
                    .map(AvalancheSubnetDelegator::from)
                    .collect()
            }),
            delegation_fee: validator.delegation_fee,
            delegation_reward_owner: validator
                .delegation_reward_owner
                .clone()
                .map(AvalancheOutputOwners::from),
        }
    }
}

/// Avalanche Subnet delegator
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheSubnetDelegator {
    #[serde(rename = "txID")]
    pub tx_id: Id,
    #[serde(rename = "nodeID", skip)]
    pub node_id: NodeId,
    pub start_time: u64,
    pub end_time: u64,
    pub stake_amount: u64,
    pub potential_reward: Option<u64>,
    pub reward_owner: Option<AvalancheOutputOwners>,
}

impl From<ApiPrimaryDelegator> for AvalancheSubnetDelegator {
    fn from(delegator: ApiPrimaryDelegator) -> Self {
        Self {
            tx_id: delegator.tx_id,
            node_id: delegator.node_id,
            start_time: delegator.start_time,
            end_time: delegator.end_time,
            stake_amount: delegator.stake_amount,
            potential_reward: delegator.potential_reward,
            reward_owner: delegator.reward_owner.map(AvalancheOutputOwners::from),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avalanche::{AvalancheNetwork, AVAX_PRIMARY_NETWORK_ID};

    const NETWORK_RUNNER_CCHAIN_ID: &str = "VctwH3nkmztWbkdNXbuo6eCYndsUuemtM9ZFmEUZ5QpA1Fu8G";
    const NETWORK_RUNNER_NODE_ID: &str = "NodeID-MFrZFVCXPv5iCn6M9K6XduxGTYp891xXZ";
    const AVAX_EWOQ_PRIVATE_KEY: &str =
        "PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN";

    // Load the test network using avalanche-network-runner
    fn load_test_network() -> AvalancheNetwork {
        AvalancheNetwork::load("local", Some("tests/conf/avalanche-network-runner.yml")).unwrap()
    }

    #[test]
    #[ignore]
    fn test_avalanche_subnet_get_blockchain() {
        let local_network = load_test_network();
        let subnet = local_network.get_subnet(AVAX_PRIMARY_NETWORK_ID).unwrap();

        let blockchain = subnet.get_blockchain(NETWORK_RUNNER_CCHAIN_ID).unwrap();
        assert_eq!(blockchain.name, "C-Chain");
    }

    #[test]
    #[ignore]
    fn test_avalanche_subnet_get_blockchain_by_name() {
        let local_network = load_test_network();
        let subnet = local_network.get_subnet(AVAX_PRIMARY_NETWORK_ID).unwrap();

        let blockchain = subnet.get_blockchain_by_name("C-Chain").unwrap();
        assert_eq!(blockchain.id.to_string(), NETWORK_RUNNER_CCHAIN_ID);
    }

    #[test]
    #[ignore]
    fn test_avalanche_subnet_get_validator() {
        let mut local_network = load_test_network();
        local_network
            .update_subnet_validators(AVAX_PRIMARY_NETWORK_ID)
            .unwrap();

        let subnet = local_network.get_subnet(AVAX_PRIMARY_NETWORK_ID).unwrap();

        let validator = subnet.get_validator(NETWORK_RUNNER_NODE_ID).unwrap();
        assert_eq!(validator.node_id.to_string(), NETWORK_RUNNER_NODE_ID);
    }

    #[async_std::test]
    #[serial_test::serial]
    #[ignore]
    async fn test_avalanche_subnet_create() {
        let mut local_network = load_test_network();
        let wallet = local_network
            .create_wallet_from_cb58(AVAX_EWOQ_PRIVATE_KEY)
            .unwrap();

        let created_subnet = AvalancheSubnet::create(&wallet, true).await.unwrap();

        local_network.update_subnets().unwrap();
        let network_subnet = local_network
            .get_subnet(&created_subnet.id.to_string())
            .unwrap();

        assert_eq!(&created_subnet, network_subnet);
    }
}
