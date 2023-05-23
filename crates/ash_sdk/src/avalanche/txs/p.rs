// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to issue transactions on the X-Chain

use crate::{avalanche::wallets::AvalancheWallet, errors::*};
use avalanche_types::{ids::Id, wallet::p::import};

/// Import AVAX from another chain (to the P-Chain)
// See https://github.com/ava-labs/avalanche-types-rs/blob/0f499e038ca01af09c5be207b6d144262222e659/src/wallet/p/import.rs#L98
pub async fn import_avax(
    wallet: &AvalancheWallet,
    source_chain_id: Id,
    check_acceptance: bool,
) -> Result<Id, AshError> {
    let tx_id = import::Tx::new(&wallet.pchain_wallet.p())
        .source_blockchain_id(source_chain_id)
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
