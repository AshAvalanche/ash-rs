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

    // Create a new Ash node from an Avalanche node ID byte slice
    pub fn from_bytes_id(nodeid: &[u8]) -> Result<Self, Error> {
        let id = Id::from_slice(nodeid);

        Ok(AshNode { id })
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
    const BYTES_ID: [u8; 20] = [
        161, 46, 131, 50, 233, 173, 105, 174, 159, 112, 163, 213, 113, 90, 223, 204, 223, 5, 188,
        101,
    ];

    #[test]
    fn create_from_cb58_id() {
        // Creating the node should succeed
        let node = AshNode::from_cb58_id(CB58_ID).unwrap();

        assert_eq!(node.id.to_string(), format!("NodeID-{}", CB58_ID));
        assert_eq!(node.id.short_id().to_string(), CB58_ID);
    }

    #[test]
    fn create_from_bytes_id() {
        // Creating the node should succeed
        let node = AshNode::from_bytes_id(&BYTES_ID).unwrap();

        assert_eq!(node.id.to_string(), format!("NodeID-{}", CB58_ID));
        assert_eq!(node.id.short_id().to_string(), CB58_ID);
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
