// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche Subnets and validators

use crate::{
    avalanche::{
        blockchains::AvalancheBlockchain,
        jsonrpc::{info, platformvm::SubnetStringControlKeys, subnet_evm},
        nodes::AvalancheNode,
        txs::p,
        wallets::AvalancheWallet,
        warp::WarpMessageNodeSignature,
        AvalancheOutputOwners, AVAX_PRIMARY_NETWORK_ID,
    },
    errors::*,
};
use avalanche_types::{
    ids::{node::Id as NodeId, Id},
    jsonrpc::platformvm::{ApiPrimaryDelegator, ApiPrimaryValidator},
    utils::urls::extract_scheme_host_port_path_chain_alias,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::warp::WarpMessage;

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
    #[serde(default)]
    pub validators: Vec<AvalancheSubnetValidator>,
    #[serde(default)]
    pub pending_validators: Vec<AvalancheSubnetValidator>,
}

impl AvalancheSubnet {
    /// Get a blockchain of the Subnet by its ID
    pub fn get_blockchain(&self, id: Id) -> Result<&AvalancheBlockchain, AshError> {
        self.blockchains
            .iter()
            .find(|&blockchain| blockchain.id == id)
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
    pub fn get_validator(&self, id: NodeId) -> Result<&AvalancheSubnetValidator, AshError> {
        self.validators
            .iter()
            .find(|&validator| validator.node_id == id)
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

    /// Add a validator to the Primary Network
    /// Fail if the Subnet is not the Primary Network
    pub async fn add_avalanche_validator(
        &self,
        wallet: &AvalancheWallet,
        node_id: NodeId,
        stake_amount: u64,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        reward_fee_percent: u32,
        check_acceptance: bool,
    ) -> Result<AvalancheSubnetValidator, AshError> {
        // Check if the Subnet is the Primary Network
        if self.subnet_type != AvalancheSubnetType::PrimaryNetwork {
            return Err(AvalancheSubnetError::OperationNotAllowed {
                operation: "add_avalanche_validator".to_string(),
                subnet_id: self.id.to_string(),
                subnet_type: self.subnet_type.to_string(),
            }
            .into());
        }

        let tx_id = p::add_avalanche_validator(
            wallet,
            node_id,
            stake_amount,
            start_time,
            end_time,
            reward_fee_percent,
            check_acceptance,
        )
        .await?;

        Ok(AvalancheSubnetValidator {
            tx_id,
            node_id,
            subnet_id: self.id,
            start_time: start_time.timestamp() as u64,
            end_time: end_time.timestamp() as u64,
            stake_amount: Some(stake_amount),
            delegation_fee: Some(reward_fee_percent as f32),
            validation_reward_owner: Some(AvalancheOutputOwners {
                locktime: 0,
                threshold: 1,
                addresses: vec![wallet.pchain_wallet.p_address.clone()],
            }),
            delegation_reward_owner: Some(AvalancheOutputOwners {
                locktime: 0,
                threshold: 1,
                addresses: vec![wallet.pchain_wallet.p_address.clone()],
            }),
            ..Default::default()
        })
    }

    /// Add a validator to a permissioned Subnet
    pub async fn add_validator_permissioned(
        &self,
        wallet: &AvalancheWallet,
        node_id: NodeId,
        weight: u64,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        check_acceptance: bool,
    ) -> Result<AvalancheSubnetValidator, AshError> {
        // Check if the Subnet is permissioned
        if self.subnet_type != AvalancheSubnetType::Permissioned {
            return Err(AvalancheSubnetError::OperationNotAllowed {
                operation: "add_validator_permissioned".to_string(),
                subnet_id: self.id.to_string(),
                subnet_type: self.subnet_type.to_string(),
            }
            .into());
        }

        let tx_id = p::add_permissioned_subnet_validator(
            wallet,
            self.id,
            node_id,
            weight,
            start_time,
            end_time,
            check_acceptance,
        )
        .await?;

        Ok(AvalancheSubnetValidator {
            tx_id,
            node_id,
            subnet_id: self.id,
            start_time: start_time.timestamp() as u64,
            end_time: end_time.timestamp() as u64,
            weight: Some(weight),
            ..Default::default()
        })
    }

    /// Get the validator nodes signatures of a Warp message
    /// Tries to get the signatures from a provided number of the Subnet's validators
    /// If the number of validators is not provided, tries to get the signatures from all the Subnet's validators
    /// If the number of validators is provided, stops after reaching the said number of validators
    /// Note: for now, the validator nodes queried are the ones that are part of the Subnet at the current height
    pub fn get_warp_message_node_signatures(
        &self,
        warp_message: &WarpMessage,
        signatures_threshold: Option<u32>,
    ) -> Result<Vec<WarpMessageNodeSignature>, AshError> {
        let mut signatures = vec![];

        let source_chain = self.get_blockchain(warp_message.unsigned_message.source_chain_id)?;

        // Parse the RPC URL to get the scheme, host, and port
        let (scheme, endpoint_host, port, path, ..) =
            extract_scheme_host_port_path_chain_alias(&source_chain.rpc_url).map_err(|e| {
                RpcError::UrlParseFailure {
                    rpc_url: source_chain.rpc_url.to_string(),
                    msg: e.to_string(),
                }
            })?;
        let endpoint_scheme = scheme.unwrap_or("http".to_string());
        let endpoint_path = path.unwrap_or("/ext/bc/C/rpc".to_string());
        let endpoint_port = port.unwrap_or(9650);

        // Get the node information from the info endpoint
        let mut endpoint_node = AvalancheNode {
            http_host: endpoint_host.clone(),
            http_port: endpoint_port,
            https_enabled: matches!(endpoint_scheme.as_str(), "https"),
            ..Default::default()
        };
        endpoint_node.update_info()?;

        // Construct the RPC URL to query the info.peers endpoint
        let info_rpc_url = format!(
            "{}/{}",
            endpoint_node.get_http_endpoint(),
            info::AVAX_INFO_API_ENDPOINT
        );

        // Get the peers information from the info.peers endpoint (notably the nodes public IP addresses)
        let peers = info::peers(
            &info_rpc_url,
            Some(
                self.validators
                    .iter()
                    .map(|validator| validator.node_id)
                    .collect(),
            ),
        )?;

        // Until we have enough signatures or we have queried all the validators
        let validators_threshold = match signatures_threshold {
            Some(threshold) => threshold,
            None => self.validators.len() as u32,
        };
        let mut validators_index = 0_u32;
        while signatures.len() < validators_threshold as usize
            && validators_index < self.validators.len() as u32
        {
            // Get the validator node
            let validator = &self.validators[validators_index as usize];

            let signature = match validator.node_id {
                // If the validator node is the node being used as endpoint, get the signature from the node
                node_id if node_id == endpoint_node.id => subnet_evm::get_warp_signature(
                    &source_chain.rpc_url,
                    warp_message.unsigned_message.id,
                )?,
                // If the validator node is a peer of the node being used as endpoint
                _ => {
                    // Get the validator node's IP address
                    let peer = peers
                        .iter()
                        .find(|&peer| peer.node_id == validator.node_id)
                        .ok_or(AvalancheSubnetError::NotFound {
                            subnet_id: self.id.to_string(),
                            target_type: "validator node".to_string(),
                            target_value: validator.node_id.to_string(),
                        })?;

                    // Construct the RPC URL to query the warp_getSignature endpoint
                    let warp_rpc_url = format!(
                        "{}://{}:{}{}",
                        endpoint_scheme,
                        peer.public_ip.ip(),
                        peer.public_ip.port() - 1,
                        endpoint_path
                    );

                    // Get the validator node's signature for the Warp message
                    subnet_evm::get_warp_signature(&warp_rpc_url, warp_message.unsigned_message.id)?
                }
            };

            signatures.push(WarpMessageNodeSignature {
                node_id: validator.node_id,
                signature,
            });

            // Increment the validator index
            validators_index += 1;
        }

        Ok(signatures)
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
    // TODO: Store as DateTime::<Utc>?
    pub start_time: u64,
    pub end_time: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stake_amount: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub potential_reward: Option<u64>,
    pub connected: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_reward_owner: Option<AvalancheOutputOwners>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegator_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegator_weight: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegators: Option<Vec<AvalancheSubnetDelegator>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegation_fee: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub potential_reward: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    use crate::avalanche::AvalancheNetwork;
    use std::str::FromStr;

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
        let subnet = local_network
            .get_subnet(local_network.primary_network_id)
            .unwrap();

        let blockchain = subnet
            .get_blockchain(Id::from_str(NETWORK_RUNNER_CCHAIN_ID).unwrap())
            .unwrap();
        assert_eq!(blockchain.name, "C-Chain");
    }

    #[test]
    #[ignore]
    fn test_avalanche_subnet_get_blockchain_by_name() {
        let local_network = load_test_network();
        let subnet = local_network
            .get_subnet(local_network.primary_network_id)
            .unwrap();

        let blockchain = subnet.get_blockchain_by_name("C-Chain").unwrap();
        assert_eq!(
            blockchain.id,
            Id::from_str(NETWORK_RUNNER_CCHAIN_ID).unwrap()
        );
    }

    #[test]
    #[ignore]
    fn test_avalanche_subnet_get_validator() {
        let mut local_network = load_test_network();
        local_network
            .update_subnet_validators(local_network.primary_network_id)
            .unwrap();

        let subnet = local_network
            .get_subnet(local_network.primary_network_id)
            .unwrap();

        let validator = subnet
            .get_validator(NodeId::from_str(NETWORK_RUNNER_NODE_ID).unwrap())
            .unwrap();
        assert_eq!(
            validator.node_id,
            NodeId::from_str(NETWORK_RUNNER_NODE_ID).unwrap()
        );
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
        let network_subnet = local_network.get_subnet(created_subnet.id).unwrap();

        assert_eq!(&created_subnet, network_subnet);
    }

    #[async_std::test]
    #[serial_test::serial]
    #[ignore]
    async fn test_avalanche_subnet_add_validator_permissioned() {
        let local_network = load_test_network();
        let wallet = local_network
            .create_wallet_from_cb58(AVAX_EWOQ_PRIVATE_KEY)
            .unwrap();

        // Only test if adding a validator to the Primary Network fails
        // because adding a validator to a Subnet is too long and already tested
        let primary_network = local_network
            .get_subnet(local_network.primary_network_id)
            .unwrap()
            .clone();

        assert!(primary_network
            .add_validator_permissioned(
                &wallet,
                NodeId::from_str(NETWORK_RUNNER_NODE_ID).unwrap(),
                100,
                DateTime::<Utc>::from_str("2025-01-01T00:00:00.000Z").unwrap(),
                DateTime::<Utc>::from_str("2025-02-01T00:00:00.000Z").unwrap(),
                false
            )
            .await
            .is_err());
    }
}
