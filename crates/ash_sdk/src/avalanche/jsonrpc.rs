// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

pub mod avm;
pub mod info;
pub mod platformvm;

// Module that contains code to interact with the Avalanche JSON RPC endpoints

use crate::errors::*;
use avalanche_types::jsonrpc::ResponseError;

/// Trait that defines the methods to get the result and error of a JSON RPC response
/// This is used to avoid code duplication when posting JSON RPC requests
pub trait JsonRpcResponse<Resp, Res>
where
    Resp: serde::de::DeserializeOwned,
    Res: serde::de::DeserializeOwned,
{
    fn get_error(&self) -> Option<ResponseError>;
    fn get_result(&self) -> Option<Res>;
}

/// Macro that implements the JsonRpcResponse trait for a response type and its result type
#[macro_export]
macro_rules! impl_json_rpc_response {
    ($resp_type:ty, $res_type:ty) => {
        impl JsonRpcResponse<$resp_type, $res_type> for $resp_type {
            fn get_error(&self) -> Option<ResponseError> {
                self.error.clone()
            }

            fn get_result(&self) -> Option<$res_type> {
                self.result.clone()
            }
        }
    };
}

/// Get the result of a response from a JSON RPC request
/// If the response contains an error, return an error instead
pub fn get_json_rpc_req_result<Resp, Res>(
    rpc_url: &str,
    method: &str,
    params: Option<ureq::serde_json::Value>,
) -> Result<Res, RpcError>
where
    Resp: serde::de::DeserializeOwned,
    Res: serde::de::DeserializeOwned,
    Resp: JsonRpcResponse<Resp, Res>,
{
    let resp: Resp = ureq::post(rpc_url)
        .send_json(ureq::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        }))
        .map_err(|e| RpcError::Unknown(e.to_string()))?
        .into_json()
        .map_err(|e| RpcError::Unknown(e.to_string()))?;

    if let Some(error) = resp.get_error() {
        Err(RpcError::ResponseError {
            code: error.code,
            message: error.message,
            data: error.data,
        })
    } else {
        Ok(resp.get_result().unwrap())
    }
}
