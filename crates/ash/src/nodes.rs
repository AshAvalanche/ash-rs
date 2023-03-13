// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Ash nodes

use crate::avalanche::nodes::AvalancheNode;
use crate::errors::*;
use avalanche_types::ids::node::Id;
use regex::Regex;
use serde::Serialize;
use std::str::FromStr;

/// Node of the Ash protocol
#[derive(Debug)]
pub struct AshNode {
    /// The Avalanche node
    pub avalanche_node: AvalancheNode,
}

impl AshNode {
    /// Create a new Ash node from an Avalanche node ID string
    pub fn from_cb58_id(node_id: &str) -> Result<Self, AshError> {
        let id = Id::from_str(node_id).map_err(|e| AshNodeError::InvalidId {
            id: node_id.to_string(),
            msg: e.to_string(),
        })?;

        Ok(AshNode {
            avalanche_node: AvalancheNode {
                id,
                ..Default::default()
            },
        })
    }

    /// Create a new Ash node from an Avalanche node ID byte slice
    pub fn from_bytes_id(node_id: &[u8]) -> Result<Self, AshError> {
        if node_id.len() != 20 {
            return Err(AshNodeError::InvalidId {
                id: hex::encode(node_id),
                msg: "should be 20 bytes long".to_string(),
            }
            .into());
        }

        let id = Id::from_slice(node_id);

        Ok(AshNode {
            avalanche_node: AvalancheNode {
                id,
                ..Default::default()
            },
        })
    }

    /// Create a new Ash node from an Avalanche node ID hex string
    pub fn from_hex_id(node_id: &str) -> Result<Self, AshError> {
        // Convert the hex string to a byte slice
        let nodeid = hex::decode(node_id);

        match nodeid {
            Ok(nodeid) => AshNode::from_bytes_id(&nodeid),
            Err(e) => Err(AshNodeError::InvalidId {
                id: node_id.to_string(),
                msg: e.to_string(),
            }
            .into()),
        }
    }

    /// Create a new Ash node from a string
    /// Try to find out the node ID format using a regex
    pub fn from_string(node_id: &str) -> Result<Self, AshError> {
        // Check if the node ID is a valid CB58 string
        let re = Regex::new(r"^(NodeID-)?[A-Za-z0-9]{32,33}$").unwrap();

        if re.is_match(node_id) {
            return AshNode::from_cb58_id(node_id);
        }

        // Check if the node ID is a valid hex string
        let re = Regex::new(r"^(0x)?[0-9a-fA-F]{40}$").unwrap();

        if re.is_match(node_id) {
            return AshNode::from_hex_id(node_id.trim_start_matches("0x"));
        }

        Err(AshNodeError::InvalidId {
            id: nodeid.to_string(),
            msg: "unknown node ID format".to_string(),
        }
        .into())
    }

    /// Get the node's ID as an AshNodeId struct
    pub fn id(&self) -> AshNodeId {
        AshNodeId {
            p_chain: self.avalanche_node.id.to_string(),
            cb58: self.avalanche_node.id.short_id().to_string(),
            hex: self
                .avalanche_node
                .id
                .as_ref()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect(),
            bytes: self.avalanche_node.id.to_vec(),
        }
    }

    /// Get the node's info as an AshNodeInfo struct
    pub fn info(&self) -> AshNodeInfo {
        AshNodeInfo { id: self.id() }
    }
}

/// Ash node info
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AshNodeInfo {
    /// The node's ID
    pub id: AshNodeId,
}

/// Ash node ID
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AshNodeId {
    /// The node's ID as a P-Chain string
    pub p_chain: String,
    /// The node's ID as a CB58 string
    pub cb58: String,
    /// The node's ID as a hex string
    pub hex: String,
    /// The node's ID as a byte slice
    pub bytes: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const CB58_ID: &str = "FhFWdWodxktJYq884nrJjWD8faLTk9jmp";
    const HEX_ID: &str = "a12e8332e9ad69ae9f70a3d5715adfccdf05bc65";
    const BYTES_ID: [u8; 20] = [
        161, 46, 131, 50, 233, 173, 105, 174, 159, 112, 163, 213, 113, 90, 223, 204, 223, 5, 188,
        101,
    ];

    #[test]
    fn create_from_cb58_id() {
        // Creating the node should succeed
        let node = AshNode::from_cb58_id(CB58_ID).unwrap();

        assert_eq!(node.avalanche_node.id.short_id().to_string(), CB58_ID);
    }

    #[test]
    fn create_from_bytes_id() {
        // Creating the node should succeed
        let node = AshNode::from_bytes_id(&BYTES_ID).unwrap();

        assert_eq!(node.avalanche_node.id.short_id().to_string(), CB58_ID);
    }

    #[test]
    fn create_from_hex_id() {
        // Creating the node should succeed
        let node = AshNode::from_hex_id(HEX_ID).unwrap();

        assert_eq!(node.avalanche_node.id.short_id().to_string(), CB58_ID);
    }

    #[test]
    fn create_from_string() {
        // Creating the node should succeed
        let node = AshNode::from_string(CB58_ID).unwrap();

        assert_eq!(node.avalanche_node.id.short_id().to_string(), CB58_ID);

        // Creating the node should succeed
        let node = AshNode::from_string(HEX_ID).unwrap();

        assert_eq!(node.avalanche_node.id.short_id().to_string(), CB58_ID);

        // Creating the node should succeed
        let node = AshNode::from_string(&format!("0x{HEX_ID}")).unwrap();

        assert_eq!(node.avalanche_node.id.short_id().to_string(), CB58_ID);

        // Creating the node should succeed
        let node = AshNode::from_string(&format!("NodeID-{CB58_ID}")).unwrap();

        assert_eq!(node.avalanche_node.id.short_id().to_string(), CB58_ID);

        // Creating the node should fail
        let node = AshNode::from_string("invalid");

        assert!(node.is_err());
    }

    #[test]
    fn get_info() {
        let node = AshNode::from_cb58_id(CB58_ID).unwrap();

        let node_info = node.info();

        assert_eq!(node_info.id.p_chain, format!("NodeID-{CB58_ID}"));
        assert_eq!(node_info.id.cb58, CB58_ID);
        assert_eq!(node_info.id.bytes, &BYTES_ID);
        assert_eq!(
            node_info.id.hex,
            BYTES_ID
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
    }
}
