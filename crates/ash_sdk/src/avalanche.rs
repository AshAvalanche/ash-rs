// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

pub mod blockchains;
pub mod jsonrpc;
pub mod nodes;
pub mod subnets;
pub mod txs;
pub mod vms;
pub mod wallets;
pub mod warp;

// Module that contains code to interact with Avalanche networks

use crate::{
    avalanche::{
        blockchains::AvalancheBlockchain,
        jsonrpc::{avm, platformvm},
        subnets::AvalancheSubnet,
        wallets::AvalancheWallet,
    },
    conf::AshConfig,
    errors::*,
};
use async_std::task;
use avalanche_types::{
    ids::{short::Id as ShortId, Id},
    jsonrpc::{avm::GetBalanceResult, platformvm::ApiOwner},
    key::secp256k1::address::avax_address_to_short_bytes,
    txs::utxo,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Avalanche Primary Network ID
/// This Subnet contains the P-Chain that is used for all Subnet operations
/// (the P-Chain ID is the same as the Primary Network ID)
pub const AVAX_PRIMARY_NETWORK_ID: &str = "11111111111111111111111111111111LpoYY";

/// Convert a human readable address to a ShortId
fn address_to_short_id(address: &str, chain_alias: &str) -> Result<ShortId, AshError> {
    let (_, addr_bytes) = avax_address_to_short_bytes(chain_alias, address).map_err(|e| {
        AvalancheNetworkError::InvalidAddress {
            address: address.to_string(),
            msg: e.to_string(),
        }
    })?;

    Ok(ShortId::from_slice(&addr_bytes))
}

/// Avalanche network
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheNetwork {
    /// Network name
    pub name: String,
    /// Primary Network ID
    #[serde(skip)]
    pub primary_network_id: Id,
    /// List of the network's Subnets
    pub subnets: Vec<AvalancheSubnet>,
}

impl Default for AvalancheNetwork {
    fn default() -> Self {
        Self {
            name: "mainnet".to_string(),
            primary_network_id: Id::from_str(AVAX_PRIMARY_NETWORK_ID).unwrap(),
            subnets: vec![],
        }
    }
}

impl AvalancheNetwork {
    /// Load an AvalancheNetwork from the configuration
    pub fn load(network_name: &str, config: Option<&str>) -> Result<AvalancheNetwork, AshError> {
        let ash_config = AshConfig::load(config)?;
        let mut avax_network = ash_config
            .avalanche_networks
            .iter()
            .find(|&avax_network| avax_network.name == network_name)
            .ok_or(ConfigError::NotFound {
                target_type: "network".to_string(),
                target_value: network_name.to_string(),
            })?
            .clone();

        avax_network.primary_network_id = Default::default();

        // Error if the Primary Network is not found or if the P-Chain is not found
        let _ = avax_network
            .get_subnet(avax_network.primary_network_id)?
            .get_blockchain(avax_network.primary_network_id)?;

        Ok(avax_network)
    }

    /// Get the P-Chain
    pub fn get_pchain(&self) -> Result<&AvalancheBlockchain, AshError> {
        let pchain = self
            .get_subnet(self.primary_network_id)?
            .get_blockchain(self.primary_network_id)?;
        Ok(pchain)
    }

    /// Get the C-Chain
    pub fn get_cchain(&self) -> Result<&AvalancheBlockchain, AshError> {
        let cchain = self
            .get_subnet(self.primary_network_id)?
            .get_blockchain_by_name("C-Chain")?;
        Ok(cchain)
    }

    /// Get the X-Chain
    pub fn get_xchain(&self) -> Result<&AvalancheBlockchain, AshError> {
        let xchain = self
            .get_subnet(self.primary_network_id)?
            .get_blockchain_by_name("X-Chain")?;
        Ok(xchain)
    }

    /// Update the AvalancheNetwork Subnets by querying an API endpoint
    pub fn update_subnets(&mut self) -> Result<(), AshError> {
        let rpc_url = &self.get_pchain()?.rpc_url;

        let api_subnets = platformvm::get_network_subnets(rpc_url, &self.name).map_err(|e| {
            RpcError::GetFailure {
                data_type: "Subnets".to_string(),
                target_type: "network".to_string(),
                target_value: self.name.clone(),
                msg: e.to_string(),
            }
        })?;

        // Update the Subnets with the ones returned by the API
        // If a Subnet is already present in the network (loaded from configuration),
        // update its control keys, threshold and type, but keep the blockchains unchanged
        // If the Subnet is not present in the network, add it
        self.subnets = api_subnets
            .iter()
            .map(|updated_subnet| {
                if let Some(existing_subnet) = self
                    .subnets
                    .iter()
                    .find(|existing_subnet| existing_subnet.id == updated_subnet.id)
                {
                    let mut subnet = existing_subnet.clone();
                    subnet.control_keys = updated_subnet.control_keys.clone();
                    subnet.threshold = updated_subnet.threshold;
                    subnet.subnet_type = updated_subnet.subnet_type.clone();
                    subnet
                } else {
                    updated_subnet.clone()
                }
            })
            .collect::<Vec<_>>();

        Ok(())
    }

    /// Get a Subnet of the network by its ID
    pub fn get_subnet(&self, id: Id) -> Result<&AvalancheSubnet, AshError> {
        self.subnets.iter().find(|&subnet| subnet.id == id).ok_or(
            AvalancheNetworkError::NotFound {
                network: self.name.clone(),
                target_type: "Subnet".to_string(),
                target_value: id.to_string(),
            }
            .into(),
        )
    }

    /// Update the AvalancheNetwork blockchains by querying an API endpoint
    /// This function will update the blockchains of all subnets
    pub fn update_blockchains(&mut self) -> Result<(), AshError> {
        let rpc_url = &self.get_pchain()?.rpc_url;

        let api_blockchains =
            platformvm::get_network_blockchains(rpc_url, &self.name).map_err(|e| {
                RpcError::GetFailure {
                    data_type: "blockchains".to_string(),
                    target_type: "network".to_string(),
                    target_value: self.name.clone(),
                    msg: e.to_string(),
                }
            })?;

        // For each Subnet, update the blockchains with the ones returned by the API
        // If a blockchain is already present in the Subnet (loaded from configuration),
        // update its ID and VM ID, but keep the RPC URL and VM type unchanged
        // If the blockchain is not present in the Subnet, add it
        self.subnets = self
            .subnets
            .iter()
            .map(|subnet| {
                let mut updated_subnet = subnet.clone();
                // Get the blockchains of the Subnet returned by the API
                let updated_chains = api_blockchains
                    .iter()
                    .filter(|chain| chain.subnet_id == updated_subnet.id)
                    .cloned()
                    .collect::<Vec<_>>();
                // For existing blockchains in the Subnet, update the ID, VM ID, and Subnet ID
                updated_subnet.blockchains = updated_subnet
                    .blockchains
                    .iter()
                    .map(|existing_chain| {
                        if let Some(updated_chain) = updated_chains
                            .iter()
                            .find(|updated_chain| updated_chain.name == existing_chain.name)
                        {
                            let mut blockchain = existing_chain.clone();
                            blockchain.id = updated_chain.id;
                            blockchain.vm_id = updated_chain.vm_id;
                            blockchain.subnet_id = updated_chain.subnet_id;
                            blockchain
                        } else {
                            existing_chain.clone()
                        }
                    })
                    .collect::<Vec<_>>();
                // Add the new blockchains to the Subnet
                let new_chains = updated_chains
                    .iter()
                    .filter(|updated_chain| {
                        !updated_subnet
                            .blockchains
                            .iter()
                            .any(|chain| chain.name == updated_chain.name)
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                updated_subnet.blockchains.extend(new_chains);
                updated_subnet
            })
            .collect::<Vec<_>>();

        Ok(())
    }

    /// Get a Blockchain of the network by its ID
    /// This function will search in all Subnets
    pub fn get_blockchain(&self, id: Id) -> Result<&AvalancheBlockchain, AshError> {
        self.subnets
            .iter()
            .find(|&subnet| subnet.get_blockchain(id).is_ok())
            .ok_or(AvalancheNetworkError::NotFound {
                network: self.name.clone(),
                target_type: "blockchain".to_string(),
                target_value: id.to_string(),
            })?
            .get_blockchain(id)
    }

    /// Get a Blockchain of the network by its name
    /// This function will search in all Subnets
    pub fn get_blockchain_by_name(&self, name: &str) -> Result<&AvalancheBlockchain, AshError> {
        self.subnets
            .iter()
            .find(|&subnet| subnet.get_blockchain_by_name(name).is_ok())
            .ok_or(AvalancheNetworkError::NotFound {
                network: self.name.clone(),
                target_type: "blockchain".to_string(),
                target_value: name.to_string(),
            })?
            .get_blockchain_by_name(name)
    }

    /// Update the validators of a Subnet by querying an API endpoint
    pub fn update_subnet_validators(&mut self, subnet_id: Id) -> Result<(), AshError> {
        let rpc_url = &self.get_pchain()?.rpc_url;

        let validators = platformvm::get_current_validators(rpc_url, subnet_id)?;

        // Replace the validators of the Subnet
        let mut subnet = self.get_subnet(subnet_id)?.clone();

        subnet.validators = validators;

        // Get the index of the Subnet
        let subnet_index = self
            .subnets
            .iter()
            .position(|subnet| subnet.id == subnet_id)
            .ok_or(AvalancheNetworkError::NotFound {
                network: self.name.clone(),
                target_type: "Subnet".to_string(),
                target_value: subnet_id.to_string(),
            })?;

        // Replace the Subnet
        self.subnets[subnet_index] = subnet;

        Ok(())
    }

    /// Check if the operation is allowed on the network
    /// If not, return an error
    fn check_operation_allowed(
        &self,
        operation: &str,
        network_blacklist: Vec<&str>,
    ) -> Result<(), AshError> {
        if network_blacklist.contains(&self.name.as_str()) {
            return Err(AvalancheNetworkError::OperationNotAllowed {
                operation: operation.to_string(),
                network: self.name.clone(),
            }
            .into());
        }

        Ok(())
    }

    /// Create a new wallet for the network from a CB58-encoded private key
    /// For security reasons, wallets cannot be created on the mainnet
    pub fn create_wallet_from_cb58(&self, private_key: &str) -> Result<AvalancheWallet, AshError> {
        self.check_operation_allowed("wallet creation", vec!["mainnet"])?;

        let xchain_url = &self.get_xchain()?.rpc_url;
        let pchain_url = &self.get_pchain()?.rpc_url;

        let wallet = task::block_on(async {
            AvalancheWallet::new_from_cb58(private_key, xchain_url, pchain_url).await
        })?;

        Ok(wallet)
    }

    /// Create a new wallet for the network from en hex-encoded private key
    /// For security reasons, wallets cannot be created on the mainnet
    pub fn create_wallet_from_hex(&self, private_key: &str) -> Result<AvalancheWallet, AshError> {
        self.check_operation_allowed("wallet creation", vec!["mainnet"])?;

        let xchain_url = &self.get_xchain()?.rpc_url;
        let pchain_url = &self.get_pchain()?.rpc_url;

        let wallet = task::block_on(async {
            AvalancheWallet::new_from_hex(private_key, xchain_url, pchain_url).await
        })?;

        Ok(wallet)
    }

    // Disabled for now because it has no concrete use case
    /// Create a new wallet for the network from a mnemonic
    /// For security reasons, wallets cannot be created on the mainnet
    // pub fn create_wallet_from_mnemonic_phrase(
    //     &self,
    //     phrase: &str,
    //     account_index: u32,
    // ) -> Result<AvalancheWallet, AshError> {
    //     self.check_operation_allowed("wallet creation", vec!["mainnet"])?;

    //     let xchain_url = &self.get_xchain()?.rpc_url;
    //     let pchain_url = &self.get_pchain()?.rpc_url;

    //     let wallet = task::block_on(async {
    //         AvalancheWallet::new_from_mnemonic_phrase(phrase, account_index, xchain_url, pchain_url)
    //             .await
    //     })?;

    //     Ok(wallet)
    // }

    /// Get the balance of an address on the X-Chain
    pub fn get_xchain_balance(
        &self,
        address: &str,
        asset_id: &str,
    ) -> Result<AvalancheXChainBalance, AshError> {
        let xchain_url = &self.get_xchain()?.rpc_url;

        let balance = avm::get_balance(xchain_url, address, asset_id)?;

        Ok(balance)
    }
}

/// Avalanche output owners
/// See https://docs.avax.network/specs/platform-transaction-serialization#secp256k1-output-owners-output
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AvalancheOutputOwners {
    pub locktime: u64,
    pub threshold: u32,
    pub addresses: Vec<String>,
}

impl From<ApiOwner> for AvalancheOutputOwners {
    fn from(api_owner: ApiOwner) -> Self {
        Self {
            locktime: api_owner.locktime,
            threshold: api_owner.threshold,
            addresses: api_owner.addresses,
        }
    }
}

/// Avalanche X-Chain balance
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AvalancheXChainBalance {
    pub balance: u64,
    #[serde(rename = "utxoIDs")]
    pub utxos_ids: Vec<utxo::Id>,
}

impl From<GetBalanceResult> for AvalancheXChainBalance {
    fn from(result: GetBalanceResult) -> Self {
        Self {
            balance: result.balance,
            utxos_ids: result.utxo_ids.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avalanche::{blockchains::AvalancheBlockchain, vms::AvalancheVmType};
    use std::env;

    const AVAX_FUJI_CCHAIN_ID: &str = "yH8D7ThNJkxmtkuv2jgBa4P1Rn3Qpr4pPr7QYNfcdoS6k6HWp";
    const AVAX_FUJI_XCHAIN_ID: &str = "2JVSBoinj9C2J33VntvzYtVJNZdN2NKiwwKjcumHUWEb5DbBrm";
    const AVAX_FUJI_EVM_ID: &str = "mgj786NP7uDwBCcq6YwThhaN8FLyybkCa4zBWTQbNgmK6k9A6";
    const AVAX_FUJI_DFK_SUBNET_ID: &str = "XHLRR9cvMtCR8KZsjU8nLxg1JbV7aS23AcLVeBMVHLKkSBriS";
    const AVAX_FUJI_DFK_CHAIN_ID: &str = "32sexHqc3tBQsik8h7WP5F2ruL5svqhX5opeTgXCRVX8HpbKF";

    // Using avalanche-network-runner to run a test network
    const AVAX_CB58_PRIVATE_KEY: &str =
        "PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN";
    const AVAX_HEX_PRIVATE_KEY: &str =
        "0x56289e99c94b6912bfc12adc093c9b51124f0dc54ac7a766b2bc5ccf558d8027";
    // This mnemonic phrase is not linked to the ewoq account
    // const AVAX_MNEMONIC_PHRASE: &str =
    //     "vehicle arrive more spread busy regret onion fame argue nice grocery humble vocal slot quit toss learn artwork theory fault tip belt cloth disorder";
    const AVAX_EWOQ_XCHAIN_ADDR: &str = "X-custom18jma8ppw3nhx5r4ap8clazz0dps7rv5u9xde7p";

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
    fn test_avalanche_network_load() {
        // Only test the fuji network as the mainnet network is structurally the same
        let fuji = load_test_network();
        assert_eq!(fuji.name, "fuji");
        assert_eq!(fuji.subnets.len(), 1);

        let AvalancheSubnet {
            id,
            control_keys,
            threshold,
            blockchains,
            ..
        } = &fuji.subnets[0];
        assert_eq!(id, &fuji.primary_network_id);
        assert_eq!(control_keys.len(), 0);
        assert_eq!(threshold, &0);
        assert_eq!(blockchains.len(), 3);

        let AvalancheBlockchain {
            id,
            name,
            vm_id,
            vm_type,
            ..
        } = &blockchains[1];
        assert_eq!(id, &Id::from_str(AVAX_FUJI_CCHAIN_ID).unwrap());
        assert_eq!(name, "C-Chain");
        assert_eq!(vm_id, &Id::from_str(AVAX_FUJI_EVM_ID).unwrap());
        assert_eq!(vm_type, &AvalancheVmType::Coreth);

        assert!(AvalancheNetwork::load("invalid", None).is_err());
    }

    #[test]
    fn test_avalanche_network_load_no_primary() {
        // Load the wrong.yml file which doesn't have the Primary Network
        // This should fail as the Primary Network is required
        assert!(
            AvalancheNetwork::load("no-primary-network", Some("tests/conf/wrong.yml")).is_err()
        );
    }

    #[test]
    fn test_avalanche_network_load_no_pchain() {
        // Load the wrong.yml file which doesn't have the P-Chain
        // This should fail as the P-Chain is required
        assert!(AvalancheNetwork::load("no-pchain", Some("tests/conf/wrong.yml")).is_err());
    }

    #[test]
    fn test_avalanche_network_get_chain() {
        let fuji = load_test_network();

        let pchain = fuji.get_pchain().unwrap();
        let cchain = fuji.get_cchain().unwrap();
        let xchain = fuji.get_xchain().unwrap();

        assert_eq!(pchain.id, fuji.primary_network_id);
        assert_eq!(pchain.name, "P-Chain");

        assert_eq!(cchain.id, Id::from_str(AVAX_FUJI_CCHAIN_ID).unwrap());
        assert_eq!(cchain.name, "C-Chain");

        assert_eq!(xchain.id, Id::from_str(AVAX_FUJI_XCHAIN_ID).unwrap());
        assert_eq!(xchain.name, "X-Chain");
    }

    #[test]
    fn test_avalanche_network_get_subnet() {
        let fuji = load_test_network();

        // Should never fail as self.primary_network_id should always be a valid key
        let primary_network = fuji.get_subnet(fuji.primary_network_id).unwrap();
        assert_eq!(primary_network.id, fuji.primary_network_id);
        assert_eq!(primary_network.blockchains.len(), 3);

        assert!(fuji
            .get_subnet(Id::from_str("7F8HV64nQER6ZupFNJwsYAKGbADv1T7pQYmoRPm1uVbeLMs7N").unwrap())
            .is_err());
    }

    #[test]
    fn test_avalanche_network_update_subnets() {
        let mut fuji = load_test_network();
        fuji.update_subnets().unwrap();

        // Test that the number of Subnets is greater than 1
        assert!(fuji.subnets.len() > 1);

        // Test that the Primary Network is still present
        // and that the P-Chain is still present
        let primary_network = fuji.get_subnet(fuji.primary_network_id).unwrap();
        assert_eq!(primary_network.id, fuji.primary_network_id);
        assert_eq!(primary_network.blockchains.len(), 3);
        assert!(primary_network
            .blockchains
            .iter()
            .any(|blockchain| blockchain.id == fuji.primary_network_id));

        // Test that the DFK Subnet is present
        let dfk_subnet = fuji
            .get_subnet(Id::from_str(AVAX_FUJI_DFK_SUBNET_ID).unwrap())
            .unwrap();
        assert_eq!(
            dfk_subnet.id,
            Id::from_str(AVAX_FUJI_DFK_SUBNET_ID).unwrap()
        );
    }

    #[test]
    fn test_avalanche_network_update_blockchains() {
        let mut fuji = load_test_network();
        fuji.update_subnets().unwrap();
        fuji.update_blockchains().unwrap();

        // Test that the Primary Network is still present
        assert!(fuji
            .subnets
            .iter()
            .any(|subnet| subnet.id == fuji.primary_network_id));

        // Test that the DFK Subnet contains the DFK chain
        let dfk_subnet = fuji
            .get_subnet(Id::from_str(AVAX_FUJI_DFK_SUBNET_ID).unwrap())
            .unwrap();
        assert!(dfk_subnet
            .blockchains
            .iter()
            .any(|blockchain| blockchain.id.to_string() == AVAX_FUJI_DFK_CHAIN_ID));
    }

    #[test]
    #[ignore]
    fn test_avalanche_network_update_blockchains_primary_network() {
        let mut local_network = AvalancheNetwork::load(
            "local-light",
            Some("tests/conf/avalanche-network-runner.yml"),
        )
        .unwrap();
        local_network.update_subnets().unwrap();
        local_network.update_blockchains().unwrap();

        // Test that the X-Chain and C-Chain are present
        // They are not defined in the local-light network and should be fetched from the API
        let primary_network = local_network
            .get_subnet(local_network.primary_network_id)
            .unwrap();
        assert!(primary_network
            .blockchains
            .iter()
            .any(|blockchain| blockchain.name.to_string() == "X-Chain"));
        assert!(primary_network
            .blockchains
            .iter()
            .any(|blockchain| blockchain.name.to_string() == "C-Chain"));
    }

    #[test]
    fn test_avalanche_network_update_subnet_validators() {
        // The method platform.getCurrentValidators is not available on QuickNode
        // Tempoary workaround: use Ankr public endpoint
        let mut fuji = AvalancheNetwork::load("fuji-ankr", None).unwrap();
        fuji.update_subnets().unwrap();
        fuji.update_subnet_validators(fuji.primary_network_id)
            .unwrap();

        // Test that the Primary Network is still present
        assert!(fuji
            .subnets
            .iter()
            .any(|subnet| subnet.id == fuji.primary_network_id));

        // Test that the Primary Network has validators
        let primary_network = fuji.get_subnet(fuji.primary_network_id).unwrap();
        assert!(primary_network.validators.len() > 0);
    }

    #[test]
    #[ignore]
    fn test_avalanche_network_create_wallet_from_cb58() {
        let local_network = load_avalanche_network_runner();

        let wallet = local_network
            .create_wallet_from_cb58(AVAX_CB58_PRIVATE_KEY)
            .unwrap();

        assert_eq!(wallet.private_key.to_cb58(), AVAX_CB58_PRIVATE_KEY);
    }

    #[test]
    #[ignore]
    fn test_avalanche_network_create_wallet_from_hex() {
        let local_network = load_avalanche_network_runner();

        let wallet = local_network
            .create_wallet_from_hex(AVAX_HEX_PRIVATE_KEY)
            .unwrap();

        assert_eq!(wallet.private_key.to_hex(), AVAX_HEX_PRIVATE_KEY);
    }

    // #[test]
    // #[ignore]
    // fn test_avalanche_network_create_wallet_from_mnemonic() {
    //     let local_network = load_avalanche_network_runner();

    //     let wallet = local_network
    //         .create_wallet_from_mnemonic_phrase(AVAX_MNEMONIC_PHRASE, 0)
    //         .unwrap();

    //     assert_eq!(
    //         wallet.private_key.to_hex(),
    //         "0xf88975995ec2c83832dc7fb071b78d015ffc1bc4474810c1f05f60738f4ffd26"
    //     );
    // }

    #[test]
    #[ignore]
    fn test_avalanche_network_get_xchain_balance() {
        let local_network = load_avalanche_network_runner();

        let balance = local_network
            .get_xchain_balance(AVAX_EWOQ_XCHAIN_ADDR, "AVAX")
            .unwrap();
        assert!(balance.balance > 0);
    }
}
