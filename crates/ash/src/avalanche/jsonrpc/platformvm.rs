// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

// Module that contains code to interact with Avalanche PlatformVM blockchains

use crate::avalanche::avalanche_id_from_string;
use crate::avalanche::subnets::AvalancheSubnet;
use avalanche_types::ids::Id;
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
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

#[cfg(test)]
mod tests {
    use super::*;
    use avalanche_types::ids::Id;
    use std::str::FromStr;

    const AVAX_PRIMARY_NETWORK_ID: &str = "11111111111111111111111111111111LpoYY";

    #[test]
    fn test_get_network_subnets() {
        let rpc_url = "https://api.avax-test.network/ext/bc/P";
        let subnets = get_network_subnets(rpc_url).unwrap();

        // Test that the primary network subnet is present
        assert!(subnets
            .iter()
            .any(|subnet| subnet.id == Id::from_str(AVAX_PRIMARY_NETWORK_ID).unwrap()));
    }
}
