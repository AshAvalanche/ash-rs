// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche nodes

use crate::avalanche::{avalanche_node_id_from_string, jsonrpc::info::*};
use avalanche_types::{ids::node::Id, jsonrpc::info::VmVersions};
use serde::{Deserialize, Serialize};

/// Avalanche node
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheNode {
    #[serde(deserialize_with = "avalanche_node_id_from_string")]
    pub id: Id,
    pub http_host: String,
    pub http_port: u16,
    pub public_ip: String,
    pub stacking_port: u16,
    pub versions: AvalancheNodeVersions,
    pub uptime: AvalancheNodeUptime,
}

/// Avalanche node version
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheNodeVersions {
    pub avalanchego_version: String,
    pub database_version: String,
    pub git_commit: String,
    pub vm_versions: VmVersions,
    // Not yet implemented in avalanche_types
    // pub rpc_protocol_version: String,
}

/// Avalanche node uptime
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheNodeUptime {
    pub rewarding_stake_percentage: f64,
    pub weighted_average_percentage: f64,
}

impl AvalancheNode {
    /// Update the node's information
    pub fn update_info(&mut self) -> Result<(), String> {
        let api_path = format!(
            "http://{0}:{1}/{AVAX_INFO_API_ENDPOINT}",
            self.http_host, self.http_port
        );

        self.id = get_node_id(&api_path).map_err(|e| e.to_string())?;

        // The get_node_ip() return has to be splited to get public_ip and stacking_port
        let node_ip = get_node_ip(&api_path).map_err(|e| e.to_string())?;
        let node_ip_split: Vec<&str> = node_ip.split(':').collect();
        self.public_ip = node_ip_split[0].to_string();
        self.stacking_port = node_ip_split[1].parse().unwrap();

        self.versions = get_node_version(&api_path).map_err(|e| e.to_string())?;
        self.uptime = get_node_uptime(&api_path).map_err(|e| e.to_string())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Using avalanche-network-runner to run a test network
    const ASH_TEST_HTTP_HOST: &str = "127.0.0.1";
    const ASH_TEST_HTTP_PORT: u16 = 9650;
    const ASH_TEST_STACKING_PORT: u16 = 9651;
    const ASH_TEST_NODE_ID: &str = "NodeID-7Xhw2mDxuDS44j42TCB6U5579esbSt3Lg";

    #[test]
    #[ignore]
    fn test_update_info() {
        let mut node = AvalancheNode {
            http_host: ASH_TEST_HTTP_HOST.to_string(),
            http_port: ASH_TEST_HTTP_PORT,
            ..Default::default()
        };

        // Test that the node has the right http_host and http_port
        assert_eq!(node.http_host, ASH_TEST_HTTP_HOST);
        assert_eq!(node.http_port, ASH_TEST_HTTP_PORT);

        node.update_info().unwrap();

        // Test the node id, public_ip and stacking_port
        assert_eq!(node.id.to_string(), ASH_TEST_NODE_ID);
        assert_eq!(node.public_ip, ASH_TEST_HTTP_HOST);
        assert_eq!(node.stacking_port, ASH_TEST_STACKING_PORT);

        // Only test that the node version is not empty
        assert!(!node.versions.avalanchego_version.is_empty());
        assert!(!node.versions.database_version.is_empty());
        assert!(!node.versions.git_commit.is_empty());
        assert!(node.versions.vm_versions != VmVersions::default());

        // Test that the node uptime is not equal to 0
        assert_ne!(node.uptime.rewarding_stake_percentage, 0.0);
        assert_ne!(node.uptime.weighted_average_percentage, 0.0);
    }
}
