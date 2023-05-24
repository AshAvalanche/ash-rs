// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to issue transactions on the X-Chain

use crate::{avalanche::wallets::AvalancheWallet, errors::*};
use avalanche_types::{ids::Id, wallet::p::create_subnet};

/// Create a new subnet
/// TODO: Add control keys and threshold as parameters
/// See: https://github.com/ava-labs/avalanche-types-rs/pull/76
pub async fn create_subnet(
    wallet: &AvalancheWallet,
    check_acceptance: bool,
) -> Result<Id, AshError> {
    let tx_id = create_subnet::Tx::new(&wallet.pchain_wallet.p())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avalanche::AvalancheNetwork;
    use async_std;

    const AVAX_EWOQ_PRIVATE_KEY: &str =
        "PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN";
    const AVAX_LOCAL_PCHAIN_ADDR: &str = "P-custom18jma8ppw3nhx5r4ap8clazz0dps7rv5u9xde7p";

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
        let subnet = local_network.get_subnet(&tx_id.to_string()).unwrap();

        assert_eq!(subnet.threshold, 1);
        assert_eq!(subnet.control_keys.len(), 1);
        assert_eq!(subnet.control_keys[0], AVAX_LOCAL_PCHAIN_ADDR);
    }
}
