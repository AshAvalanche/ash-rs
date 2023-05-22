// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche wallets

use crate::{
    avalanche::{address_to_short_id, txs::x},
    errors::*,
};
use avalanche_types::{
    ids::Id,
    key::secp256k1::private_key::Key as PrivateKey,
    wallet::{Builder as WalletBuilder, Wallet},
};
use serde::{Deserialize, Serialize};

/// Avalanche wallet
#[derive(Debug, Clone)]
pub struct AvalancheWallet {
    pub private_key: PrivateKey,
    pub xchain_wallet: Wallet<PrivateKey>,
    pub pchain_wallet: Wallet<PrivateKey>,
}

impl AvalancheWallet {
    /// Create a new Avalanche wallet
    pub async fn new(
        private_key: PrivateKey,
        xchain_url: &str,
        pchain_url: &str,
    ) -> Result<Self, AshError> {
        // Create one wallet for each chain because the RPC URLs can be different
        let xchain_wallet = WalletBuilder::new(&private_key)
            .base_http_url(xchain_url.to_string())
            .build()
            .await
            .map_err(|e| AvalancheWalletError::CreationFailure(e.to_string()))?;
        let pchain_wallet = WalletBuilder::new(&private_key)
            .base_http_url(pchain_url.to_string())
            .build()
            .await
            .map_err(|e| AvalancheWalletError::CreationFailure(e.to_string()))?;

        Ok(Self {
            private_key,
            xchain_wallet,
            pchain_wallet,
        })
    }

    /// Create a new Avalanche wallet from a CB58-encoded private key
    pub async fn new_from_cb58(
        private_key: &str,
        xchain_url: &str,
        pchain_url: &str,
    ) -> Result<Self, AshError> {
        let private_key = PrivateKey::from_cb58(private_key)
            .map_err(|e| AvalancheWalletError::InvalidPrivateKey(e.to_string()))?;

        Self::new(private_key, xchain_url, pchain_url).await
    }

    /// Create a new Avalanche wallet from an hex-encoded private key
    pub async fn new_from_hex(
        private_key: &str,
        xchain_url: &str,
        pchain_url: &str,
    ) -> Result<Self, AshError> {
        let private_key = PrivateKey::from_hex(private_key)
            .map_err(|e| AvalancheWalletError::InvalidPrivateKey(e.to_string()))?;

        Self::new(private_key, xchain_url, pchain_url).await
    }

    // Disabled because it has no concrete use case
    /// Create a new Avalanche wallet from a mnemonic phrase
    /// The phrase must be 24 words long
    // pub async fn new_from_mnemonic_phrase(
    //     phrase: &str,
    //     account_index: u32,
    //     xchain_url: &str,
    //     pchain_url: &str,
    // ) -> Result<Self, AshError> {
    //     let private_key = PrivateKey::from_mnemonic_phrase(
    //         phrase,
    //         &format!("{}/0/{}", AVAX_ACCOUNT_DERIV_PATH, account_index),
    //     )
    //     .map_err(|e| AvalancheWalletError::InvalidPrivateKey(e.to_string()))?;

    //     Self::new(private_key, xchain_url, pchain_url).await
    // }

    /// Export the private key as a CB58-encoded string
    pub fn export_private_key_cb58(&self) -> String {
        self.private_key.to_cb58()
    }

    /// Export the private key as an hex-encoded string
    pub fn export_private_key_hex(&self) -> String {
        self.private_key.to_hex()
    }

    /// Transfer AVAX to a given address on the X-Chain
    /// Returns the transaction ID
    pub async fn transfer_avax_xchain(
        &self,
        to: &str,
        amount: u64,
        check_acceptance: bool,
    ) -> Result<Id, AshError> {
        let receiver = address_to_short_id(to, "X");
        let tx_id = x::transfer(self, receiver, amount, check_acceptance).await?;

        Ok(tx_id)
    }
}

/// Avalanche wallet information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvalancheWalletInfo {
    /// X-Chain address
    pub xchain_address: String,
    /// P-Chain address
    pub pchain_address: String,
    /// EVM address
    pub evm_address: String,
}

impl From<AvalancheWallet> for AvalancheWalletInfo {
    fn from(wallet: AvalancheWallet) -> Self {
        Self {
            xchain_address: wallet.xchain_wallet.x_address,
            pchain_address: wallet.pchain_wallet.p_address,
            evm_address: wallet.xchain_wallet.eth_address,
        }
    }
}

/// Generate a private key from random bytes
pub fn generate_private_key() -> Result<PrivateKey, AshError> {
    let private_key = PrivateKey::generate()
        .map_err(|e| AvalancheWalletError::PrivateKeyGenerationFailure(e.to_string()))?;

    Ok(private_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avalanche::AvalancheNetwork;
    use async_std;

    const AVAX_CB58_PRIVATE_KEY: &str =
        "PrivateKey-ewoqjP7PxY4yr3iLTpLisriqt94hdyDFNgchSxGGztUrTXtNN";
    const AVAX_HEX_PRIVATE_KEY: &str =
        "0x56289e99c94b6912bfc12adc093c9b51124f0dc54ac7a766b2bc5ccf558d8027";
    // This mnemonic phrase is not linked to the ewoq account
    // const AVAX_MNEMONIC_PHRASE: &str =
    //     "vehicle arrive more spread busy regret onion fame argue nice grocery humble vocal slot quit toss learn artwork theory fault tip belt cloth disorder";

    // Load the test network using avalanche-network-runner
    fn load_test_network() -> AvalancheNetwork {
        AvalancheNetwork::load("local", Some("tests/conf/avalanche-network-runner.yml")).unwrap()
    }

    #[async_std::test]
    #[ignore]
    async fn test_create_new_from_cb58() {
        let network = load_test_network();
        let wallet = AvalancheWallet::new_from_cb58(
            AVAX_CB58_PRIVATE_KEY,
            &network.get_xchain().unwrap().rpc_url,
            &network.get_pchain().unwrap().rpc_url,
        )
        .await
        .unwrap();

        assert_eq!(wallet.private_key.to_cb58(), AVAX_CB58_PRIVATE_KEY);
    }

    #[async_std::test]
    #[ignore]
    async fn test_create_new_from_hex() {
        let network = load_test_network();
        let wallet = AvalancheWallet::new_from_hex(
            AVAX_HEX_PRIVATE_KEY,
            &network.get_xchain().unwrap().rpc_url,
            &network.get_pchain().unwrap().rpc_url,
        )
        .await
        .unwrap();

        assert_eq!(wallet.private_key.to_hex(), AVAX_HEX_PRIVATE_KEY);
    }

    // #[async_std::test]
    // #[ignore]
    // async fn test_create_new_from_mnemonic_phrase() {
    //     let network = load_test_network();
    //     let wallet = AvalancheWallet::new_from_mnemonic_phrase(
    //         AVAX_MNEMONIC_PHRASE,
    //         0,
    //         &network.get_xchain().unwrap().rpc_url,
    //         &network.get_pchain().unwrap().rpc_url,
    //     )
    //     .await
    //     .unwrap();

    //     assert_eq!(
    //         wallet.private_key.to_hex(),
    //         "0xf88975995ec2c83832dc7fb071b78d015ffc1bc4474810c1f05f60738f4ffd26"
    //     );
    // }
}
