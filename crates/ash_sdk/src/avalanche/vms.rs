// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

pub mod subnet_evm;

// Module that contains code to interact with Avalanche VMs

use crate::errors::*;
use avalanche_types::ids::Id;
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
/// For Custom VMs, simply encode the JSON as bytes
pub fn encode_genesis_data(
    vm_type: AvalancheVmType,
    genesis_json: &str,
) -> Result<Vec<u8>, AshError> {
    match vm_type {
        AvalancheVmType::SubnetEVM => subnet_evm::encode_genesis_data(genesis_json),
        AvalancheVmType::Custom(_) => {
            let parsed_json: Result<serde_json::Value, serde_json::Error> =
                serde_json::from_str(genesis_json);
            match parsed_json {
                Ok(json) => Ok(serde_json::to_vec(&json).map_err(|e| {
                    AvalancheVMError::GenesisEncoding(format!("failed encode JSON: {}", e))
                })?),
                Err(e) => Err(AvalancheVMError::GenesisEncoding(format!(
                    "error parsing genesis JSON: {}",
                    e
                ))
                .into()),
            }
        }
        _ => Err(AvalancheVMError::GenesisEncoding(format!(
            "encoding is not supported for VM '{}'",
            vm_type
        ))
        .into()),
    }
}

/// Generate the VM ID from the VM name as a string
/// The VM ID is the CB58 encoded 32-byte identifier of the VM
pub fn generate_vm_id(vm_name: &str) -> Id {
    Id::from_slice(vm_name.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::str::FromStr;

    const SUBNET_EVM_GENESIS_BYTES: &[u8] = &[
        123, 34, 99, 111, 110, 102, 105, 103, 34, 58, 123, 34, 99, 104, 97, 105, 110, 73, 100, 34,
        58, 49, 51, 50, 49, 51, 44, 34, 102, 101, 101, 67, 111, 110, 102, 105, 103, 34, 58, 123,
        34, 103, 97, 115, 76, 105, 109, 105, 116, 34, 58, 56, 48, 48, 48, 48, 48, 48, 44, 34, 116,
        97, 114, 103, 101, 116, 66, 108, 111, 99, 107, 82, 97, 116, 101, 34, 58, 50, 44, 34, 109,
        105, 110, 66, 97, 115, 101, 70, 101, 101, 34, 58, 50, 53, 48, 48, 48, 48, 48, 48, 48, 48,
        48, 44, 34, 116, 97, 114, 103, 101, 116, 71, 97, 115, 34, 58, 49, 53, 48, 48, 48, 48, 48,
        48, 44, 34, 98, 97, 115, 101, 70, 101, 101, 67, 104, 97, 110, 103, 101, 68, 101, 110, 111,
        109, 105, 110, 97, 116, 111, 114, 34, 58, 51, 54, 44, 34, 109, 105, 110, 66, 108, 111, 99,
        107, 71, 97, 115, 67, 111, 115, 116, 34, 58, 48, 44, 34, 109, 97, 120, 66, 108, 111, 99,
        107, 71, 97, 115, 67, 111, 115, 116, 34, 58, 49, 48, 48, 48, 48, 48, 48, 44, 34, 98, 108,
        111, 99, 107, 71, 97, 115, 67, 111, 115, 116, 83, 116, 101, 112, 34, 58, 50, 48, 48, 48,
        48, 48, 125, 44, 34, 104, 111, 109, 101, 115, 116, 101, 97, 100, 66, 108, 111, 99, 107, 34,
        58, 48, 44, 34, 101, 105, 112, 49, 53, 48, 66, 108, 111, 99, 107, 34, 58, 48, 44, 34, 101,
        105, 112, 49, 53, 48, 72, 97, 115, 104, 34, 58, 34, 48, 120, 50, 48, 56, 54, 55, 57, 57,
        97, 101, 101, 98, 101, 97, 101, 49, 51, 53, 99, 50, 52, 54, 99, 54, 53, 48, 50, 49, 99, 56,
        50, 98, 52, 101, 49, 53, 97, 50, 99, 52, 53, 49, 51, 52, 48, 57, 57, 51, 97, 97, 99, 102,
        100, 50, 55, 53, 49, 56, 56, 54, 53, 49, 52, 102, 48, 34, 44, 34, 101, 105, 112, 49, 53,
        53, 66, 108, 111, 99, 107, 34, 58, 48, 44, 34, 101, 105, 112, 49, 53, 56, 66, 108, 111, 99,
        107, 34, 58, 48, 44, 34, 98, 121, 122, 97, 110, 116, 105, 117, 109, 66, 108, 111, 99, 107,
        34, 58, 48, 44, 34, 99, 111, 110, 115, 116, 97, 110, 116, 105, 110, 111, 112, 108, 101, 66,
        108, 111, 99, 107, 34, 58, 48, 44, 34, 112, 101, 116, 101, 114, 115, 98, 117, 114, 103, 66,
        108, 111, 99, 107, 34, 58, 48, 44, 34, 105, 115, 116, 97, 110, 98, 117, 108, 66, 108, 111,
        99, 107, 34, 58, 48, 44, 34, 109, 117, 105, 114, 71, 108, 97, 99, 105, 101, 114, 66, 108,
        111, 99, 107, 34, 58, 48, 44, 34, 115, 117, 98, 110, 101, 116, 69, 86, 77, 84, 105, 109,
        101, 115, 116, 97, 109, 112, 34, 58, 48, 125, 44, 34, 110, 111, 110, 99, 101, 34, 58, 34,
        48, 120, 48, 34, 44, 34, 116, 105, 109, 101, 115, 116, 97, 109, 112, 34, 58, 34, 48, 120,
        48, 34, 44, 34, 101, 120, 116, 114, 97, 68, 97, 116, 97, 34, 58, 34, 48, 120, 48, 48, 34,
        44, 34, 103, 97, 115, 76, 105, 109, 105, 116, 34, 58, 34, 48, 120, 55, 97, 49, 50, 48, 48,
        34, 44, 34, 100, 105, 102, 102, 105, 99, 117, 108, 116, 121, 34, 58, 34, 48, 120, 48, 34,
        44, 34, 109, 105, 120, 72, 97, 115, 104, 34, 58, 34, 48, 120, 48, 48, 48, 48, 48, 48, 48,
        48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 34, 44, 34, 99, 111, 105, 110, 98, 97, 115,
        101, 34, 58, 34, 48, 120, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        48, 34, 44, 34, 97, 108, 108, 111, 99, 34, 58, 123, 34, 56, 100, 98, 57, 55, 67, 55, 99,
        69, 99, 69, 50, 52, 57, 99, 50, 98, 57, 56, 98, 68, 67, 48, 50, 50, 54, 67, 99, 52, 67, 50,
        65, 53, 55, 66, 70, 53, 50, 70, 67, 34, 58, 123, 34, 98, 97, 108, 97, 110, 99, 101, 34, 58,
        34, 48, 120, 50, 57, 53, 98, 101, 57, 54, 101, 54, 52, 48, 54, 54, 57, 55, 50, 48, 48, 48,
        48, 48, 48, 34, 125, 125, 44, 34, 110, 117, 109, 98, 101, 114, 34, 58, 34, 48, 120, 48, 34,
        44, 34, 103, 97, 115, 85, 115, 101, 100, 34, 58, 34, 48, 120, 48, 34, 44, 34, 112, 97, 114,
        101, 110, 116, 72, 97, 115, 104, 34, 58, 34, 48, 120, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
        48, 48, 48, 48, 48, 48, 48, 48, 48, 34, 125,
    ];

    #[test]
    fn test_encode_genesis_data_unsupported_vm() {
        assert_eq!(
            encode_genesis_data(AvalancheVmType::AvalancheVM, "").err(),
            Some(AshError::from(AvalancheVMError::GenesisEncoding(
                "encoding is not supported for VM 'AvalancheVM'".to_string()
            )))
        );
    }

    #[test]
    fn test_encode_genesis_data_custom_vm() {
        let genesis_str = fs::read_to_string("tests/genesis/subnet-evm.json").unwrap();

        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(
                &encode_genesis_data(AvalancheVmType::Custom(String::new()), &genesis_str).unwrap()
            )
            .unwrap(),
            serde_json::from_slice::<serde_json::Value>(SUBNET_EVM_GENESIS_BYTES).unwrap()
        )
    }

    #[test]
    fn test_generate_vm_id() {
        let vm_id = generate_vm_id("timestamp");

        assert_eq!(
            vm_id,
            Id::from_str("tGas3T58KzdjLHhBDMnH2TvrddhqTji5iZAMZ3RXs2NLpSnhH").unwrap()
        );
    }
}
