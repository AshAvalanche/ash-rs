// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains code to interact with Subnet-EVM precompiles

include!(concat!(env!("OUT_DIR"), "/warp_messenger.rs"));

use crate::{avalanche::blockchains::AvalancheBlockchain, errors::*};
use avalanche_types::ids::Id;
use ethers::{
    core::types::{Address, BlockNumber, Log, H256},
    providers::{Http, Middleware, Provider},
};

/// WarpMessenger precompile address
pub const WARP_MESSENGER_ADDRESS: &str = "0x0200000000000000000000000000000000000005";

/// WarpMessenger precompile HTTP provider
#[derive(Debug, Clone)]
pub struct WarpMessengerHttp {
    pub chain_id: Id,
    pub contract: WarpMessenger<Provider<Http>>,
}

impl WarpMessengerHttp {
    /// Create a new WarpMessengerHttp instance
    pub fn new(chain: &AvalancheBlockchain) -> Result<WarpMessengerHttp, AshError> {
        let client = chain.get_ethers_provider()?;
        let warp_messenger = WarpMessenger::new(
            WARP_MESSENGER_ADDRESS.parse::<Address>().unwrap(),
            client.into(),
        );

        Ok(WarpMessengerHttp {
            chain_id: chain.id,
            contract: warp_messenger,
        })
    }

    /// Get the blockchain ID as seen by the WarpMessenger precompile
    pub async fn get_blockchain_id(&self) -> Result<[u8; 32], AshError> {
        let blockchain_id = self
            .contract
            .get_blockchain_id()
            .call()
            .await
            .map_err(|e| RpcError::EthCallFailure {
                contract_addr: self.contract.address().to_string(),
                function_name: "getBlockchainID".to_string(),
                msg: e.to_string(),
            })?;

        Ok(blockchain_id)
    }

    /// Get SendWarpMessage event logs emitted between 2 blocks
    /// Filter by destination_chain_id, destination_address and sender
    /// Return the list of event logs ordered
    pub async fn get_send_warp_message_logs(
        &self,
        from_block: BlockNumber,
        to_block: BlockNumber,
        destination_chain_id: Option<[u8; 32]>,
        destination_address: Option<Address>,
        sender: Option<Address>,
    ) -> Result<Vec<Log>, AshError> {
        // Do not use the auto generated event filter because it does not work for some reason
        // let mut event_filter = self
        //     .contract
        //     .event::<SendWarpMessageFilter>()
        //     .from_block(from_block)
        //     .to_block(to_block);

        let mut event_filter = ethers::types::Filter::new()
            .address(self.contract.address())
            .from_block(from_block)
            .to_block(to_block);

        event_filter = match destination_chain_id {
            Some(chain_id) => event_filter.topic1(H256::from(chain_id)),
            None => event_filter,
        };
        event_filter = match destination_address {
            Some(address) => event_filter.topic2(address),
            None => event_filter,
        };
        event_filter = match sender {
            Some(address) => event_filter.topic3(address),
            None => event_filter,
        };

        let events = self
            .contract
            .client()
            .provider()
            .get_logs(&event_filter)
            .await
            .map_err(|e| RpcError::EthLogsFailure {
                contract_addr: self.contract.address().to_string(),
                msg: e.to_string(),
            })?;

        Ok(events)
    }
}
