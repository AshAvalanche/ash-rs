// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

use avalanche_types::ids::node::Id;
use regex::Regex;
use std::{io::Error, str::FromStr};

// Struct that represents a node of the Ash protocol
#[derive(Debug)]
pub struct AshNode {
    // The node's ID
    pub id: Id,
}

impl AshNode {
    // Create a new Ash node from an Avalanche node ID string
    pub fn from_cb58_id(nodeid: &str) -> Result<Self, Error> {
        let id = Id::from_str(nodeid)?;

        Ok(AshNode { id })
    }

    // Create a new Ash node from an Avalanche node ID byte slice
    pub fn from_bytes_id(nodeid: &[u8]) -> Result<Self, Error> {
        let id = Id::from_slice(nodeid);

        Ok(AshNode { id })
    }

    // Create a new Ash node from an Avalanche node ID hex string
    pub fn from_hex_id(nodeid: &str) -> Result<Self, Error> {
        // Convert the hex string to a byte slice
        let nodeid = hex::decode(nodeid);

        match nodeid {
            Ok(nodeid) => AshNode::from_bytes_id(&nodeid),
            Err(_) => Err(Error::new(
                std::io::ErrorKind::InvalidInput,
                "Could not convert string to bytes",
            )),
        }
    }

    // Create a new Ash node from a string
    // Try to find out the node ID format using a regex
    pub fn from_string(nodeid: &str) -> Result<Self, Error> {
        // Check if the node ID is a valid CB58 string
        let re = Regex::new(r"^(NodeID-)?[A-Za-z0-9]{32,33}$").unwrap();

        if re.is_match(nodeid) {
            return AshNode::from_cb58_id(nodeid);
        }

        // Check if the node ID is a valid hex string
        let re = Regex::new(r"^(0x)?[0-9a-fA-F]{40}$").unwrap();

        if re.is_match(nodeid) {
            return AshNode::from_hex_id(nodeid.trim_start_matches("0x"));
        }

        Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid node ID",
        ))
    }

    // Get the node's ID as a string
    pub fn get_id_string(&self) -> String {
        self.id.to_string()
    }

    // Get the node's ID as a CB58 string
    pub fn get_id_cb58(&self) -> String {
        self.id.short_id().to_string()
    }

    // Get the node's ID as a byte slice
    pub fn get_id_bytes(&self) -> &[u8] {
        self.id.as_ref()
    }

    // Get the node's ID as a hex string
    pub fn get_id_hex(&self) -> String {
        self.id
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod test {
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

        assert_eq!(node.id.short_id().to_string(), CB58_ID);
    }

    #[test]
    fn create_from_bytes_id() {
        // Creating the node should succeed
        let node = AshNode::from_bytes_id(&BYTES_ID).unwrap();

        assert_eq!(node.id.short_id().to_string(), CB58_ID);
    }

    #[test]
    fn create_from_hex_id() {
        // Creating the node should succeed
        let node = AshNode::from_hex_id(HEX_ID).unwrap();

        assert_eq!(node.id.short_id().to_string(), CB58_ID);
    }

    #[test]
    fn create_from_string() {
        // Creating the node should succeed
        let node = AshNode::from_string(CB58_ID).unwrap();

        assert_eq!(node.id.short_id().to_string(), CB58_ID);

        // Creating the node should succeed
        let node = AshNode::from_string(HEX_ID).unwrap();

        assert_eq!(node.id.short_id().to_string(), CB58_ID);

        // Creating the node should succeed
        let node = AshNode::from_string(&format!("0x{}", HEX_ID)).unwrap();

        assert_eq!(node.id.short_id().to_string(), CB58_ID);

        // Creating the node should succeed
        let node = AshNode::from_string(&format!("NodeID-{}", CB58_ID)).unwrap();

        assert_eq!(node.id.short_id().to_string(), CB58_ID);

        // Creating the node should fail
        let node = AshNode::from_string("invalid");

        assert!(node.is_err());
    }

    #[test]
    fn get_id() {
        let node = AshNode::from_cb58_id(CB58_ID).unwrap();

        assert_eq!(node.get_id_string(), format!("NodeID-{}", CB58_ID));
        assert_eq!(node.get_id_cb58(), CB58_ID);
        assert_eq!(node.get_id_bytes(), BYTES_ID);
        assert_eq!(
            node.get_id_hex(),
            BYTES_ID
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>()
        );
    }
}
