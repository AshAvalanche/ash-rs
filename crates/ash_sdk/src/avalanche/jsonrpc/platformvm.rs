// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche PlatformVM API

use crate::avalanche::{
    avalanche_id_from_string,
    blockchains::AvalancheBlockchain,
    jsonrpc::{get_json_rpc_req_result, JsonRpcResponse},
    subnets::{AvalancheSubnet, AvalancheSubnetValidator},
};
use crate::errors::*;
use avalanche_types::{
    ids::Id,
    jsonrpc::{
        platformvm::{GetCurrentValidatorsResponse, GetCurrentValidatorsResult},
        ResponseError,
    },
};
use serde::Deserialize;
use serde_aux::prelude::*;
use std::str::FromStr;
use ureq;

impl JsonRpcResponse<GetCurrentValidatorsResponse, GetCurrentValidatorsResult>
    for GetCurrentValidatorsResponse
{
    fn get_error(&self) -> Option<ResponseError> {
        self.error.clone()
    }

    fn get_result(&self) -> Option<GetCurrentValidatorsResult> {
        self.result.clone()
    }
}

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
) -> Result<Vec<AvalancheSubnetValidator>, RpcError> {
    let current_validators =
        get_json_rpc_req_result::<GetCurrentValidatorsResponse, GetCurrentValidatorsResult>(
            rpc_url,
            "platform.getCurrentValidators",
            Some(ureq::json!({ "subnetID": subnet_id })),
        )?
        .validators
        .ok_or(RpcError::GetFailure {
            data_type: "validators".to_string(),
            target_type: "Subnet".to_string(),
            target_value: subnet_id.to_string(),
            msg: "No validators found".to_string(),
        })?
        .iter()
        .map(|validator| {
            AvalancheSubnetValidator::from_api_primary_validator(
                validator,
                // Unwrap is safe because we checked for a response error above
                Id::from_str(subnet_id).unwrap(),
            )
        })
        .collect();

    Ok(current_validators)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avalanche::AvalancheNetwork;
    use avalanche_types::ids::node::Id as NodeId;
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
        assert!(ava_labs_node.weight > Some(0));
        // Test that the node has a non-zero potential reward
        assert!(ava_labs_node.potential_reward > Some(0));
        // Test that the node has a non-zero delegation fee
        assert!(ava_labs_node.delegation_fee > Some(0.0));
    }
}
