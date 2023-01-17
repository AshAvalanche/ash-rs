// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

// Module that contains the Ash CLI root parser

mod node;

use clap::{Parser, Subcommand};
use node::{parse as node_parse, NodeCommand};

#[derive(Parser)]
#[command(author, version)]
#[command(about = "Ash CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: CliCommands,
    #[arg(long, help = "Output in JSON format", global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum CliCommands {
    Node(NodeCommand),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        CliCommands::Node(node) => node_parse(node, cli.json),
    }
}
