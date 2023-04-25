// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to test transactions

// Not testing as the underlying code is not working
// See https://github.com/ava-labs/avalanche-types-rs/blob/0f499e038ca01af09c5be207b6d144262222e659/src/wallet/p/import.rs#L98

// use ash::avalanche::{txs::x, AvalancheNetwork};
// use async_std;
// use avalanche_types::{
//     ids::short::Id as ShortId, jsonrpc::client,
//     key::secp256k1::address::avax_address_to_short_bytes,
// };
// use serial_test::serial;

// const AVAX_EWOQ_PRIVATE_KEY: &str = "PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN";

// // Load the test network using avalanche-network-runner
// fn load_test_network() -> AvalancheNetwork {
//     AvalancheNetwork::load("local", Some("tests/conf/avalanche-network-runner.yml")).unwrap()
// }

// // Convert a human readable address to a ShortId
// fn address_to_short_id(address: &str, chain_alias: &str) -> ShortId {
//     let (_, addr_bytes) = avax_address_to_short_bytes(chain_alias, address).unwrap();
//     ShortId::from_slice(&addr_bytes)
// }

// // Get the balance of an address on the X-Chain
// async fn get_xchain_balance(address: &str) -> u64 {
//     client::x::get_balance(&load_test_network().get_xchain().unwrap().rpc_url, address)
//         .await
//         .unwrap()
//         .result
//         .unwrap()
//         .balance
// }

// async fn get_pchain_balance(address: &str) -> u64 {
//     client::p::get_balance(&load_test_network().get_pchain().unwrap().rpc_url, address)
//         .await
//         .unwrap()
//         .result
//         .unwrap()
//         .balance
// }

// #[async_std::test]
// #[ignore]
// #[serial]
// async fn test_txs_export_import_avax_to_xchain() {
//     let local_network = load_test_network();
//     let local_wallet = local_network
//         .create_wallet(AVAX_EWOQ_PRIVATE_KEY)
//         .await
//         .unwrap();

//     let init_xchain_balance = get_xchain_balance(&local_wallet.x_wallet.x_address).await;
//     let init_pchain_balance = get_pchain_balance(&local_wallet.p_wallet.p_address).await;

//     p::export_avax_to_xchain(
//         &local_wallet,
//         local_network.get_xchain().unwrap().id,
//         100000000,
//         true,
//     )
//     .await
//     .unwrap();

//     x::import_avax_from_pchain(&local_wallet, local_network.get_pchain().unwrap().id, true)
//         .await
//         .unwrap();

//     let final_xchain_balance = get_xchain_balance(&local_wallet.x_wallet.x_address).await;
//     let final_pchain_balance = get_pchain_balance(&local_wallet.p_wallet.p_address).await;

//     assert_eq!(
//         init_xchain_balance + 100000000 - local_wallet.x_wallet.tx_fee,
//         final_xchain_balance
//     );
//     assert_eq!(
//         init_pchain_balance - 100000000 - local_wallet.p_wallet.tx_fee,
//         final_pchain_balance
//     );
// }
