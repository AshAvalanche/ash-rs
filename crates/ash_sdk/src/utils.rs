// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains utility functions

use crate::errors::*;
use ethers::types::BlockNumber;
use std::str::FromStr;

pub fn parse_evm_block_number(block_number: &str) -> Result<BlockNumber, AshError> {
    BlockNumber::from_str(block_number).map_err(|e| {
        AvalancheBlockchainError::BlockNumberParseFailure {
            block_number: block_number.to_string(),
            msg: e.to_string(),
        }
        .into()
    })
}
