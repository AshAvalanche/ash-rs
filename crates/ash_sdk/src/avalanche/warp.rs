// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche Warp Messaging

use crate::{
    avalanche::vms::subnet_evm::warp::{AddressedPayload, SubnetEVMWarpMessage},
    errors::*,
};
use avalanche_types::ids::Id;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const WARP_ANYCAST_ID: &str = "2wkBET2rRgE8pahuaczxKbmv7ciehqsne57F9gtzf1PVcUJEQG";

/// Unsigned Warp message
/// See https://github.com/ava-labs/avalanchego/blob/e70a17d9d988b5067f3ef5c4a057f15ae1271ac4/vms/platformvm/warp/unsigned_message.go#L14
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct WarpUnsignedMessage {
    #[serde(skip)]
    pub bytes: Vec<u8>,
    pub id: Id,
    #[serde(rename = "networkID")]
    pub network_id: u32,
    #[serde(rename = "sourceChainID")]
    pub source_chain_id: Id,
    pub payload: WarpMessagePayload,
}

impl WarpUnsignedMessage {
    /// Try to parse a Subnet-EVM Warp message event log data as an unsigned Warp message
    /// and parse the payload as a Subnet-EVM AddressedPayload
    pub fn try_from_subnet_evm_log_data(bytes: &[u8]) -> Result<Self, AshError> {
        let mut warp_message = Self::from(bytes);
        let warp_payload = match warp_message.payload {
            WarpMessagePayload::Unknown(bytes) => bytes,
            _ => panic!("Warp message payload is not Unknown"),
        };

        warp_message.payload = WarpMessagePayload::SubnetEVMAddressedPayload(
            AddressedPayload::try_from(warp_payload)?,
        );

        Ok(warp_message)
    }
}

impl From<&[u8]> for WarpUnsignedMessage {
    fn from(bytes: &[u8]) -> Self {
        // [0..2] -> ?
        // [2..6] -> networkID
        // [6..38] -> sourceChainID
        // [38..] -> payload
        let network_id = u32::from_be_bytes(bytes[2..6].try_into().unwrap());
        let source_chain_id = Id::from_slice(&bytes[6..38]);
        let payload = WarpMessagePayload::Unknown(bytes[38..].to_vec());

        // Compute the message ID = SHA256(bytes)
        let mut hasher = Sha256::new();
        hasher.update(bytes);

        Self {
            id: Id::from_slice(&hasher.finalize()[..]),
            bytes: bytes.to_vec(),
            network_id,
            source_chain_id,
            payload,
        }
    }
}

/// Warp message payload
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WarpMessagePayload {
    /// Unknown Warp Message payload
    /// This is used to parse Warp messages that are not supported by the current version of the library
    Unknown(Vec<u8>),
    /// Subnet-EVM Warp Message payload
    SubnetEVMAddressedPayload(AddressedPayload),
}

impl Default for WarpMessagePayload {
    fn default() -> Self {
        WarpMessagePayload::Unknown(vec![])
    }
}

/// Warp message status
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub enum WarpMessageStatus {
    #[default]
    Sent,
    Signed(u64),
}

/// Verified Warp message
/// Pre-verified Warp message as it will be parsed on the destination chain
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub enum VerifiedWarpMessage {
    /// Unknown Warp Message
    /// This is used to parse Warp messages that are not supported by the current version of the library
    #[default]
    Unknown,
    /// Subnet-EVM Verified Warp message
    SubnetEVM(SubnetEVMWarpMessage),
}

/// Warp message
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarpMessage {
    pub unsigned_message: WarpUnsignedMessage,
    pub verified_message: VerifiedWarpMessage,
    pub status: WarpMessageStatus,
}
