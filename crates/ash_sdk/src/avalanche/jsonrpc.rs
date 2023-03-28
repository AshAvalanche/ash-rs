// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

pub mod info;
pub mod platformvm;

// Module that contains code to interact with the Avalanche JSON RPC endpoints

use crate::errors::*;
use avalanche_types::jsonrpc::ResponseError;
use ureq;

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

    if resp.get_error().is_some() {
        Err(RpcError::ResponseError {
            code: resp.get_error().unwrap().code,
            message: resp.get_error().unwrap().message,
            data: resp.get_error().unwrap().data,
        })
    } else {
        Ok(resp.get_result().unwrap())
    }
}
