// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to generate msgs

use avalanche_types::{ids::node::Id as NodeId, ids::Id};

/// Ash library msgs enum
#[derive(Debug)]
pub enum AshError {
    ConfigError(String),
    RpcError(String),
    AvalancheNetworkError {
        network: String,
        msg: String,
    },
    AvalancheSubnetError {
        id: Id,
        msg: String,
    },
    AvalancheNodeError {
        http_host: String,
        http_port: u16,
        msg: String,
    },
    AshNodeError {
        id: NodeId,
        msg: String,
    },
}

impl std::fmt::Display for AshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AshError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            AshError::RpcError(msg) => write!(f, "RPC error: {}", msg),
            AshError::AvalancheNetworkError { network, msg } => {
                write!(f, "AvalancheNetwork('{}') error: {}", network, msg)
            }
            AshError::AvalancheSubnetError { id, msg } => {
                write!(f, "AvalancheSubnet('{}') error: {}", id, msg)
            }
            AshError::AvalancheNodeError {
                http_host,
                http_port,
                msg,
            } => write!(
                f,
                "AvalancheNode('{}:{}') error: {}",
                http_host, http_port, msg
            ),
            AshError::AshNodeError { id, msg } => write!(f, "AshNode('{}') error: {}", id, msg),
        }
    }
}
