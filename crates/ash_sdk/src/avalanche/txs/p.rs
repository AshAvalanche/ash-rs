// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to issue transactions on the X-Chain

use crate::{avalanche::wallets::AvalancheWallet, errors::*};
use avalanche_types::{
    ids::{node::Id as NodeId, Id},
    wallet::p,
};
use chrono::{DateTime, Duration, Utc};

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
            msg: format!("failed to create subnet: {e}"),
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
            msg: format!("failed to create blockchain on Subnet {subnet_id}: {e}"),
        })?;

    Ok(tx_id)
}

/// Add a validator to the Primary Network
pub async fn add_permissioned_subnet_validator(
    wallet: &AvalancheWallet,
    subnet_id: Id,
    node_id: NodeId,
    weight: u64,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    check_acceptance: bool,
) -> Result<Id, AshError> {
    let (tx_id, success) = p::add_subnet_validator::Tx::new(&wallet.pchain_wallet.p())
        .subnet_id(subnet_id)
        .node_id(node_id)
        .weight(weight)
        .start_time(start_time)
        .end_time(end_time)
        .check_acceptance(check_acceptance)
        .poll_initial_wait(Duration::seconds(1).to_std().unwrap())
        .issue()
        .await
        .map_err(|e| AvalancheWalletError::IssueTx {
            blockchain_name: "P-Chain".to_string(),
            tx_type: "add_subnet_validator".to_string(),
            msg: format!("failed to add '{node_id}' as validator to Subnet '{subnet_id}': {e}"),
        })?;

    // Check if the validator was successfully added
    // If the validator is already a validator, tx_id is empty and success false
    match success {
        true => Ok(tx_id),
        false => {
            if Id::is_empty(&tx_id) {
                Err(AvalancheWalletError::IssueTx {
                    blockchain_name: "P-Chain".to_string(),
                    tx_type: "add_validator".to_string(),
                    msg: format!("'{node_id}' is already a validator to Subnet '{subnet_id}'"),
                }
                .into())
            } else {
                // This is theoretically unreachable
                Err(AvalancheWalletError::IssueTx {
                    blockchain_name: "P-Chain".to_string(),
                    tx_type: "add_validator".to_string(),
                    msg: format!(
                        "failed to add '{node_id}' as validator to Subnet '{subnet_id}': Unknown error"
                    ),
                }
                .into())
            }
        }
    }
}

/// Add a validator to the Avalanche Primary Network
pub async fn add_avalanche_validator(
    wallet: &AvalancheWallet,
    node_id: NodeId,
    stake_amount: u64,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    reward_fee_percent: u32,
    check_acceptance: bool,
) -> Result<Id, AshError> {
    let (tx_id, success) = p::add_validator::Tx::new(&wallet.pchain_wallet.p())
        .node_id(node_id)
        .stake_amount(stake_amount)
        .start_time(start_time)
        .end_time(end_time)
        .reward_fee_percent(reward_fee_percent)
        .check_acceptance(check_acceptance)
        .poll_initial_wait(Duration::seconds(1).to_std().unwrap())
        .issue()
        .await
        .map_err(|e| AvalancheWalletError::IssueTx {
            blockchain_name: "P-Chain".to_string(),
            tx_type: "add_validator".to_string(),
            msg: format!("failed to add '{node_id}' as Avalanche validator: {e}"),
        })?;

    // Check if the validator was successfully added
    // If the validator is already a validator, tx_id is empty and success false
    match success {
        true => Ok(tx_id),
        false => {
            if Id::is_empty(&tx_id) {
                Err(AvalancheWalletError::IssueTx {
                    blockchain_name: "P-Chain".to_string(),
                    tx_type: "add_validator".to_string(),
                    msg: format!("'{node_id}' is already an Avalanche validator"),
                }
                .into())
            } else {
                // This is theoretically unreachable
                Err(AvalancheWalletError::IssueTx {
                    blockchain_name: "P-Chain".to_string(),
                    tx_type: "add_validator".to_string(),
                    msg: format!("failed to add '{node_id}' as Avalanche validator: Unknown error"),
                }
                .into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avalanche::{
        vms::{encode_genesis_data, AvalancheVmType, subnet_evm::AVAX_SUBNET_EVM_ID},
        AvalancheNetwork,
    };
    use chrono::Duration;
    use std::{fs, str::FromStr};

    const AVAX_EWOQ_PRIVATE_KEY: &str =
        "PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN";
    const NETWORK_RUNNER_PCHAIN_ADDR: &str = "P-custom18jma8ppw3nhx5r4ap8clazz0dps7rv5u9xde7p";
    const NETWORK_RUNNER_NODE_ID: &str = "NodeID-7Xhw2mDxuDS44j42TCB6U5579esbSt3Lg";

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
        assert_eq!(subnet.control_keys[0], NETWORK_RUNNER_PCHAIN_ADDR);
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

        // Create a Subnet to create the Blockchain on
        let subnet_id = create_subnet(&local_wallet, true).await.unwrap();

        let tx_id = create_blockchain(
            &local_wallet,
            subnet_id,
            genesis_data,
            Id::from_str(AVAX_SUBNET_EVM_ID).unwrap(),
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
        let blockchain = subnet.get_blockchain(tx_id).unwrap();

        assert_eq!(blockchain.name, "testCreateBlockchain");
        assert_eq!(blockchain.vm_id, Id::from_str(AVAX_SUBNET_EVM_ID).unwrap());
    }

    #[async_std::test]
    #[serial_test::serial]
    #[ignore]
    async fn test_add_validators() {
        let mut local_network = load_test_network();
        let local_wallet = local_network
            .create_wallet_from_cb58(AVAX_EWOQ_PRIVATE_KEY)
            .unwrap();

        // Create a Subnet
        let subnet_id = create_subnet(&local_wallet, true).await.unwrap();

        // Add a validator to the Subnet
        // The validator is added with a start time of 20 seconds from now and an end time of 24 hours from now
        let start_time = Utc::now() + Duration::seconds(20);
        let end_time = Utc::now() + Duration::seconds(86420);
        add_permissioned_subnet_validator(
            &local_wallet,
            subnet_id,
            NodeId::from_str(NETWORK_RUNNER_NODE_ID).unwrap(),
            100,
            start_time,
            end_time,
            true,
        )
        .await
        .unwrap();

        // Check that the validator was added
        local_network.update_subnets().unwrap();
        local_network.update_subnet_validators(subnet_id).unwrap();

        let subnet_validator = local_network
            .get_subnet(subnet_id)
            .unwrap()
            .get_validator(NodeId::from_str(NETWORK_RUNNER_NODE_ID).unwrap());

        assert!(subnet_validator.is_ok());
        assert_eq!(subnet_validator.unwrap().weight, Some(100));

        // Try to add a validator that already exists on the Primary Network
        let avalanche_validator = add_avalanche_validator(
            &local_wallet,
            NodeId::from_str(NETWORK_RUNNER_NODE_ID).unwrap(),
            1 * 1_000_000_000,
            start_time,
            end_time,
            2,
            true,
        )
        .await;

        assert!(avalanche_validator.is_err());
        assert_eq!(
            avalanche_validator.err(),
            Some(AshError::AvalancheWalletError(
                AvalancheWalletError::IssueTx {
                    blockchain_name: "P-Chain".to_string(),
                    tx_type: "add_validator".to_string(),
                    msg: format!(
                        "'{}' is already an Avalanche validator",
                        NodeId::from_str(NETWORK_RUNNER_NODE_ID).unwrap()
                    ),
                }
            ))
        )
    }
}
