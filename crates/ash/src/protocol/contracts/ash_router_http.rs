// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Ash contracts

include!(concat!(env!("OUT_DIR"), "/ash_router_abigen.rs"));

use crate::{avalanche::blockchains::AvalancheBlockchain, errors::*};
use ethers::core::types::Address;
use ethers::providers::{Http, Provider};
use serde::Serialize;
use AshRouter;

/// AshRouter contract HTTP provider
#[derive(Debug, Clone, Serialize)]
pub struct AshRouterHttp {
    address: Address,
    #[serde(skip)]
    provider: AshRouter<Provider<Http>>,
}

impl AshRouterHttp {
    /// Create a new AshRouter contract HTTP provider on the given Avalanche blockchain
    pub fn new(address: &str, chain: &AvalancheBlockchain) -> Result<AshRouterHttp, AshError> {
        let client = chain.get_ethers_provider()?;
        let ash_router = AshRouter::new(
            address
                .parse::<Address>()
                .map_err(|e| ConfigError::ParseFailure {
                    value: address.to_string(),
                    target_type: "Address".to_string(),
                    msg: e.to_string(),
                })?,
            client.into(),
        );

        Ok(AshRouterHttp {
            address: ash_router.address(),
            provider: ash_router,
        })
    }

    /// Get the AshFactory contract address
    pub async fn factory_addr(&self) -> Result<Address, AshError> {
        let factory_address =
            self.provider
                .factory_addr()
                .call()
                .await
                .map_err(|e| RpcError::EthCallFailure {
                    contract_addr: self.address.to_string(),
                    function_name: "factoryAddr".to_string(),
                    msg: e.to_string(),
                })?;

        Ok(factory_address)
    }

    /// Get the list of rentable Ash nodes
    pub async fn get_rentable_validators(&self) -> Result<Vec<[u8; 24]>, AshError> {
        let rentable_validators = self
            .provider
            .get_rentable_validators()
            .call()
            .await
            .map_err(|e| RpcError::EthCallFailure {
                contract_addr: self.provider.address().to_string(),
                function_name: "getRentableValidators".to_string(),
                msg: e.to_string(),
            })?;

        Ok(rentable_validators)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{avalanche::AvalancheNetwork, protocol::contracts::AshContractMetadata};
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

    #[async_std::test]
    async fn test_ash_router_get_rentable_validators() {
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

        assert!(ash_router.get_rentable_validators().await.is_ok());
    }
}
