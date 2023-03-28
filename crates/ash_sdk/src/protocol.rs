// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

pub mod contracts;
pub mod nodes;

// Module that contains code to interact with the Ash protocol

use crate::avalanche::AvalancheNetwork;
use crate::errors::*;
use crate::protocol::{
    contracts::{ash_router_http::AshRouterHttp, AshContractMetadata},
    nodes::AshNode,
};
use serde::Serialize;

/// Ash protocol entities
#[derive(Debug, Serialize)]
pub struct AshProtocol {
    pub router_contract: AshRouterHttp,
    pub nodes: Vec<AshNode>,
}

impl AshProtocol {
    /// Create a new Ash protocol instance
    pub fn new(network: &AvalancheNetwork, config: Option<&str>) -> Result<AshProtocol, AshError> {
        let router_config = AshContractMetadata::load("AshRouter", config)?;

        let router_contract = AshRouterHttp::new(
            &router_config.get_address(&network.name)?,
            network.get_cchain()?,
        )?;

        Ok(AshProtocol {
            router_contract,
            nodes: vec![],
        })
    }

    /// Update the list of Ash nodes registered on the protocol
    pub async fn update_nodes(&mut self) -> Result<(), AshError> {
        let nodes = self.router_contract.get_rentable_validators().await?;
        self.nodes = nodes
            .iter()
            // Unwrap is safe because we know that the nodes are valid bytes arrays of length 20
            .map(|node| AshNode::from_bytes_id(node).unwrap())
            .collect();

        Ok(())
    }
}
