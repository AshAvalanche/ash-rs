// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2023, E36 Knots

// Module that contains the conf subcommand parser

use ash::conf::AshConfig;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(about = "Interact with Avalanche networks", long_about = None)]
pub struct ConfCommand {
    #[command(subcommand)]
    command: ConfCommands,
}

#[derive(Subcommand)]
enum ConfCommands {
    #[command(about = "Initialize an Ash config file", long_about = None)]
    Init {
        #[arg(from_global)]
        config: String,
        #[arg(long, help = "Overwrite existing config file")]
        force: bool,
    },
}

// Initialize an Ash config file
fn init(config: String, force: bool) {
    match AshConfig::dump_default(&config, force) {
        Ok(_) => println!("Config file initialized at '{}'", config),
        Err(err) => eprintln!("Error initializing config file: {}", err),
    }
}

// Parse conf subcommand
pub fn parse(conf: ConfCommand) {
    match conf.command {
        ConfCommands::Init { config, force } => init(config, force),
    }
}
