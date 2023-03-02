// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the Ash CLI root parser

mod avalanche;
mod conf;
mod node;
mod utils;

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::process::exit;

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Ash CLI")]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,
    #[arg(long, help = "Output in JSON format", global = true)]
    json: bool,
    #[arg(long, help = "Path to the configuration file", global = true)]
    config: Option<String>,
}

#[derive(Subcommand)]
enum CliCommands {
    #[command(visible_alias = "avax")]
    Avalanche(avalanche::AvalancheCommand),
    Conf(conf::ConfCommand),
    Node(node::NodeCommand),
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        CliCommands::Avalanche(avalanche) => {
            avalanche::parse(avalanche, cli.config.as_deref(), cli.json)
        }
        CliCommands::Conf(conf) => conf::parse(conf),
        CliCommands::Node(node) => node::parse(node, cli.json),
    }
    .unwrap_or_else(|e| {
        eprintln!("{}", e.message.red());
        exit(e.exit_code)
    });
}
