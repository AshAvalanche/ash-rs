// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche subnets

use crate::avalanche::{avalanche_id_from_string, blockchains::AvalancheBlockchain};
use avalanche_types::ids::Id;
use serde::{Deserialize, Serialize};

/// Avalanche Subnet
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheSubnet {
    #[serde(deserialize_with = "avalanche_id_from_string")]
    pub id: Id,
    pub control_keys: Vec<String>,
    pub threshold: u8,
    /// List of the Subnet's blockchains
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
