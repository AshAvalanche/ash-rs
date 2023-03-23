// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche PlatformVM API

use crate::avalanche::blockchains::AvalancheBlockchain;
use crate::avalanche::subnets::{
    AvalancheSubnet, AvalancheSubnetDelegator, AvalancheSubnetValidator,
};
use crate::avalanche::{
    avalanche_id_from_string, avalanche_node_id_from_string, AvalancheOutputOwners,
};
use avalanche_types::{ids::node::Id as NodeId, ids::Id};
use serde::Deserialize;
use serde_aux::prelude::*;
use std::str::FromStr;
use ureq;

#[derive(Deserialize)]
#[allow(dead_code)]
struct PlatformApiGetSubnetsResponse {
    jsonrpc: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    id: u8,
    result: PlatformApiGetSubnetsResult,
}

#[derive(Deserialize)]
struct PlatformApiGetSubnetsResult {
    subnets: Vec<PlatformApiSubnet>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlatformApiSubnet {
    #[serde(deserialize_with = "avalanche_id_from_string")]
    id: Id,
    control_keys: Vec<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    threshold: u8,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct PlatformApiGetBlockchainsResponse {
    jsonrpc: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    id: u8,
    result: PlatformApiGetBlockchainsResult,
}

#[derive(Deserialize)]
struct PlatformApiGetBlockchainsResult {
    blockchains: Vec<PlatformApiBlockchain>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlatformApiBlockchain {
    #[serde(deserialize_with = "avalanche_id_from_string")]
    id: Id,
    name: String,
    #[serde(alias = "subnetID", deserialize_with = "avalanche_id_from_string")]
    subnet_id: Id,
    #[serde(alias = "vmID", deserialize_with = "avalanche_id_from_string")]
    vm_id: Id,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct PlatformApiGetCurrentValidatorsResponse {
    jsonrpc: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    id: u8,
    result: PlatformApiGetCurrentValidatorsResult,
}

#[derive(Deserialize)]
struct PlatformApiGetCurrentValidatorsResult {
    validators: Vec<PlatformApiValidator>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct PlatformApiValidator {
    #[serde(rename = "txID", deserialize_with = "avalanche_id_from_string")]
    tx_id: Id,
    #[serde(rename = "nodeID", deserialize_with = "avalanche_node_id_from_string")]
    node_id: NodeId,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    start_time: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    end_time: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    stake_amount: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    weight: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    potential_reward: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    delegation_fee: f32,
    connected: bool,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    uptime: f32,
    validation_reward_owner: PlatformApiRewardOwner,
    delegators: Vec<PlatformApiDelegator>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    delegator_count: u32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    delegator_weight: u64,
    delegation_reward_owner: PlatformApiRewardOwner,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlatformApiDelegator {
    #[serde(rename = "txID", deserialize_with = "avalanche_id_from_string")]
    tx_id: Id,
    #[serde(rename = "nodeID", deserialize_with = "avalanche_node_id_from_string")]
    node_id: NodeId,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    start_time: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    end_time: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    stake_amount: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    weight: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    potential_reward: u64,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlatformApiRewardOwner {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    locktime: u64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    threshold: u32,
    addresses: Vec<String>,
}

// Get the Subnets of the network by querying the P-Chain API
pub fn get_network_subnets(rpc_url: &str) -> Result<Vec<AvalancheSubnet>, ureq::Error> {
    let resp: PlatformApiGetSubnetsResponse = ureq::post(rpc_url)
        .send_json(ureq::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "platform.getSubnets",
            "params": {}
        }))?
        .into_json()?;

    let network_subnets = resp
        .result
        .subnets
        .iter()
        .map(|subnet| AvalancheSubnet {
            id: subnet.id,
            control_keys: subnet.control_keys.clone(),
            threshold: subnet.threshold,
            ..Default::default()
        })
        .collect();

    Ok(network_subnets)
}

// Get the blockchains of the network by querying the P-Chain API
pub fn get_network_blockchains(rpc_url: &str) -> Result<Vec<AvalancheBlockchain>, ureq::Error> {
    let resp: PlatformApiGetBlockchainsResponse = ureq::post(rpc_url)
        .send_json(ureq::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "platform.getBlockchains",
            "params": {}
        }))?
        .into_json()?;

    let network_blockchains = resp
        .result
        .blockchains
        .iter()
        .map(|blockchain| AvalancheBlockchain {
            id: blockchain.id,
            name: blockchain.name.clone(),
            subnet_id: blockchain.subnet_id,
            vm_id: blockchain.vm_id,
            vm_type: String::new(),
            rpc_url: String::new(),
        })
        .collect();

    Ok(network_blockchains)
}

// Get the current validators of a Subnet by querying the P-Chain API
pub fn get_current_validators(
    rpc_url: &str,
    subnet_id: &str,
) -> Result<Vec<AvalancheSubnetValidator>, ureq::Error> {
    let resp: PlatformApiGetCurrentValidatorsResponse = ureq::post(rpc_url)
        .send_json(ureq::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "platform.getCurrentValidators",
            "params": {
                "subnetID": subnet_id
            }
        }))?
        .into_json()?;

    let current_validators = resp
        .result
        .validators
        .iter()
        .map(|validator| AvalancheSubnetValidator {
            tx_id: validator.tx_id,
            node_id: validator.node_id,
            subnet_id: Id::from_str(subnet_id).unwrap(),
            start_time: validator.start_time,
            end_time: validator.end_time,
            stake_amount: validator.stake_amount,
            weight: validator.weight,
            potential_reward: validator.potential_reward,
            delegation_fee: validator.delegation_fee,
            connected: validator.connected,
            uptime: validator.uptime,
            validation_reward_owner: AvalancheOutputOwners {
                locktime: validator.validation_reward_owner.locktime,
                threshold: validator.validation_reward_owner.threshold,
                addresses: validator.validation_reward_owner.addresses.clone(),
            },
            delegators: validator
                .delegators
                .iter()
                .map(|delegator| AvalancheSubnetDelegator {
                    tx_id: delegator.tx_id,
                    node_id: delegator.node_id,
                    start_time: delegator.start_time,
                    end_time: delegator.end_time,
                    stake_amount: delegator.stake_amount,
                    weight: delegator.weight,
                    potential_reward: delegator.potential_reward,
                    ..Default::default()
                })
                .collect(),
            delegator_count: validator.delegator_count,
            delegator_weight: validator.delegator_weight,
            delegation_reward_owner: AvalancheOutputOwners {
                locktime: validator.delegation_reward_owner.locktime,
                threshold: validator.delegation_reward_owner.threshold,
                addresses: validator.delegation_reward_owner.addresses.clone(),
            },
        })
        .collect();

    Ok(current_validators)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avalanche::AvalancheNetwork;
    use std::env;

    const AVAX_PRIMARY_NETWORK_ID: &str = "11111111111111111111111111111111LpoYY";
    const AVAX_FUJI_CCHAIN_ID: &str = "yH8D7ThNJkxmtkuv2jgBa4P1Rn3Qpr4pPr7QYNfcdoS6k6HWp";
    const AVAX_FUJI_XCHAIN_ID: &str = "2JVSBoinj9C2J33VntvzYtVJNZdN2NKiwwKjcumHUWEb5DbBrm";
    // ID of a node operated by Ava Labs
    const AVAX_FUJI_NODE_ID: &str = " NodeID-JjvzhxnLHLUQ5HjVRkvG827ivbLXPwA9u ";

    // Load the test network from the ASH_TEST_CONFIG file
    fn load_test_network() -> AvalancheNetwork {
        let config_path =
            env::var("ASH_TEST_AVAX_CONFIG").unwrap_or("tests/conf/default.yml".to_string());
        AvalancheNetwork::load("fuji", Some(&config_path)).unwrap()
    }

    #[test]
    fn test_get_network_subnets() {
        let fuji = load_test_network();
        let rpc_url = &fuji.get_pchain().unwrap().rpc_url;

        let subnets = get_network_subnets(rpc_url).unwrap();

        // Test that the primary network subnet is present
        assert!(subnets
            .iter()
            .any(|subnet| subnet.id == Id::from_str(AVAX_PRIMARY_NETWORK_ID).unwrap()));
    }

    #[test]
    fn test_get_network_blockchains() {
        let fuji = load_test_network();
        let rpc_url = &fuji.get_pchain().unwrap().rpc_url;

        let blockchains = get_network_blockchains(rpc_url).unwrap();

        // Test that the C-Chain and X-Chain are present
        let c_chain = blockchains
            .iter()
            .find(|blockchain| blockchain.id == Id::from_str(AVAX_FUJI_CCHAIN_ID).unwrap())
            .unwrap();
        let x_chain = blockchains
            .iter()
            .find(|blockchain| blockchain.id == Id::from_str(AVAX_FUJI_XCHAIN_ID).unwrap())
            .unwrap();

        assert_eq!(c_chain.name, "C-Chain");
        assert_eq!(x_chain.name, "X-Chain");
    }

    #[test]
    fn test_get_current_validators() {
        // The method platform.getCurrentValidators is not available on QuickNode
        // Tempoary workaround: use Ankr public endpoint
        let fuji = AvalancheNetwork::load("fuji-ankr", None).unwrap();
        let rpc_url = &fuji.get_pchain().unwrap().rpc_url;

        let validators = get_current_validators(rpc_url, AVAX_PRIMARY_NETWORK_ID).unwrap();

        // Test that the node operated by Ava Labs is present
        // Should not fail if the node is present
        let ava_labs_node = validators
            .iter()
            .find(|validator| validator.node_id == NodeId::from_str(AVAX_FUJI_NODE_ID).unwrap())
            .unwrap();

        // Test that the node is connected
        assert!(ava_labs_node.connected);
        // Test that the node has a non-zero uptime
        assert!(ava_labs_node.uptime > 0.0);
        // Test that the node has a non-zero weight
        assert!(ava_labs_node.weight > 0);
        // Test that the node has a non-zero potential reward
        assert!(ava_labs_node.potential_reward > 0);
        // Test that the node has a non-zero delegation fee
        assert!(ava_labs_node.delegation_fee > 0.0);
    }
}
