// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche Info API

use crate::{
    avalanche::jsonrpc::{get_json_rpc_req_result, JsonRpcResponse},
    avalanche::nodes::{AvalancheNodeUptime, AvalancheNodeVersions},
    errors::*,
    impl_json_rpc_response,
};
use avalanche_types::{
    ids::node::Id,
    jsonrpc::{info::*, ResponseError},
};
use std::net::SocketAddr;

/// Info API endpoint
pub const AVAX_INFO_API_ENDPOINT: &str = "ext/info";

impl_json_rpc_response!(GetNodeIdResponse, GetNodeIdResult);
impl_json_rpc_response!(GetNodeIpResponse, GetNodeIpResult);
impl_json_rpc_response!(GetNodeVersionResponse, GetNodeVersionResult);
impl_json_rpc_response!(UptimeResponse, UptimeResult);

// Get the ID of a node by querying the Info API
pub fn get_node_id(rpc_url: &str) -> Result<Id, RpcError> {
    let node_id = get_json_rpc_req_result::<GetNodeIdResponse, GetNodeIdResult>(
        rpc_url,
        "info.getNodeID",
        None,
    )?
    .node_id;

    Ok(node_id)
}

// Get the IP of a node by querying the Info API
pub fn get_node_ip(rpc_url: &str) -> Result<SocketAddr, RpcError> {
    let ip = get_json_rpc_req_result::<GetNodeIpResponse, GetNodeIpResult>(
        rpc_url,
        "info.getNodeIP",
        None,
    )?
    .ip;

    Ok(ip)
}

// Get the version of a node by querying the Info API
pub fn get_node_version(rpc_url: &str) -> Result<AvalancheNodeVersions, RpcError> {
    let node_version = get_json_rpc_req_result::<GetNodeVersionResponse, GetNodeVersionResult>(
        rpc_url,
        "info.getNodeVersion",
        None,
    )?
    .into();

    Ok(node_version)
}

// Get the uptime of a node by querying the Info API
pub fn get_node_uptime(rpc_url: &str) -> Result<AvalancheNodeUptime, RpcError> {
    let uptime =
        get_json_rpc_req_result::<UptimeResponse, UptimeResult>(rpc_url, "info.uptime", None)?
            .into();

    Ok(uptime)
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::*;
    use avalanche_types::jsonrpc::info::VmVersions;

    // Using avalanche-network-runner to run a test network
    const ASH_TEST_HTTP_HOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
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
            SocketAddr::new(ASH_TEST_HTTP_HOST, ASH_TEST_STACKING_PORT)
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
