// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to issue transactions on the P-Chain

use crate::{avalanche::wallets::AvalancheWallet, errors::*};
use avalanche_types::{
    ids::Id,
    wallet::p::{export, import},
};

/// Import AVAX from the X-Chain (to the P-Chain)
pub async fn import_avax_from_xchain(
    wallet: &AvalancheWallet,
    xchain_id: Id,
    check_acceptance: bool,
) -> Result<Id, AshError> {
    let tx_id = import::Tx::new(&wallet.p_wallet.p())
        .source_blockchain_id(xchain_id)
        .check_acceptance(check_acceptance)
        .issue()
        .await
        .map_err(|e| AvalancheWalletError::IssueTx {
            blockchain_name: "P-Chain".to_string(),
            tx_type: "import".to_string(),
            msg: e.to_string(),
        })?;

    Ok(tx_id)
}

/// Export AVAX to the X-Chain (from the P-Chain)
pub async fn export_avax_to_xchain(
    wallet: &AvalancheWallet,
    xchain_id: Id,
    amount: u64,
    check_acceptance: bool,
) -> Result<Id, AshError> {
    let tx_id = export::Tx::new(&wallet.p_wallet.p())
        .destination_blockchain_id(xchain_id)
        .amount(amount)
        .check_acceptance(check_acceptance)
        .issue()
        .await
        .map_err(|e| AvalancheWalletError::IssueTx {
            blockchain_name: "P-Chain".to_string(),
            tx_type: "export".to_string(),
            msg: e.to_string(),
        })?;

    Ok(tx_id)
}

// Not testing as the underlying code is not working
// See https://github.com/ava-labs/avalanche-types-rs/blob/0f499e038ca01af09c5be207b6d144262222e659/src/wallet/p/import.rs#L98

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::avalanche::AvalancheNetwork;
//     use async_std;
//     use avalanche_types::jsonrpc::client::p;
//     use serial_test::serial;

//     const AVAX_EWOQ_PRIVATE_KEY: &str =
//         "PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN";

//     // Load the test network using avalanche-network-runner
//     fn load_test_network() -> AvalancheNetwork {
//         AvalancheNetwork::load("local", Some("tests/conf/avalanche-network-runner.yml")).unwrap()
//     }

//     async fn get_pchain_balance(address: &str) -> u64 {
//         p::get_balance(&load_test_network().get_pchain().unwrap().rpc_url, address)
//             .await
//             .unwrap()
//             .result
//             .unwrap()
//             .balance
//     }

// #[async_std::test]
// #[ignore]
// #[serial]
// async fn test_export_avax_to_xchain() {
//     let local_network = load_test_network();
//     let local_wallet = local_network
//         .create_wallet(AVAX_EWOQ_PRIVATE_KEY)
//         .await
//         .unwrap();
//     let init_balance = get_pchain_balance(&local_wallet.p_wallet.p_address).await;

//     export_avax_to_xchain(
//         &local_wallet,
//         local_network.get_xchain().unwrap().id,
//         100000000,
//         true,
//     )
//     .await
//     .unwrap();

//     let final_balance = get_pchain_balance(&local_wallet.p_wallet.p_address).await;

//     assert_eq!(
//         init_balance - 100000000 - local_wallet.p_wallet.tx_fee,
//         final_balance
//     );
//     // assert_eq!(init_pchain_balance + 100000000, final_pchain_balance);
// }
// }
