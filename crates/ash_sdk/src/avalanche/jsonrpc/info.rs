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
    ids::node::Id as NodeId,
    jsonrpc::{info::*, ResponseError},
    key::bls::ProofOfPossession,
};
use std::net::SocketAddr;

/// Info API endpoint
pub const AVAX_INFO_API_ENDPOINT: &str = "ext/info";

impl_json_rpc_response!(GetNodeIdResponse, GetNodeIdResult);
impl_json_rpc_response!(GetNodeIpResponse, GetNodeIpResult);
impl_json_rpc_response!(GetNodeVersionResponse, GetNodeVersionResult);
impl_json_rpc_response!(UptimeResponse, UptimeResult);
impl_json_rpc_response!(GetNetworkNameResponse, GetNetworkNameResult);
impl_json_rpc_response!(IsBootstrappedResponse, IsBootstrappedResult);
impl_json_rpc_response!(PeersResponse, PeersResult);

/// Get the ID of a node by querying the Info API
pub fn get_node_id(rpc_url: &str) -> Result<(NodeId, Option<ProofOfPossession>), RpcError> {
    let node_id = get_json_rpc_req_result::<GetNodeIdResponse, GetNodeIdResult>(
        rpc_url,
        "info.getNodeID",
        None,
    )?;

    Ok((node_id.node_id, node_id.node_pop))
}

/// Get the IP of a node by querying the Info API
pub fn get_node_ip(rpc_url: &str) -> Result<SocketAddr, RpcError> {
    let ip = get_json_rpc_req_result::<GetNodeIpResponse, GetNodeIpResult>(
        rpc_url,
        "info.getNodeIP",
        None,
    )?
    .ip;

    Ok(ip)
}

/// Get the version of a node by querying the Info API
pub fn get_node_version(rpc_url: &str) -> Result<AvalancheNodeVersions, RpcError> {
    let node_version = get_json_rpc_req_result::<GetNodeVersionResponse, GetNodeVersionResult>(
        rpc_url,
        "info.getNodeVersion",
        None,
    )?
    .into();

    Ok(node_version)
}

/// Get the uptime of a node by querying the Info API
pub fn get_node_uptime(rpc_url: &str) -> Result<AvalancheNodeUptime, RpcError> {
    let uptime =
        get_json_rpc_req_result::<UptimeResponse, UptimeResult>(rpc_url, "info.uptime", None)?
            .into();

    Ok(uptime)
}

/// Get the name of the network a node is participating in by querying the Info API
pub fn get_network_name(rpc_url: &str) -> Result<String, RpcError> {
    let network_name = get_json_rpc_req_result::<GetNetworkNameResponse, GetNetworkNameResult>(
        rpc_url,
        "info.getNetworkName",
        None,
    )?
    .network_name;

    Ok(network_name)
}

/// Check if a given chain is done boostrapping by querying the Info API
/// `chain` is the chain ID or alias of the chain to check
pub fn is_bootstrapped(rpc_url: &str, chain: &str) -> Result<bool, RpcError> {
    let is_bootstrapped = get_json_rpc_req_result::<IsBootstrappedResponse, IsBootstrappedResult>(
        rpc_url,
        "info.isBootstrapped",
        Some(ureq::json!({
            "chain": chain.to_string()
        })),
    )?
    .is_bootstrapped;

    Ok(is_bootstrapped)
}

/// Get the peers of a node by querying the Info API
pub fn peers(rpc_url: &str, node_ids: Option<Vec<NodeId>>) -> Result<Vec<Peer>, RpcError> {
    let peers = get_json_rpc_req_result::<PeersResponse, PeersResult>(
        rpc_url,
        "info.peers",
        Some(ureq::json!({
            "nodeIDs": node_ids.or(Some(vec![]))
        })),
    )?
    .peers
    .unwrap_or(vec![]);

    Ok(peers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        net::{IpAddr, Ipv4Addr},
        str::FromStr,
    };

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
        let (node_id, _) = get_node_id(&rpc_url).unwrap();
        assert_eq!(node_id, NodeId::from_str(ASH_TEST_NODE_ID).unwrap());
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

    #[test]
    #[ignore]
    fn test_peers() {
        let rpc_url = format!(
            "http://{}:{}/{}",
            ASH_TEST_HTTP_HOST, ASH_TEST_HTTP_PORT, AVAX_INFO_API_ENDPOINT
        );
        let all_peers = peers(&rpc_url, None).unwrap();

        // Check that the node has 4 peers (number of nodes in the test network)
        assert!(all_peers.len() == 4);

        // Check that the node_ids filter works
        // Expected node ID: NodeID-MFrZFVCXPv5iCn6M9K6XduxGTYp891xXZ
        let node_ids = vec![NodeId::from_str("NodeID-MFrZFVCXPv5iCn6M9K6XduxGTYp891xXZ").unwrap()];
        let filtered_peers = peers(&rpc_url, Some(node_ids)).unwrap();

        // Check that the node has 1 peer
        assert!(filtered_peers.len() == 1);
        assert!(
            filtered_peers[0].node_id
                == NodeId::from_str("NodeID-MFrZFVCXPv5iCn6M9K6XduxGTYp891xXZ").unwrap()
        );
    }
}
