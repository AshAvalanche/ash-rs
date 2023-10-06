// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

pub(crate) mod error;
pub(crate) mod keyring;
pub(crate) mod parsing;
pub(crate) mod prompt;
pub(crate) mod templating;

use clap::{builder::Str, crate_version};

/// Enrich the version information with whether the command triggers a transaction or not
/// This is used to help use the CLI in a non-interactive way
pub(crate) fn version_tx_cmd(is_tx: bool) -> Str {
    Str::from(format!("{} (tx_cmd={})", crate_version!(), is_tx))
}
