// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains parsing utility functions

use crate::utils::error::CliError;
use ash_sdk::ids::{node::Id as NodeId, Id};
use chrono::{DateTime, Utc};
use std::str::FromStr;

// Parse an ID from a string
pub(crate) fn parse_id(id: &str) -> Result<Id, CliError> {
    let id = Id::from_str(id).map_err(|e| CliError::dataerr(format!("Error parsing ID: {e}")))?;
    Ok(id)
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
