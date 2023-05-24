// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to issue transactions on the X-Chain

use crate::{avalanche::wallets::AvalancheWallet, errors::*};
use avalanche_types::{ids::Id, wallet::p};

/// Create a new subnet
/// TODO: Add control keys and threshold as parameters
/// See: https://github.com/ava-labs/avalanche-types-rs/pull/76
pub async fn create_subnet(
    wallet: &AvalancheWallet,
    check_acceptance: bool,
) -> Result<Id, AshError> {
    let tx_id = p::create_subnet::Tx::new(&wallet.pchain_wallet.p())
        .check_acceptance(check_acceptance)
        .issue()
        .await
        .map_err(|e| AvalancheWalletError::IssueTx {
            blockchain_name: "P-Chain".to_string(),
            tx_type: "create_subnet".to_string(),
            msg: e.to_string(),
        })?;

    Ok(tx_id)
}

/// Create a new blockchain
pub async fn create_blockchain(
    wallet: &AvalancheWallet,
    subnet_id: Id,
    genesis_data: Vec<u8>,
    vm_id: Id,
    name: &str,
    check_acceptance: bool,
) -> Result<Id, AshError> {
    let tx_id = p::create_chain::Tx::new(&wallet.pchain_wallet.p())
        .subnet_id(subnet_id)
        .genesis_data(genesis_data)
        .vm_id(vm_id)
        .chain_name(name.to_string())
        .check_acceptance(check_acceptance)
        .issue()
        .await
        .map_err(|e| AvalancheWalletError::IssueTx {
            blockchain_name: "P-Chain".to_string(),
            tx_type: "create_blockchain".to_string(),
            msg: e.to_string(),
        })?;

    Ok(tx_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avalanche::{
        vms::{encode_genesis_data, AvalancheVmType},
        AvalancheNetwork,
    };
    use std::{fs, str::FromStr};

    const AVAX_EWOQ_PRIVATE_KEY: &str =
        "PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN";
    const AVAX_LOCAL_PCHAIN_ADDR: &str = "P-custom18jma8ppw3nhx5r4ap8clazz0dps7rv5u9xde7p";
    const SUBNET_EVM_VM_ID: &str = "spePNvBxaWSYL2tB5e2xMmMNBQkXMN8z2XEbz1ML2Aahatwoc";

    // Load the test network using avalanche-network-runner
    fn load_test_network() -> AvalancheNetwork {
        AvalancheNetwork::load("local", Some("tests/conf/avalanche-network-runner.yml")).unwrap()
    }

    #[async_std::test]
    #[serial_test::serial]
    #[ignore]
    async fn test_create_subnet() {
        let mut local_network = load_test_network();
        let local_wallet = local_network
            .create_wallet_from_cb58(AVAX_EWOQ_PRIVATE_KEY)
            .unwrap();

        let tx_id = create_subnet(&local_wallet, true).await.unwrap();

        // Check that the Subnet was created
        // The Subnet has the same ID as the transaction that created it
        local_network.update_subnets().unwrap();
        let subnet = local_network.get_subnet(tx_id).unwrap();

        assert_eq!(subnet.threshold, 1);
        assert_eq!(subnet.control_keys.len(), 1);
        assert_eq!(subnet.control_keys[0], AVAX_LOCAL_PCHAIN_ADDR);
    }

    #[async_std::test]
    #[serial_test::serial]
    #[ignore]
    async fn test_create_blockchain() {
        let mut local_network = load_test_network();
        let local_wallet = local_network
            .create_wallet_from_cb58(AVAX_EWOQ_PRIVATE_KEY)
            .unwrap();
        let genesis_str = fs::read_to_string("tests/genesis/subnet-evm.json").unwrap();
        let genesis_data = encode_genesis_data(AvalancheVmType::SubnetEVM, &genesis_str).unwrap();

        let subnet_id = create_subnet(&local_wallet, true).await.unwrap();
        let tx_id = create_blockchain(
            &local_wallet,
            subnet_id,
            genesis_data,
            Id::from_str(SUBNET_EVM_VM_ID).unwrap(),
            "testCreateBlockchain",
            true,
        )
        .await
        .unwrap();

        // Check that the Blockchain was created
        // The Blockchain has the same ID as the transaction that created it
        local_network.update_subnets().unwrap();
        local_network.update_blockchains().unwrap();

        let subnet = local_network.get_subnet(subnet_id).unwrap();
        let blockchain = subnet.get_blockchain(&tx_id.to_string()).unwrap();

        assert_eq!(blockchain.name, "testCreateBlockchain");
        assert_eq!(blockchain.vm_id, Id::from_str(SUBNET_EVM_VM_ID).unwrap());
    }
}
