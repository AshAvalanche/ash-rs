// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to generate errors

use avalanche_types::ids::Id;
use thiserror::Error;

/// Ash library errors enum
#[derive(Error, Debug)]
pub enum AshError {
    #[error("Config error: {0}")]
    ConfigError(#[from] ConfigError),
    #[error("RPC error: {0}")]
    RpcError(#[from] RpcError),
    #[error("AvalancheNetwork error: {0}")]
    AvalancheNetworkError(#[from] AvalancheNetworkError),
    #[error("AvalancheSubnet error: {0}")]
    AvalancheSubnetError(#[from] AvalancheSubnetError),
    #[error("AshNode error: {0}")]
    AshNodeError(#[from] AshNodeError),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to build configuration: {0}")]
    BuildFailure(String),
    #[error("failed to deserialize configuration from '{config_file}': {msg}")]
    DeserializeFailure { config_file: String, msg: String },
    #[error("failed to dump configuration at '{config_file}': {msg}")]
    DumpFailure { config_file: String, msg: String },
    #[error("{target_type} '{target_value}' not found in configuration")]
    NotFound {
        target_type: String,
        target_value: String,
    },
}

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("failed to get {data_type} for {target_type} '{target_value}': {msg}")]
    GetFailure {
        data_type: String,
        target_type: String,
        target_value: String,
        msg: String,
    },
}

#[derive(Error, Debug)]
pub enum AvalancheNetworkError {
    #[error("{target_type} '{target_value}' not found in network '{network}'")]
    NotFound {
        network: String,
        target_type: String,
        target_value: String,
    },
}

#[derive(Error, Debug)]
pub enum AvalancheSubnetError {
    #[error("{target_type} '{target_value}' not found in Subnet '{subnet_id}'")]
    NotFound {
        subnet_id: Id,
        target_type: String,
        target_value: String,
    },
}

#[derive(Error, Debug)]
pub enum AshNodeError {
    #[error("'{id}' is not a valid node ID: {msg}")]
    InvalidId { id: String, msg: String },
}
