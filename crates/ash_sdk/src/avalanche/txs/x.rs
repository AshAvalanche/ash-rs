// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to issue transactions on the X-Chain

use crate::{avalanche::wallets::AvalancheWallet, errors::*};
use avalanche_types::{
    ids::{short::Id as ShortId, Id},
    wallet::x::transfer,
};

/// Transfer AVAX from a wallet to the receiver
pub async fn transfer_avax(
    wallet: &AvalancheWallet,
    receiver: ShortId,
    amount: u64,
    check_acceptance: bool,
) -> Result<Id, AshError> {
    let tx_id = transfer::Tx::new(&wallet.xchain_wallet.x())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avalanche::{address_to_short_id, jsonrpc::avm::get_balance, AvalancheNetwork};

    const AVAX_EWOQ_PRIVATE_KEY: &str =
        "PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN";
    const AVAX_LOCAL_XCHAIN_ADDR: &str = "X-custom1w44zzlzf68gwaskce2s4r82t5u08pje5mhq2en";

    // Load the test network using avalanche-network-runner
    fn load_test_network() -> AvalancheNetwork {
        AvalancheNetwork::load("local", Some("tests/conf/avalanche-network-runner.yml")).unwrap()
    }

    #[async_std::test]
    #[serial_test::serial]
    #[ignore]
    async fn test_transfer() {
        let local_network = load_test_network();
        let local_wallet = local_network
            .create_wallet_from_cb58(AVAX_EWOQ_PRIVATE_KEY)
            .unwrap();
        let rpc_url = &local_network.get_xchain().unwrap().rpc_url;
        let init_balance = get_balance(rpc_url, AVAX_LOCAL_XCHAIN_ADDR, "AVAX").unwrap();

        transfer_avax(
            &local_wallet,
            address_to_short_id(AVAX_LOCAL_XCHAIN_ADDR, "X").unwrap(),
            100000000,
            true,
        )
        .await
        .unwrap();

        let final_balance = get_balance(rpc_url, AVAX_LOCAL_XCHAIN_ADDR, "AVAX").unwrap();

        assert_eq!(init_balance.balance + 100000000, final_balance.balance)
    }
}
