// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche blockchains

use crate::{
    avalanche::{
        txs::p,
        vms::{
            subnet_evm::{precompiles::WarpMessengerHttp, warp::SubnetEVMWarpMessage},
            AvalancheVmType,
        },
        wallets::AvalancheWallet,
        warp::*,
    },
    errors::*,
    utils::*,
};
use avalanche_types::{ids::Id, jsonrpc::platformvm::Blockchain};
use ethers::{
    providers::{Http, Provider},
    types::H256,
};
use serde::{Deserialize, Serialize};

/// Avalanche blockchain
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheBlockchain {
    #[serde(default)]
    pub id: Id,
    pub name: String,
    #[serde(default, rename = "subnetID")]
    pub subnet_id: Id,
    #[serde(default, rename = "vmID")]
    pub vm_id: Id,
    pub vm_type: AvalancheVmType,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub rpc_url: String,
}

impl AvalancheBlockchain {
    /// Create a new blockchain
    pub async fn create(
        wallet: &AvalancheWallet,
        subnet_id: Id,
        name: &str,
        vm_type: AvalancheVmType,
        vm_id: Id,
        genesis_data: Vec<u8>,
        check_acceptance: bool,
    ) -> Result<Self, AshError> {
        let tx_id = p::create_blockchain(
            wallet,
            subnet_id,
            genesis_data,
            vm_id,
            name,
            check_acceptance,
        )
        .await?;

        Ok(Self {
            id: tx_id,
            name: name.to_string(),
            subnet_id,
            vm_id,
            vm_type: vm_type.clone(),
            ..Default::default()
        })
    }

    /// Get an ethers Provider for this blockchain
    /// Only works for EVM blockchains
    pub fn get_ethers_provider(&self) -> Result<Provider<Http>, AshError> {
        match self.vm_type {
            AvalancheVmType::Coreth | AvalancheVmType::SubnetEVM => Ok(Provider::<Http>::try_from(
                self.rpc_url.clone(),
            )
            .map_err(|e| AvalancheBlockchainError::EthersProvider {
                blockchain_id: self.id.to_string(),
                msg: e.to_string(),
            })?),
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

    /// Get the blockchain ID as seen by the Warp Messenger
    pub async fn get_warp_blockchain_id(&self) -> Result<H256, AshError> {
        let warp_blockchain_id = match self.vm_type {
            AvalancheVmType::SubnetEVM => {
                let warp_messenger = WarpMessengerHttp::new(self)?;
                warp_messenger.get_blockchain_id().await?
            }
            _ => Err(AvalancheBlockchainError::OperationNotAllowed {
                blockchain_id: self.id.to_string(),
                vm_type: self.vm_type.to_string(),
                operation: "get Warp blockchain ID".to_string(),
            })?,
        };

        Ok(warp_blockchain_id)
    }

    /// Get the Warp messages sent from this blockchain between 2 blocks
    pub async fn get_warp_messages(
        &self,
        from_block: &str,
        to_block: &str,
    ) -> Result<Vec<WarpMessage>, AshError> {
        let warp_messages = match self.vm_type {
            AvalancheVmType::SubnetEVM => {
                let parsed_from_block = parse_evm_block_number(from_block)?;
                let parsed_to_block = parse_evm_block_number(to_block)?;
                let warp_messenger = WarpMessengerHttp::new(self)?;
                warp_messenger
                    .get_send_warp_message_logs(
                        parsed_from_block,
                        parsed_to_block,
                        None,
                        None,
                        None,
                    )
                    .await?
                    .into_iter()
                    .map(|log| WarpMessage {
                        unsigned_message: WarpUnsignedMessage::try_from_subnet_evm_log_data(
                            &log.data.to_vec()[..],
                        )
                        .or_else::<Result<WarpUnsignedMessage, AshError>, _>(|_| {
                            Ok(WarpUnsignedMessage::from(&log.data.to_vec()[..]))
                        })
                        .unwrap(),
                        verified_message: VerifiedWarpMessage::SubnetEVM(
                            SubnetEVMWarpMessage::from(log),
                        ),
                        ..Default::default()
                    })
                    .collect::<Vec<_>>()
            }
            _ => Err(AvalancheBlockchainError::OperationNotAllowed {
                blockchain_id: self.id.to_string(),
                vm_type: self.vm_type.to_string(),
                operation: "get Warp messages".to_string(),
            })?,
        };

        Ok(warp_messages)
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
    use super::*;
    use crate::avalanche::{vms::encode_genesis_data, AvalancheNetwork, AvalancheSubnet};
    use std::{env, fs, str::FromStr};

    const AVAX_EWOQ_PRIVATE_KEY: &str =
        "PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN";
    const SUBNET_EVM_VM_ID: &str = "spePNvBxaWSYL2tB5e2xMmMNBQkXMN8z2XEbz1ML2Aahatwoc";

    // Load the test network from the ASH_TEST_CONFIG file
    fn load_test_network() -> AvalancheNetwork {
        let config_path =
            env::var("ASH_TEST_AVAX_CONFIG").unwrap_or("tests/conf/default.yml".to_string());
        AvalancheNetwork::load("fuji", Some(&config_path)).unwrap()
    }

    // Using avalanche-network-runner to run a test network
    fn load_avalanche_network_runner() -> AvalancheNetwork {
        AvalancheNetwork::load("local", Some("tests/conf/avalanche-network-runner.yml")).unwrap()
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

    #[async_std::test]
    #[serial_test::serial]
    #[ignore]
    async fn test_avalanche_blockchain_create() {
        let mut local_network = load_avalanche_network_runner();
        let wallet = local_network
            .create_wallet_from_cb58(AVAX_EWOQ_PRIVATE_KEY)
            .unwrap();
        let genesis_json = fs::read_to_string("tests/genesis/subnet-evm.json").unwrap();
        let genesis_data = encode_genesis_data(AvalancheVmType::SubnetEVM, &genesis_json).unwrap();

        // Create an empty subnet
        let created_subnet = AvalancheSubnet::create(&wallet, true).await.unwrap();

        let created_blockchain = AvalancheBlockchain::create(
            &wallet,
            created_subnet.id,
            "testAvalancheBlockchainCreate",
            AvalancheVmType::SubnetEVM,
            Id::from_str(SUBNET_EVM_VM_ID).unwrap(),
            genesis_data,
            true,
        )
        .await
        .unwrap();

        local_network.update_subnets().unwrap();
        local_network.update_blockchains().unwrap();
        let network_subnet = local_network.get_subnet(created_subnet.id).unwrap();
        let mut network_blockchain = network_subnet
            .get_blockchain(created_blockchain.id)
            .unwrap()
            .clone();

        // Manually set the vm_type as it's not returned by the API
        network_blockchain.vm_type = AvalancheVmType::SubnetEVM;

        assert_eq!(created_blockchain, network_blockchain);
    }
}
