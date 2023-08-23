// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

mod avalanche;
mod conf;
mod utils;

#[macro_use]
extern crate enum_display_derive;

// Module that contains the Ash CLI root parser

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::process::exit;

#[derive(Parser)]
/// Ash CLI. More information at https://ash.center/docs/toolkit/ash-cli/introduction
#[command(author, version)]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,
    /// Output in JSON format
    #[arg(long, short = 'j', global = true, env = "ASH_JSON")]
    json: bool,
    /// Path to the configuration file
    #[arg(long, short = 'c', global = true, env = "ASH_CONFIG")]
    config: Option<String>,
}

#[derive(Subcommand)]
enum CliCommands {
    Avalanche(avalanche::AvalancheCommand),
    Conf(conf::ConfCommand),
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        CliCommands::Avalanche(avalanche) => {
            avalanche::parse(avalanche, cli.config.as_deref(), cli.json)
        }
        CliCommands::Conf(conf) => conf::parse(conf),
    }
    .unwrap_or_else(|e| {
        eprintln!("{}", e.message.red());
        exit(e.exit_code)
    });
}
