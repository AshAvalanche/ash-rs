// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche wallets

use crate::errors::*;
use avalanche_types::{
    key::secp256k1::private_key::Key as PrivateKey,
    wallet::{Builder as WalletBuilder, Wallet},
};

/// Avalanche wallet
#[derive(Debug, Clone)]
pub struct AvalancheWallet {
    pub private_key: PrivateKey,
    pub x_wallet: Wallet<PrivateKey>,
    pub p_wallet: Wallet<PrivateKey>,
}

impl AvalancheWallet {
    /// Create a new Avalanche wallet
    pub async fn new(
        private_key: &str,
        xchain_url: &str,
        pchain_url: &str,
    ) -> Result<Self, AshError> {
        let private_key = PrivateKey::from_cb58(private_key)
            .map_err(|e| AvalancheWalletError::InvalidPrivateKey(e.to_string()))?;
        let x_wallet = WalletBuilder::new(&private_key)
            .base_http_url(xchain_url.to_string())
            .build()
            .await
            .map_err(|e| AvalancheWalletError::CreationFailure(e.to_string()))?;
        let p_wallet = WalletBuilder::new(&private_key)
            .base_http_url(pchain_url.to_string())
            .build()
            .await
            .map_err(|e| AvalancheWalletError::CreationFailure(e.to_string()))?;

        Ok(Self {
            private_key,
            x_wallet,
            p_wallet,
        })
    }
}
