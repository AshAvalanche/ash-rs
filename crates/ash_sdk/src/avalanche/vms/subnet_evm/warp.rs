// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Subnet-EVM Warp messages

use crate::{
    avalanche::warp::{WarpMessagePayload, WarpUnsignedMessage},
    errors::*,
};
use ethers::types::{Address, Bytes, Log, H256};
use serde::{Deserialize, Serialize};

/// Subnet-EVM Warp message
/// See https://github.com/ava-labs/subnet-evm/blob/309daad20ba17346ae3712c96c2db594e011b29c/x/warp/contract.go#L57
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SubnetEVMWarpMessage {
    #[serde(rename = "originChainID")]
    pub origin_chain_id: H256,
    pub origin_sender_address: Address,
    #[serde(rename = "destinationChainID")]
    pub destination_chain_id: H256,
    pub destination_address: Address,
    pub payload: Option<Bytes>,
}

impl From<Log> for SubnetEVMWarpMessage {
    fn from(log: Log) -> Self {
        // The log data is the WarpUnsignedMessage with the AddressedPayload as the payload
        let warp_unsigned_message =
            WarpUnsignedMessage::try_from_subnet_evm_log_data(&log.data.to_vec()[..])
                .or_else::<Result<WarpUnsignedMessage, AshError>, _>(|_| {
                    Ok(WarpUnsignedMessage::from(&log.data.to_vec()[..]))
                })
                .unwrap();

        Self {
            origin_chain_id: H256::from_slice(&warp_unsigned_message.source_chain_id.to_vec()),
            origin_sender_address: Address::from_slice(&log.topics[3].as_fixed_bytes()[12..]),
            destination_chain_id: H256::from_slice(log.topics[1].as_fixed_bytes()),
            destination_address: Address::from_slice(&log.topics[2].as_fixed_bytes()[12..]),
            payload: match warp_unsigned_message.payload {
                WarpMessagePayload::SubnetEVMAddressedPayload(addressed_payload) => {
                    Some(addressed_payload.payload)
                }
                _ => None,
            },
        }
    }
}

impl SubnetEVMWarpMessage {
    // Create a new Subnet-EVM Warp message from a SendWarpMessageFilter event
    // The auto-generated event does not work, so we use the log instead
    // pub fn from_send_warp_message_event(chain_id: Id, log: SendWarpMessageFilter) -> Self {
    //     Self {
    //         origin_chain_id: chain_id,
    //         origin_sender_address: log.sender,
    //         destination_chain_id: Id::from_slice(&log.destination_chain_id),
    //         destination_address: log.destination_address,
    //         payload: log.message.to_vec(),
    //     }
    // }
}

/// AddressedPayload defines the format for delivering a point to point message across VMs
/// See https://github.com/ava-labs/subnet-evm/blob/309daad20ba17346ae3712c96c2db594e011b29c/warp/payload/payload.go#L14C31-L14C31
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AddressedPayload {
    pub source_address: Address,
    #[serde(rename = "destinationChainID")]
    pub destination_chain_id: H256,
    pub destination_address: Address,
    pub payload: Bytes,
}

impl TryFrom<Vec<u8>> for AddressedPayload {
    type Error = AshError;

    fn try_from(payload: Vec<u8>) -> Result<Self, AshError> {
        // [0..4] -> payload length (starts at 4)
        // [4..10] -> ?
        // [10..30] -> sourceAddress
        // [30..62] -> destinationChainID
        // [62..82] -> destinationAddress
        // [82..end] -> payload (abi encoded)

        // Check that the payload is at least 88 bytes
        if payload.len() < 88 {
            return Err(AshError::AvalancheWarpMessagingError(
                AvalancheWarpMessagingError::ParseFailure {
                    property: "payload".to_string(),
                    msg: "AddressedPayload is too short".to_string(),
                },
            ));
        }

        // Check that the payload length is correct
        let payload_length = u32::from_be_bytes(payload[0..4].try_into().unwrap());
        if (payload_length + 4) != payload.len() as u32 {
            return Err(AshError::AvalancheWarpMessagingError(
                AvalancheWarpMessagingError::ParseFailure {
                    property: "payload".to_string(),
                    msg: "AddressedPayload length is incorrect".to_string(),
                },
            ));
        }

        Ok(Self {
            source_address: Address::from_slice(&payload[10..30]),
            destination_chain_id: H256::from_slice(&payload[30..62]),
            destination_address: Address::from_slice(&payload[62..82]),
            payload: Bytes::from(payload[82..].to_vec()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    const ADDRESSED_PAYLOAD_HEX: &str = "0000005e0000000000008db97c7cece249c2b98bdc0226cc4c2a57bf52fcffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8db97c7cece249c2b98bdc0226cc4c2a57bf52fc0000000c48656c6c6f20776f726c6421";

    fn warp_message_log() -> Log {
        Log {
            address: Address::from_str("0x0200000000000000000000000000000000000005").unwrap(),
            topics: vec![
                H256::from_str("0x3e6ad4991eb8370644656486297eb0bf6a7792ef369dfd9eda2c51ec82b67b59").unwrap(),
                H256::from_str("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap(),
                H256::from_str("0x0000000000000000000000008db97c7cece249c2b98bdc0226cc4c2a57bf52fc").unwrap(),
                H256::from_str("0x0000000000000000000000008db97c7cece249c2b98bdc0226cc4c2a57bf52fc").unwrap(),
            ],
            data: Bytes::from_str("0x00000000303976dccb39c21a43aad4ffa98d4dd86a9ca29f5038a13b87658ef856bc161dbb470000005e0000000000008db97c7cece249c2b98bdc0226cc4c2a57bf52fcffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8db97c7cece249c2b98bdc0226cc4c2a57bf52fc0000000c48656c6c6f20776f726c6421").unwrap(),
            ..Default::default()
        }
    }

    #[test]
    fn test_subnet_evm_warp_message_from_log() {
        let warp_message = SubnetEVMWarpMessage::from(warp_message_log());

        assert_eq!(
            warp_message,
            SubnetEVMWarpMessage {
                origin_chain_id: H256::from_str(
                    "0x76dccb39c21a43aad4ffa98d4dd86a9ca29f5038a13b87658ef856bc161dbb47"
                )
                .unwrap(),
                origin_sender_address: Address::from_str(
                    "0x8db97c7cece249c2b98bdc0226cc4c2a57bf52fc"
                )
                .unwrap(),
                destination_chain_id: H256::from_str(
                    "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                )
                .unwrap(),
                destination_address: Address::from_str(
                    "0x8db97c7cece249c2b98bdc0226cc4c2a57bf52fc"
                )
                .unwrap(),
                payload: Some(Bytes::from_str("0x0000000c48656c6c6f20776f726c6421").unwrap()),
            }
        );
    }

    #[test]
    fn test_addressed_payload_from_bytes() {
        let addressed_payload =
            AddressedPayload::try_from(hex::decode(ADDRESSED_PAYLOAD_HEX).unwrap()).unwrap();

        assert_eq!(
            addressed_payload,
            AddressedPayload {
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
            }
        );
    }
}
