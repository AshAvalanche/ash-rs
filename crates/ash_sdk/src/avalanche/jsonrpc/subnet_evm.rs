// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche Subnet-EVM API

use crate::{
    avalanche::jsonrpc::{get_json_rpc_req_result, JsonRpcResponse},
    errors::*,
    impl_json_rpc_response,
};
use avalanche_types::{ids::Id, jsonrpc::ResponseError};
use ethers::types::Bytes;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct WarpGetSignatureResponse {
    pub jsonrpc: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: u32,
    pub result: Option<Bytes>,
    pub error: Option<ResponseError>,
}

impl_json_rpc_response!(WarpGetSignatureResponse, Bytes);

/// Get the signature of a Warp message by querying the Subnet-EVM API
pub fn get_warp_signature(rpc_url: &str, warp_message_id: Id) -> Result<[u8; 96], AshError> {
    let signature = get_json_rpc_req_result::<WarpGetSignatureResponse, Bytes>(
        rpc_url,
        "warp_getSignature",
        Some(ureq::json!([warp_message_id])),
    )?;

    if signature.len() != 96 {
        return Err(AvalancheWarpMessagingError::InvalidSignature(format!(
            "Invalid signature length: {}",
            signature.len(),
        ))
        .into());
    }

    Ok(signature.to_vec().try_into().unwrap())
}
