// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Ash contracts

include!(concat!(env!("OUT_DIR"), "/ash_router_abigen.rs"));

use crate::{avalanche::blockchains::AvalancheBlockchain, error::AshError};
use ethers::{core::types::Address, providers::Http, providers::Provider};
use AshRouter;

/// AshRouter contract HTTP provider
pub struct AshRouterHttp {
    provider: AshRouter<Provider<Http>>,
}

impl AshRouterHttp {
    /// Create a new AshRouter contract HTTP provider on the given Avalanche blockchain
    pub fn new(
        ash_router_address: &str,
        chain: &AvalancheBlockchain,
    ) -> Result<AshRouterHttp, AshError> {
        let client = chain.get_ethers_provider()?;
        let ash_router = AshRouter::new(
            ash_router_address.parse::<Address>().map_err(|e| {
                AshError::ConfigError(format!("Failed to parse AshRouter address: {e}",))
            })?,
            client.into(),
        );

        Ok(AshRouterHttp {
            provider: ash_router,
        })
    }

    /// Get the AshFactory contract address
    pub async fn factory_addr(&self) -> Result<Address, AshError> {
        let factory_address = self
            .provider
            .factory_addr()
            .call()
            .await
            .map_err(|e| AshError::RpcError(format!("Failed to get factory address: {e}",)))?;

        Ok(factory_address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{avalanche::AvalancheNetwork, contracts::AshContractMetadata};
    use std::env;

    // Load the test network from the ASH_TEST_CONFIG file
    fn load_test_network() -> AvalancheNetwork {
        let config_path =
            env::var("ASH_TEST_AVAX_CONFIG").unwrap_or("tests/conf/default.yml".to_string());
        AvalancheNetwork::load("fuji", Some(&config_path)).unwrap()
    }

    // Load the test AshRouter contract from the ASH_TEST_CONFIG file
    fn load_ash_router_metadata() -> AshContractMetadata {
        let config_path =
            env::var("ASH_TEST_AVAX_CONFIG").unwrap_or("tests/conf/default.yml".to_string());
        AshContractMetadata::load("AshRouter", Some(&config_path)).unwrap()
    }

    #[async_std::test]
    async fn test_ash_router_new() {
        let network = load_test_network();
        let ash_router_address = load_ash_router_metadata()
            .addresses
            .iter()
            .find(|&address| address.network == network.name)
            .unwrap()
            .address
            .clone();

        assert!(AshRouterHttp::new(&ash_router_address, network.get_cchain().unwrap()).is_ok());
    }

    #[async_std::test]
    async fn test_ash_router_factory_addr() {
        let network = load_test_network();
        let ash_router_address = load_ash_router_metadata()
            .addresses
            .iter()
            .find(|&address| address.network == network.name)
            .unwrap()
            .address
            .clone();

        let ash_router =
            AshRouterHttp::new(&ash_router_address, network.get_cchain().unwrap()).unwrap();

        assert!(ash_router.factory_addr().await.is_ok());
    }
}
