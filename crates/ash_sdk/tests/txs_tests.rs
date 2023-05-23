// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to test transactions

// Not testing as the underlying code is not working
// See https://github.com/ava-labs/avalanche-types-rs/blob/0f499e038ca01af09c5be207b6d144262222e659/src/wallet/p/import.rs#L98

use ash_sdk::avalanche::{
    jsonrpc::{avm, platformvm},
    txs::{p, x},
    AvalancheNetwork,
};
use async_std;

const AVAX_EWOQ_PRIVATE_KEY: &str = "PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN";

// Load the test network using avalanche-network-runner
fn load_test_network() -> AvalancheNetwork {
    AvalancheNetwork::load("local", Some("tests/conf/avalanche-network-runner.yml")).unwrap()
}

#[async_std::test]
#[ignore]
async fn test_txs_export_import_avax_to_pchain() {
    let local_network = load_test_network();
    let local_wallet = local_network
        .create_wallet_from_cb58(AVAX_EWOQ_PRIVATE_KEY)
        .unwrap();

    let xchain_rpc_url = &local_network.get_xchain().unwrap().rpc_url;
    let pchain_rpc_url = &local_network.get_pchain().unwrap().rpc_url;

    let init_xchain_balance = avm::get_balance(
        xchain_rpc_url,
        &local_wallet.xchain_wallet.x_address,
        "AVAX",
    )
    .unwrap();
    let init_pchain_balance =
        platformvm::get_balance(pchain_rpc_url, &local_wallet.pchain_wallet.p_address).unwrap();

    x::export_avax(
        &local_wallet,
        local_network.get_pchain().unwrap().id,
        100000000,
        true,
    )
    .await
    .unwrap();

    p::import_avax(&local_wallet, local_network.get_xchain().unwrap().id, true)
        .await
        .unwrap();

    let final_xchain_balance = avm::get_balance(
        xchain_rpc_url,
        &local_wallet.xchain_wallet.x_address,
        "AVAX",
    )
    .unwrap();
    let final_pchain_balance =
        platformvm::get_balance(pchain_rpc_url, &local_wallet.pchain_wallet.p_address).unwrap();

    assert_eq!(
        init_xchain_balance.balance + 100000000 - local_wallet.xchain_wallet.tx_fee,
        final_xchain_balance.balance
    );
    assert_eq!(
        init_pchain_balance.balance - 100000000 - local_wallet.pchain_wallet.tx_fee,
        final_pchain_balance.balance
    );
}
