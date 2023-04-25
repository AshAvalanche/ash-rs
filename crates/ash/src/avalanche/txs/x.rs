// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to issue transactions on the X-Chain

use crate::{avalanche::wallets::AvalancheWallet, errors::*};
use avalanche_types::{
    ids::{short::Id as ShortId, Id},
    wallet::x::{export, import, transfer},
};

/// Transfer AVAX from a wallet to the receiver
pub async fn transfer(
    wallet: &AvalancheWallet,
    receiver: ShortId,
    amount: u64,
    check_acceptance: bool,
) -> Result<Id, AshError> {
    let tx_id = transfer::Tx::new(&wallet.x_wallet.x())
        .receiver(receiver)
        .amount(amount)
        .check_acceptance(check_acceptance)
        .issue()
        .await
        .map_err(|e| AvalancheWalletError::IssueTx {
            blockchain_name: "X-Chain".to_string(),
            tx_type: "transfer".to_string(),
            msg: e.to_string(),
        })?;

    Ok(tx_id)
}

/// Export AVAX to the P-Chain (from the X-Chain)
// Not public as the underlying code is not working
// See https://github.com/ava-labs/avalanche-types-rs/blob/0f499e038ca01af09c5be207b6d144262222e659/src/wallet/p/import.rs#L98
#[allow(dead_code)]
async fn export_avax_to_pchain(
    wallet: &AvalancheWallet,
    pchain_id: Id,
    amount: u64,
    check_acceptance: bool,
) -> Result<Id, AshError> {
    let tx_id = export::Tx::new(&wallet.x_wallet.x())
        .destination_blockchain_id(pchain_id)
        .amount(amount)
        .check_acceptance(check_acceptance)
        .issue()
        .await
        .map_err(|e| AvalancheWalletError::IssueTx {
            blockchain_name: "X-Chain".to_string(),
            tx_type: "export".to_string(),
            msg: e.to_string(),
        })?;

    Ok(tx_id)
}

/// Import AVAX from the P-Chain (to the X-Chain)
// Not public as the underlying code is not working
// See https://github.com/ava-labs/avalanche-types-rs/blob/0f499e038ca01af09c5be207b6d144262222e659/src/wallet/p/import.rs#L98
#[allow(dead_code)]
async fn import_avax_from_pchain(
    wallet: &AvalancheWallet,
    pchain_id: Id,
    check_acceptance: bool,
) -> Result<Id, AshError> {
    let tx_id = import::Tx::new(&wallet.x_wallet.x())
        .source_blockchain_id(pchain_id)
        .check_acceptance(check_acceptance)
        .issue()
        .await
        .map_err(|e| AvalancheWalletError::IssueTx {
            blockchain_name: "X-Chain".to_string(),
            tx_type: "import".to_string(),
            msg: e.to_string(),
        })?;

    Ok(tx_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avalanche::AvalancheNetwork;
    use async_std;
    use avalanche_types::{
        jsonrpc::client::x, key::secp256k1::address::avax_address_to_short_bytes,
    };
    use serial_test::serial;

    const AVAX_EWOQ_PRIVATE_KEY: &str =
        "PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN";
    const AVAX_LOCAL_XCHAIN_ADDR: &str = "X-custom1w44zzlzf68gwaskce2s4r82t5u08pje5mhq2en";

    // Load the test network using avalanche-network-runner
    fn load_test_network() -> AvalancheNetwork {
        AvalancheNetwork::load("local", Some("tests/conf/avalanche-network-runner.yml")).unwrap()
    }

    // Convert a human readable address to a ShortId
    fn address_to_short_id(address: &str, chain_alias: &str) -> ShortId {
        let (_, addr_bytes) = avax_address_to_short_bytes(chain_alias, address).unwrap();
        ShortId::from_slice(&addr_bytes)
    }

    // Get the balance of an address on the X-Chain
    async fn get_xchain_balance(address: &str) -> u64 {
        x::get_balance(&load_test_network().get_xchain().unwrap().rpc_url, address)
            .await
            .unwrap()
            .result
            .unwrap()
            .balance
    }

    #[async_std::test]
    #[ignore]
    #[serial]
    async fn test_transfer() {
        let local_network = load_test_network();
        let local_wallet = local_network
            .create_wallet(AVAX_EWOQ_PRIVATE_KEY)
            .await
            .unwrap();
        let init_balance = get_xchain_balance(AVAX_LOCAL_XCHAIN_ADDR).await;

        transfer(
            &local_wallet,
            address_to_short_id(AVAX_LOCAL_XCHAIN_ADDR, "X"),
            100000000,
            true,
        )
        .await
        .unwrap();

        let final_balance = get_xchain_balance(AVAX_LOCAL_XCHAIN_ADDR).await;

        assert_eq!(init_balance + 100000000, final_balance)
    }

    // Not testing as the underlying code is not working
    // See https://github.com/ava-labs/avalanche-types-rs/blob/0f499e038ca01af09c5be207b6d144262222e659/src/wallet/p/import.rs#L98
    // #[async_std::test]
    // #[ignore]
    // #[serial]
    // async fn test_export_avax_to_pchain() {
    //     let local_network = load_test_network();
    //     let local_wallet = local_network
    //         .create_wallet(AVAX_EWOQ_PRIVATE_KEY)
    //         .await
    //         .unwrap();
    //     let init_balance = get_xchain_balance(&local_wallet.x_wallet.x_address).await;

    //     export_avax_to_pchain(
    //         &local_wallet,
    //         local_network.get_pchain().unwrap().id,
    //         100000000,
    //         true,
    //     )
    //     .await
    //     .unwrap();

    //     let final_balance = get_xchain_balance(&local_wallet.x_wallet.x_address).await;

    //     assert_eq!(
    //         init_balance - 100000000 - local_wallet.x_wallet.tx_fee,
    //         final_balance
    //     );
    //     // assert_eq!(init_pchain_balance + 100000000, final_pchain_balance);
    // }
}
