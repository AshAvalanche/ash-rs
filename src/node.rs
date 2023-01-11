// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

use std::{io::Error, str::FromStr};

use avalanche_types::ids::node::Id;

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

    // Create a new Ash node from an Avalanche node ID byte array
    pub fn from_bytes_id(nodeid: &[u8]) -> Result<Self, Error> {
        let id = Id::from_slice(nodeid);

        Ok(AshNode { id })
    }

    // Get the node's ID as a string
    pub fn get_id_str(&self) -> String {
        self.id.to_string()
    }

    // Get the node's ID as a CB58 string
    pub fn get_id_cb58(&self) -> String {
        self.id.short_id().to_string()
    }

    // Get the node's ID as a byte array
    pub fn get_id_bytes(&self) -> Vec<u8> {
        self.id.to_vec()
    }

    // Get the node's ID as a hex string
    pub fn get_id_hex(&self) -> String {
        self.id
            .to_vec()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_from_cb58_id() {
        let cb58_id = "NodeID-FhFWdWodxktJYq884nrJjWD8faLTk9jmp";

        // Creating the node should succeed
        let node = AshNode::from_cb58_id(&cb58_id).unwrap();

        assert_eq!(
            node.id.to_string(),
            "NodeID-FhFWdWodxktJYq884nrJjWD8faLTk9jmp"
        );

        assert_eq!(
            node.id.short_id().to_string(),
            "FhFWdWodxktJYq884nrJjWD8faLTk9jmp"
        );
    }

    #[test]
    fn create_from_bytes_id() {
        let bytes_id = [
            161, 46, 131, 50, 233, 173, 105, 174, 159, 112, 163, 213, 113, 90, 223, 204, 223, 5,
            188, 101,
        ];

        // Creating the node should succeed
        let node = AshNode::from_bytes_id(&bytes_id).unwrap();

        assert_eq!(
            node.id.to_string(),
            "NodeID-FhFWdWodxktJYq884nrJjWD8faLTk9jmp"
        );

        assert_eq!(
            node.id.short_id().to_string(),
            "FhFWdWodxktJYq884nrJjWD8faLTk9jmp"
        );
    }

    #[test]
    fn get_id() {
        let cb58_id = "NodeID-FhFWdWodxktJYq884nrJjWD8faLTk9jmp";

        let node = AshNode::from_cb58_id(&cb58_id).unwrap();

        assert_eq!(node.get_id_str(), cb58_id);
        assert_eq!(node.get_id_cb58(), "FhFWdWodxktJYq884nrJjWD8faLTk9jmp");
        assert_eq!(
            node.get_id_bytes(),
            [
                161, 46, 131, 50, 233, 173, 105, 174, 159, 112, 163, 213, 113, 90, 223, 204, 223,
                5, 188, 101
            ]
        );
        assert_eq!(
            node.get_id_hex(),
            "a12e8332e9ad69ae9f70a3d5715adfccdf05bc65"
        );
    }
}
