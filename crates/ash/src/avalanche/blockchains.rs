// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

// Module that contains code to interact with Avalanche blockchains

use crate::avalanche::avalanche_id_from_string;
use avalanche_types::ids::Id;
use serde::{Deserialize, Serialize};

/// Avalanche blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct AvalancheBlockchain {
    #[serde(deserialize_with = "avalanche_id_from_string")]
    pub id: Id,
    pub name: String,
    pub vm_type: String,
    pub rpc_url: String,
}
