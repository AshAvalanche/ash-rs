// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche AVM API

use crate::{
    avalanche::{
        jsonrpc::{get_json_rpc_req_result, JsonRpcResponse},
        AvalancheXChainBalance,
    },
    errors::*,
    impl_json_rpc_response,
};
use avalanche_types::jsonrpc::{avm::*, ResponseError};

/// Info API endpoint
pub const AVAX_INFO_API_ENDPOINT: &str = "ext/info";

impl_json_rpc_response!(GetBalanceResponse, GetBalanceResult);

/// Get the balance of an address by querying the X-Chain API
pub fn get_balance(
    rpc_url: &str,
    address: &str,
    asset_id: &str,
) -> Result<AvalancheXChainBalance, RpcError> {
    let balance = get_json_rpc_req_result::<GetBalanceResponse, GetBalanceResult>(
        rpc_url,
        "avm.getBalance",
        Some(ureq::json!({
            "address": address,
            "assetID": asset_id,
        })),
    )?
    .into();

    Ok(balance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avalanche::AvalancheNetwork;

    // Using avalanche-network-runner to run a test network
    const AVAX_EWOQ_XCHAIN_ADDR: &str = "X-custom18jma8ppw3nhx5r4ap8clazz0dps7rv5u9xde7p";

    // Load the test network using avalanche-network-runner
    fn load_test_network() -> AvalancheNetwork {
        AvalancheNetwork::load("local", Some("tests/conf/avalanche-network-runner.yml")).unwrap()
    }

    #[test]
    #[ignore]
    fn test_get_balance() {
        let local_network = load_test_network();
        let rpc_url = &local_network.get_xchain().unwrap().rpc_url;

        let balance = get_balance(&rpc_url, AVAX_EWOQ_XCHAIN_ADDR, "AVAX").unwrap();
        assert!(balance.balance > 0);
    }
}
