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
    #[error("AvalancheBlockchain error: {0}")]
    AvalancheBlockchainError(#[from] AvalancheBlockchainError),
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
    #[error("failed to parse '{value}' as {target_type}: {msg}")]
    ParseFailure {
        value: String,
        target_type: String,
        msg: String,
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
    #[error("RPC response contains an error: code {code}, message: {message}, data: {data:?}")]
    ResponseError {
        code: i32,
        message: String,
        data: Option<String>,
    },
    #[error("failed to call {function_name} on '{contract_addr}': {msg}")]
    EthCallFailure {
        contract_addr: String,
        function_name: String,
        msg: String,
    },
    #[error("unknown RPC error: {0}")]
    Unknown(String),
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
pub enum AvalancheBlockchainError {
    #[error("failed to get ethers Provider for blockchain '{blockchain_id}': {msg}")]
    EthersProvider { blockchain_id: Id, msg: String },
}

#[derive(Error, Debug)]
pub enum AshNodeError {
    #[error("'{id}' is not a valid node ID: {msg}")]
    InvalidId { id: String, msg: String },
}
