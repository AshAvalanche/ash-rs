// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

pub mod subnet_evm;

// Module that contains code to interact with Avalanche VMs

use crate::errors::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use strum::EnumString;

/// List of Avalanche VM types
#[derive(Default, Debug, Display, Clone, Serialize, Deserialize, PartialEq, EnumString)]
pub enum AvalancheVmType {
    /// Coreth (Avalanche C-Chain)
    Coreth,
    /// Platform VM (Avalanche P-Chain)
    PlatformVM,
    /// Avalanche VM (Avalanche X-Chain)
    AvalancheVM,
    /// Subnet EVM
    #[default]
    SubnetEVM,
    /// Any other custom VM
    Custom(String),
}

/// Encode the genesis data (JSON) to bytes
pub fn encode_genesis_data(
    vm_type: AvalancheVmType,
    genesis_json: &str,
) -> Result<Vec<u8>, AshError> {
    match vm_type {
        AvalancheVmType::SubnetEVM => subnet_evm::encode_genesis_data(genesis_json),
        _ => Err(AvalancheVMError::GenesisEncoding(format!(
            "encoding is not supported for VM '{}'",
            vm_type
        ))
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_genesis_data_unsupported_vm() {
        let vm_type: AvalancheVmType = serde_json::from_str(r#"{"Custom": "SuperVM"}"#).unwrap();

        assert_eq!(
            encode_genesis_data(vm_type, "").err(),
            Some(AshError::from(AvalancheVMError::GenesisEncoding(
                "encoding is not supported for VM 'SuperVM'".to_string()
            )))
        );
    }
}
