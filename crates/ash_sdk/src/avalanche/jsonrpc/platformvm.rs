// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche PlatformVM API

use crate::avalanche::{
    blockchains::AvalancheBlockchain,
    jsonrpc::{get_json_rpc_req_result, JsonRpcResponse},
    subnets::{AvalancheSubnet, AvalancheSubnetValidator},
};
use crate::{errors::*, impl_json_rpc_response};
use avalanche_types::{
    ids::Id,
    jsonrpc::{platformvm::*, ResponseError},
};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;

/// Subnet with control keys as addresses
/// This is done to avoid having to retransform the control keys to addresses later
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubnetStringControlKeys {
    pub id: Id,
    #[serde(default)]
    pub control_keys: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_number_from_string")]
    pub threshold: u32,
}

/// GetSubnetsResult without deserializing the control keys to ShortIds
/// This is done to avoid having to retransform the control keys to addresses later
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSubnetsResultStringControlKeys {
    pub subnets: Option<Vec<SubnetStringControlKeys>>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct GetSubnetsResponseStringControlKeys {
    pub jsonrpc: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: u32,
    pub result: Option<GetSubnetsResultStringControlKeys>,
    pub error: Option<ResponseError>,
}

impl_json_rpc_response!(
    GetSubnetsResponseStringControlKeys,
    GetSubnetsResultStringControlKeys
);
impl_json_rpc_response!(GetBlockchainsResponse, GetBlockchainsResult);
impl_json_rpc_response!(GetCurrentValidatorsResponse, GetCurrentValidatorsResult);

/// Get the Subnets of the network by querying the P-Chain API
pub fn get_network_subnets(
    rpc_url: &str,
    network_name: &str,
) -> Result<Vec<AvalancheSubnet>, RpcError> {
    let network_subnets = get_json_rpc_req_result::<
        GetSubnetsResponseStringControlKeys,
        GetSubnetsResultStringControlKeys,
    >(rpc_url, "platform.getSubnets", None)?
    .subnets
    .ok_or(RpcError::GetFailure {
        data_type: "subnets".to_string(),
        target_type: "network".to_string(),
        target_value: network_name.to_string(),
        msg: "No subnets found".to_string(),
    })?
    .into_iter()
    .map(Into::into)
    .collect();

    Ok(network_subnets)
}

/// Get the blockchains of the network by querying the P-Chain API
pub fn get_network_blockchains(
    rpc_url: &str,
    network_name: &str,
) -> Result<Vec<AvalancheBlockchain>, RpcError> {
    let network_blockchains = get_json_rpc_req_result::<
        GetBlockchainsResponse,
        GetBlockchainsResult,
    >(rpc_url, "platform.getBlockchains", None)?
    .blockchains
    .ok_or(RpcError::GetFailure {
        data_type: "blockchains".to_string(),
        target_type: "network".to_string(),
        target_value: network_name.to_string(),
        msg: "No blockchains found".to_string(),
    })?
    .into_iter()
    .map(Into::into)
    .collect();

    Ok(network_blockchains)
}

/// Get the current validators of a Subnet by querying the P-Chain API
pub fn get_current_validators(
    rpc_url: &str,
    subnet_id: Id,
) -> Result<Vec<AvalancheSubnetValidator>, RpcError> {
    let current_validators =
        get_json_rpc_req_result::<GetCurrentValidatorsResponse, GetCurrentValidatorsResult>(
            rpc_url,
            "platform.getCurrentValidators",
            Some(ureq::json!({ "subnetID": subnet_id.to_string() })),
        )?
        .validators
        .ok_or(RpcError::GetFailure {
            data_type: "validators".to_string(),
            target_type: "Subnet".to_string(),
            target_value: subnet_id.to_string(),
            msg: "No validators found".to_string(),
        })?
        .iter()
        .map(|validator| AvalancheSubnetValidator::from_api_primary_validator(validator, subnet_id))
        .collect();

    Ok(current_validators)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avalanche::AvalancheNetwork;
    use avalanche_types::ids::node::Id as NodeId;
    use std::{env, str::FromStr};

    const AVAX_PRIMARY_NETWORK_ID: &str = "11111111111111111111111111111111LpoYY";
    const AVAX_FUJI_CCHAIN_ID: &str = "yH8D7ThNJkxmtkuv2jgBa4P1Rn3Qpr4pPr7QYNfcdoS6k6HWp";
    const AVAX_FUJI_XCHAIN_ID: &str = "2JVSBoinj9C2J33VntvzYtVJNZdN2NKiwwKjcumHUWEb5DbBrm";
    // ID of a node operated by Ava Labs
    const AVAX_FUJI_NODE_ID: &str = "NodeID-4KXitMCoE9p2BHA6VzXtaTxLoEjNDo2Pt";

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

        let subnets = get_network_subnets(rpc_url, &fuji.name).unwrap();

        // Test that the Primary Network Subnet is present
        assert!(subnets
            .iter()
            .any(|subnet| subnet.id == Id::from_str(AVAX_PRIMARY_NETWORK_ID).unwrap()));
    }

    #[test]
    fn test_get_network_blockchains() {
        let fuji = load_test_network();
        let rpc_url = &fuji.get_pchain().unwrap().rpc_url;

        let blockchains = get_network_blockchains(rpc_url, &fuji.name).unwrap();

        // Test that the C-Chain and X-Chain are present
        let cchain = blockchains
            .iter()
            .find(|blockchain| blockchain.id == Id::from_str(AVAX_FUJI_CCHAIN_ID).unwrap())
            .unwrap();
        let xchain = blockchains
            .iter()
            .find(|blockchain| blockchain.id == Id::from_str(AVAX_FUJI_XCHAIN_ID).unwrap())
            .unwrap();

        assert_eq!(cchain.name, "C-Chain");
        assert_eq!(xchain.name, "X-Chain");
    }

    #[test]
    fn test_get_current_validators() {
        // The method platform.getCurrentValidators is not available on QuickNode
        // Tempoary workaround: use Ankr public endpoint
        let fuji = AvalancheNetwork::load("fuji-ankr", None).unwrap();
        let rpc_url = &fuji.get_pchain().unwrap().rpc_url;

        let validators = get_current_validators(rpc_url, fuji.primary_network_id).unwrap();

        // Test that the node operated by Ava Labs is present
        // Should not fail if the node is present
        let ava_labs_node = validators
            .iter()
            .find(|validator| validator.node_id == NodeId::from_str(AVAX_FUJI_NODE_ID).unwrap())
            .unwrap();

        // Test that the node has a non-zero uptime
        assert!(ava_labs_node.uptime > Some(0.0));
        // Test that the node has a non-zero weight
        assert!(ava_labs_node.weight > Some(0));
        // Test that the node has a non-zero potential reward
        assert!(ava_labs_node.potential_reward > Some(0));
        // Test that the node has a non-zero delegation fee
        assert!(ava_labs_node.delegation_fee > Some(0.0));
    }
}
