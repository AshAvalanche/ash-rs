// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

// Module that contains code to interact with Avalanche subnets

use crate::avalanche::{avalanche_id_from_string, blockchains::AvalancheBlockchain};
use avalanche_types::ids::Id;
use serde::{Deserialize, Serialize};

/// Avalanche Subnet
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct AvalancheSubnet {
    #[serde(deserialize_with = "avalanche_id_from_string")]
    pub id: Id,
    /// List of the subnet's blockchains
    pub blockchains: Vec<AvalancheBlockchain>,
}

impl AvalancheSubnet {
    /// Get a Blockchain from the Subnet by its ID
    pub fn get_blockchain(&self, id: &str) -> Option<&AvalancheBlockchain> {
        self.blockchains
            .iter()
            .find(|&blockchain| blockchain.id.to_string() == id)
    }
}

impl Clone for AvalancheSubnet {
    fn clone(&self) -> AvalancheSubnet {
        AvalancheSubnet {
            id: self.id.clone(),
            blockchains: self.blockchains.clone(),
        }
    }
}
