// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Avalanche Warp Messaging

use crate::{
    avalanche::vms::subnet_evm::warp::{AddressedPayload, SubnetEVMWarpMessage},
    errors::*,
};
use avalanche_types::ids::{node::Id as NodeId, Id};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const WARP_ANYCAST_ID: &str = "2wkBET2rRgE8pahuaczxKbmv7ciehqsne57F9gtzf1PVcUJEQG";

/// Unsigned Warp message
/// See https://github.com/ava-labs/avalanchego/blob/e70a17d9d988b5067f3ef5c4a057f15ae1271ac4/vms/platformvm/warp/unsigned_message.go#L14
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WarpUnsignedMessage {
    pub id: Id,
    #[serde(rename = "networkID")]
    pub network_id: u32,
    #[serde(rename = "sourceChainID")]
    pub source_chain_id: Id,
    pub payload: WarpMessagePayload,
    #[serde(skip)]
    pub bytes: Vec<u8>,
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum WarpMessageStatus {
    #[default]
    Sent,
    Signed(u16),
}

/// Verified Warp message
/// Pre-verified Warp message as it will be parsed on the destination chain
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum VerifiedWarpMessage {
    /// Unknown Warp Message
    /// This is used to parse Warp messages that are not supported by the current version of the library
    #[default]
    Unknown,
    /// Subnet-EVM Verified Warp message
    SubnetEVM(SubnetEVMWarpMessage),
}

/// Warp message
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WarpMessage {
    pub unsigned_message: WarpUnsignedMessage,
    pub verified_message: VerifiedWarpMessage,
    pub status: WarpMessageStatus,
    pub node_signatures: Vec<WarpMessageNodeSignature>,
}

impl WarpMessage {
    /// Add a node signature to the Warp message
    pub fn add_node_signature(&mut self, node_signature: WarpMessageNodeSignature) {
        // Only add the signature if it is not already present
        if !self
            .node_signatures
            .iter()
            .any(|sig| sig.node_id == node_signature.node_id)
        {
            self.node_signatures.push(node_signature);
        }

        // Update the status of the Warp message
        if self.node_signatures.len() >= 1 {
            self.status = WarpMessageStatus::Signed(self.node_signatures.len() as u16);
        } else {
            self.status = WarpMessageStatus::Sent;
        }
    }
}

/// Warp message signature from a validator node
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WarpMessageNodeSignature {
    pub node_id: NodeId,
    #[serde(
        serialize_with = "ethers::types::serialize_bytes",
        deserialize_with = "hex::deserialize"
    )]
    pub signature: [u8; 96],
}

impl Default for WarpMessageNodeSignature {
    fn default() -> Self {
        Self {
            node_id: NodeId::default(),
            signature: [0; 96],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ethers::types::{Address, Bytes, H256};
    use std::str::FromStr;

    const WARP_MESSAGE_HEX: &str = "00000000303976dccb39c21a43aad4ffa98d4dd86a9ca29f5038a13b87658ef856bc161dbb470000005e0000000000008db97c7cece249c2b98bdc0226cc4c2a57bf52fcffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8db97c7cece249c2b98bdc0226cc4c2a57bf52fc0000000c48656c6c6f20776f726c6421";

    #[test]
    fn test_warp_message_from_bytes() {
        let warp_message =
            WarpUnsignedMessage::from(hex::decode(WARP_MESSAGE_HEX).unwrap().as_slice());

        assert_eq!(warp_message.network_id, 12345);
        assert_eq!(
            warp_message.source_chain_id,
            Id::from_str("uMBaf3Nb62N2xajmxo9ZL5VcSw87tuB3snQEJb8nsyxyLq68f").unwrap()
        );
    }

    #[test]
    fn test_warp_message_try_from_subnet_evm_log_data() {
        let warp_message = WarpUnsignedMessage::try_from_subnet_evm_log_data(
            hex::decode(WARP_MESSAGE_HEX).unwrap().as_slice(),
        )
        .unwrap();

        assert_eq!(warp_message.network_id, 12345);
        assert_eq!(
            warp_message.source_chain_id,
            Id::from_str("uMBaf3Nb62N2xajmxo9ZL5VcSw87tuB3snQEJb8nsyxyLq68f").unwrap()
        );
        assert_eq!(
            warp_message.payload,
            WarpMessagePayload::SubnetEVMAddressedPayload(AddressedPayload {
                source_address: Address::from_str("0x8db97c7cece249c2b98bdc0226cc4c2a57bf52fc")
                    .unwrap(),
                destination_chain_id: H256::from_str(
                    "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap(),
                destination_address: Address::from_str(
                    "0x8db97c7cece249c2b98bdc0226cc4c2a57bf52fc"
                )
                .unwrap(),
                payload: Bytes::from_str("0x0000000c48656c6c6f20776f726c6421").unwrap(),
            })
        )
    }
}
