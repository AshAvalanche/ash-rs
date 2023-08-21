// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to generate errors

use thiserror::Error;

/// Ash library errors enum
#[derive(Error, Debug, PartialEq)]
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
    #[error("AvalancheWallet error: {0}")]
    AvalancheWalletError(#[from] AvalancheWalletError),
    #[error("Avalanche VM error: {0}")]
    AvalancheVMError(#[from] AvalancheVMError),
    #[error("Avalanche Warp Messaging error: {0}")]
    AvalancheWarpMessagingError(#[from] AvalancheWarpMessagingError),
}

#[derive(Error, Debug, PartialEq)]
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

#[derive(Error, Debug, PartialEq)]
pub enum RpcError {
    #[error("failed to parse RPC URL '{rpc_url}': {msg}")]
    UrlParseFailure { rpc_url: String, msg: String },
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
    #[error("failed to query event logs on '{contract_addr}': {msg}")]
    EthLogsFailure { contract_addr: String, msg: String },
    #[error("unknown RPC error: {0}")]
    Unknown(String),
}

#[derive(Error, Debug, PartialEq)]
pub enum AvalancheNetworkError {
    #[error("{target_type} '{target_value}' not found in network '{network}'")]
    NotFound {
        network: String,
        target_type: String,
        target_value: String,
    },
    #[error("{operation} is not allowed on network '{network}'")]
    OperationNotAllowed { operation: String, network: String },
    #[error("'{address}' is not a valid address: {msg}")]
    InvalidAddress { address: String, msg: String },
}

#[derive(Error, Debug, PartialEq)]
pub enum AvalancheSubnetError {
    #[error("{target_type} '{target_value}' not found in Subnet '{subnet_id}'")]
    NotFound {
        subnet_id: String,
        target_type: String,
        target_value: String,
    },
    #[error("{operation} is not allowed on {subnet_type} Subnet '{subnet_id}'")]
    OperationNotAllowed {
        operation: String,
        subnet_id: String,
        subnet_type: String,
    },
}

#[derive(Error, Debug, PartialEq)]
pub enum AvalancheBlockchainError {
    #[error("operation '{operation}' is not allowed on '{vm_type}' blockchain '{blockchain_id}'")]
    OperationNotAllowed {
        blockchain_id: String,
        vm_type: String,
        operation: String,
    },
    #[error("failed to get ethers Provider for blockchain '{blockchain_id}': {msg}")]
    EthersProvider { blockchain_id: String, msg: String },
    #[error("failed to parse block number from '{block_number}': {msg}")]
    BlockNumberParseFailure { block_number: String, msg: String },
}

#[derive(Error, Debug, PartialEq)]
pub enum AvalancheWalletError {
    #[error("failed to generate private key: {0}")]
    PrivateKeyGenerationFailure(String),
    #[error("failed to use provided private key: {0}")]
    InvalidPrivateKey(String),
    #[error("failed to create Avalanche wallet: {0}")]
    CreationFailure(String),
    #[error("failed to issue '{tx_type}' transaction on blockchain '{blockchain_name}': {msg}")]
    IssueTx {
        blockchain_name: String,
        tx_type: String,
        msg: String,
    },
}

#[derive(Error, Debug, PartialEq)]
pub enum AvalancheVMError {
    #[error("unsupported VM '{0}'")]
    UnsupportedVM(String),
    #[error("failed to encode genesis data: {0}")]
    GenesisEncoding(String),
}

#[derive(Error, Debug, PartialEq)]
pub enum AvalancheWarpMessagingError {
    #[error("failed to parse {property} of message: {msg}")]
    ParseFailure { property: String, msg: String },
    #[error("invalid message signature: {0}")]
    InvalidSignature(String),
}
