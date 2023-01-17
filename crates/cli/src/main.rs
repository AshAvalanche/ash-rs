// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

// Module that contains the Ash CLI root parser

mod node;
mod subnet;

use clap::{Parser, Subcommand};

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
    Node(node::NodeCommand),
    Subnet(subnet::SubnetCommand),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        CliCommands::Node(node) => node::parse(node, cli.json),
        CliCommands::Subnet(subnet) => subnet::parse(subnet, cli.json),
    }
}
