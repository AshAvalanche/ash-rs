// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

// Module that contains code to interact with Avalanche PlatformVM blockchains

use crate::avalanche::avalanche_id_from_string;
use crate::avalanche::{blockchains::AvalancheBlockchain, subnets::AvalancheSubnet};
use avalanche_types::ids::Id;
use serde::Deserialize;
use serde_aux::prelude::*;
use ureq;

#[derive(Deserialize)]
#[allow(dead_code)]
struct PlatformApiGetSubnetsResponse {
    jsonrpc: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    id: u8,
    result: PlatformApiGetSubnetsResponseResult,
}

#[derive(Deserialize)]
struct PlatformApiGetSubnetsResponseResult {
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
    result: PlatformApiGetBlockchainsResponseResult,
}

#[derive(Deserialize)]
struct PlatformApiGetBlockchainsResponseResult {
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
            blockchains: vec![],
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

#[cfg(test)]
mod tests {
    use super::*;
    use avalanche_types::ids::Id;
    use std::str::FromStr;

    const AVAX_PRIMARY_NETWORK_ID: &str = "11111111111111111111111111111111LpoYY";
    const AVAX_MAINNET_CCHAIN_ID: &str = "2q9e4r6Mu3U68nU1fYjgbR6JvwrRx36CohpAX5UQxse55x1Q5";
    const AVAX_MAINNET_XCHAIN_ID: &str = "2oYMBNV4eNHyqk2fjjV5nVQLDbtmNJzq5s3qs3Lo6ftnC6FByM";

    #[test]
    fn test_get_network_subnets() {
        let rpc_url = "https://api.avax.network/ext/bc/P";
        let subnets = get_network_subnets(rpc_url).unwrap();

        // Test that the primary network subnet is present
        assert!(subnets
            .iter()
            .any(|subnet| subnet.id == Id::from_str(AVAX_PRIMARY_NETWORK_ID).unwrap()));
    }

    #[test]
    fn test_get_network_blockchains() {
        let rpc_url = "https://api.avax.network/ext/bc/P";
        let blockchains = get_network_blockchains(rpc_url).unwrap();

        // Test that the C-Chain and X-Chain are present
        let c_chain = blockchains
            .iter()
            .find(|blockchain| blockchain.id == Id::from_str(AVAX_MAINNET_CCHAIN_ID).unwrap())
            .unwrap();
        let x_chain = blockchains
            .iter()
            .find(|blockchain| blockchain.id == Id::from_str(AVAX_MAINNET_XCHAIN_ID).unwrap())
            .unwrap();

        assert_eq!(c_chain.name, "C-Chain");
        assert_eq!(x_chain.name, "X-Chain");
    }
}
