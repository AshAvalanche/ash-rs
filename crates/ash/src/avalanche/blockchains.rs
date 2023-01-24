// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

// Module that contains code to interact with Avalanche blockchains

use crate::avalanche::avalanche_id_from_string;
use avalanche_types::ids::Id;
use serde::{Deserialize, Serialize};

/// Avalanche blockchain
#[derive(Debug, Serialize, Deserialize)]
pub struct AvalancheBlockchain {
    #[serde(deserialize_with = "avalanche_id_from_string")]
    pub id: Id,
    pub name: String,
    pub vm_type: String,
    pub rpc_url: String,
}

impl Clone for AvalancheBlockchain {
    fn clone(&self) -> AvalancheBlockchain {
        AvalancheBlockchain {
            id: self.id.clone(),
            name: self.name.clone(),
            vm_type: self.vm_type.clone(),
            rpc_url: self.rpc_url.clone(),
        }
    }
}
