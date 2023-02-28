// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche Info API

use crate::avalanche::{
    avalanche_node_id_from_string, nodes::AvalancheNodeUptime, nodes::AvalancheNodeVersion,
};
use avalanche_types::{ids::node::Id, jsonrpc::info::*};
use serde::Deserialize;
use serde_aux::prelude::*;

/// Info API endpoint
pub const AVAX_INFO_API_ENDPOINT: &str = "ext/info";

#[derive(Deserialize)]
#[allow(dead_code)]
struct InfoApiGetNodeIdResponse {
    jsonrpc: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    id: u8,
    result: InfoApiGetNodeIdResult,
}

#[derive(Deserialize)]
struct InfoApiGetNodeIdResult {
    #[serde(rename = "nodeID", deserialize_with = "avalanche_node_id_from_string")]
    node_id: Id,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct InfoApiGetNodeIpResponse {
    jsonrpc: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    id: u8,
    result: InfoApiGetNodeIpResult,
}

#[derive(Deserialize)]
struct InfoApiGetNodeIpResult {
    ip: String,
}

// Get the ID of a node by querying the Info API
pub fn get_node_id(rpc_url: &str) -> Result<Id, ureq::Error> {
    let resp: InfoApiGetNodeIdResponse = ureq::post(rpc_url)
        .send_json(ureq::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "info.getNodeID",
            "params": {}
        }))?
        .into_json()?;

    Ok(resp.result.node_id)
}

// Get the IP of a node by querying the Info API
pub fn get_node_ip(rpc_url: &str) -> Result<String, ureq::Error> {
    let resp: InfoApiGetNodeIpResponse = ureq::post(rpc_url)
        .send_json(ureq::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "info.getNodeIP",
            "params": {}
        }))?
        .into_json()?;

    Ok(resp.result.ip)
}

// Get the version of a node by querying the Info API
pub fn get_node_version(rpc_url: &str) -> Result<AvalancheNodeVersion, ureq::Error> {
    let resp: GetNodeVersionResponse = ureq::post(rpc_url)
        .send_json(ureq::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "info.getNodeVersion",
            "params": {}
        }))?
        .into_json()?;

    let node_version = resp.result.unwrap();
    Ok(AvalancheNodeVersion {
        avalanchego_version: node_version.version,
        database_version: node_version.database_version,
        git_commit: node_version.git_commit,
        vm_versions: node_version.vm_versions,
    })
}

// Get the uptime of a node by querying the Info API
pub fn get_node_uptime(rpc_url: &str) -> Result<AvalancheNodeUptime, ureq::Error> {
    let resp: UptimeResponse = ureq::post(rpc_url)
        .send_json(ureq::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "info.uptime",
            "params": {}
        }))?
        .into_json()?;

    let node_uptime = resp.result.unwrap();
    Ok(AvalancheNodeUptime {
        rewarding_stake_percentage: node_uptime.rewarding_stake_percentage,
        weighted_average_percentage: node_uptime.weighted_average_percentage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use avalanche_types::jsonrpc::info::VmVersions;

    // Using avalanche-network-runner to run a test network
    const ASH_TEST_HTTP_HOST: &str = "127.0.0.1";
    const ASH_TEST_HTTP_PORT: u16 = 9650;
    const ASH_TEST_STACKING_PORT: u16 = 9651;
    const ASH_TEST_NODE_ID: &str = "NodeID-7Xhw2mDxuDS44j42TCB6U5579esbSt3Lg";

    #[test]
    #[ignore]
    fn test_get_node_id() {
        let rpc_url = format!(
            "http://{}:{}/{}",
            ASH_TEST_HTTP_HOST, ASH_TEST_HTTP_PORT, AVAX_INFO_API_ENDPOINT
        );
        let node_id = get_node_id(&rpc_url).unwrap();
        assert_eq!(node_id.to_string(), ASH_TEST_NODE_ID);
    }

    #[test]
    #[ignore]
    fn test_get_node_ip() {
        let rpc_url = format!(
            "http://{}:{}/{}",
            ASH_TEST_HTTP_HOST, ASH_TEST_HTTP_PORT, AVAX_INFO_API_ENDPOINT
        );
        let node_ip = get_node_ip(&rpc_url).unwrap();
        assert_eq!(
            node_ip,
            format!("{ASH_TEST_HTTP_HOST}:{ASH_TEST_STACKING_PORT}")
        );
    }

    #[test]
    #[ignore]
    fn test_get_node_version() {
        let rpc_url = format!(
            "http://{}:{}/{}",
            ASH_TEST_HTTP_HOST, ASH_TEST_HTTP_PORT, AVAX_INFO_API_ENDPOINT
        );
        let node_version = get_node_version(&rpc_url).unwrap();

        // Only check if the version is not empty
        assert!(!node_version.avalanchego_version.is_empty());
        assert!(!node_version.database_version.is_empty());
        assert!(!node_version.git_commit.is_empty());
        assert!(node_version.vm_versions != VmVersions::default());
    }

    #[test]
    #[ignore]
    fn test_get_node_uptime() {
        let rpc_url = format!(
            "http://{}:{}/{}",
            ASH_TEST_HTTP_HOST, ASH_TEST_HTTP_PORT, AVAX_INFO_API_ENDPOINT
        );
        let node_uptime = get_node_uptime(&rpc_url).unwrap();

        // Check if the uptime is > 0
        assert!(node_uptime.rewarding_stake_percentage > 0.0);
        assert!(node_uptime.weighted_average_percentage > 0.0);
    }
}
