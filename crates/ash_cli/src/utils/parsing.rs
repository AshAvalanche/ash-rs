// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains parsing utility functions

use crate::utils::error::CliError;
use ash_sdk::ids::{node::Id as NodeId, Id};
use chrono::{DateTime, Utc};
use std::str::FromStr;

// Parse an ID from a string
pub(crate) fn parse_id(id: &str) -> Result<Id, CliError> {
    // Try to parse the ID as CB58 first
    if let Ok(id) = Id::from_str(id) {
        return Ok(id);
    }

    // Then try to parse it as hex
    let id_bytes = hex::decode(id.trim_start_matches("0x"))
        .map_err(|e| CliError::dataerr(format!("Error parsing ID: {e}")))?;

    Ok(Id::from_slice(&id_bytes))
}

// Parse a node ID from a string
pub(crate) fn parse_node_id(id: &str) -> Result<NodeId, CliError> {
    let id = NodeId::from_str(id)
        .map_err(|e| CliError::dataerr(format!("Error parsing NodeID: {e}")))?;
    Ok(id)
}

// Parse a DateTime from a string
pub(crate) fn parse_datetime(datetime: &str) -> Result<DateTime<Utc>, CliError> {
    let datetime = DateTime::parse_from_rfc3339(datetime)
        .map_err(|e| CliError::dataerr(format!("Error parsing DateTime: {e}")))?;
    Ok(datetime.with_timezone(&Utc))
}
