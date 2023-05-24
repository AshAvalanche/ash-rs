// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche blockchains

use crate::errors::*;
use avalanche_types::{ids::Id, jsonrpc::platformvm::Blockchain};
use ethers::providers::{Http, Provider};
use serde::{Deserialize, Serialize};

/// Avalanche blockchain
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheBlockchain {
    pub id: Id,
    pub name: String,
    #[serde(skip)]
    pub subnet_id: Id,
    #[serde(default)]
    pub vm_id: Id,
    #[serde(default)]
    pub vm_type: String,
    #[serde(default)]
    pub rpc_url: String,
}

impl AvalancheBlockchain {
    /// Get an ethers Provider for this blockchain
    /// Only works for EVM blockchains
    pub fn get_ethers_provider(&self) -> Result<Provider<Http>, AshError> {
        match self.vm_type.as_str() {
            "EVM" => Ok(
                Provider::<Http>::try_from(self.rpc_url.clone()).map_err(|e| {
                    AvalancheBlockchainError::EthersProvider {
                        blockchain_id: self.id.to_string(),
                        msg: e.to_string(),
                    }
                })?,
            ),
            _ => Err(AvalancheBlockchainError::EthersProvider {
                blockchain_id: self.id.to_string(),
                msg: format!(
                    "cannot create an ethers Provider for '{}' type blockchain",
                    self.vm_type
                ),
            }
            .into()),
        }
    }
}

impl From<Blockchain> for AvalancheBlockchain {
    fn from(blockchain: Blockchain) -> Self {
        Self {
            id: blockchain.id,
            name: blockchain.name,
            subnet_id: blockchain.subnet_id,
            vm_id: blockchain.vm_id,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::avalanche::AvalancheNetwork;
    use std::env;

    // Load the test network from the ASH_TEST_CONFIG file
    fn load_test_network() -> AvalancheNetwork {
        let config_path =
            env::var("ASH_TEST_AVAX_CONFIG").unwrap_or("tests/conf/default.yml".to_string());
        AvalancheNetwork::load("fuji", Some(&config_path)).unwrap()
    }

    #[test]
    fn test_avalanche_blockchain_get_ethers_provider() {
        let fuji = load_test_network();

        // Test that we can get an ethers Provider for the C-Chain
        let cchain_provider = fuji.get_cchain().unwrap().get_ethers_provider();
        assert!(cchain_provider.is_ok());

        // Test that the provider URL is correct
        assert_eq!(
            cchain_provider.unwrap().url().to_string(),
            fuji.get_cchain().unwrap().rpc_url
        );
    }

    #[test]
    fn test_avalanche_blockchain_get_ethers_provider_not_evm() {
        let fuji = load_test_network();

        // Test that we can't get an ethers Provider for the P-Chain
        assert!(fuji.get_pchain().unwrap().get_ethers_provider().is_err());
    }
}
