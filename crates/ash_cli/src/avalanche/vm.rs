// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the vm subcommand parser

use crate::utils::{error::CliError, templating::*};
use ash_sdk::avalanche::vms::{encode_genesis_data, generate_vm_id, AvalancheVmType};
use clap::{Parser, Subcommand};

/// Interact with Avalanche VMs
#[derive(Parser)]
#[command()]
pub(crate) struct VmCommand {
    #[command(subcommand)]
    command: VmSubcommands,
}

#[derive(Subcommand)]
enum VmSubcommands {
    /// Encode a VM genesis (in JSON) to bytes
    #[command()]
    EncodeGenesis {
        /// Path to the genesis JSON file
        genesis_file: String,
        /// VM type
        #[arg(long, short = 't', default_value = "SubnetEVM")]
        vm_type: AvalancheVmType,
    },
    /// Generate the VM ID from the VM name
    #[command()]
    GenerateId {
        /// VM name
        vm_name: String,
    },
}

fn encode_genesis(
    genesis_file: &str,
    vm_type: AvalancheVmType,
    json: bool,
) -> Result<(), CliError> {
    let genesis_json = std::fs::read_to_string(genesis_file).map_err(|e| {
        CliError::dataerr(format!("Error reading genesis file {genesis_file}: {e}"))
    })?;

    let genesis_bytes = encode_genesis_data(vm_type, &genesis_json).map_err(|e| {
        CliError::dataerr(format!("Error encoding genesis file {genesis_file}: {e}"))
    })?;

    if json {
        println!(
            "{}",
            serde_json::json!({ "genesisBytes": format!("0x{}", hex::encode(genesis_bytes)) })
        );
        return Ok(());
    }

    println!("{}", template_genesis_encoded(genesis_bytes, 0));

    Ok(())
}

fn generate_id(vm_name: &str, json: bool) -> Result<(), CliError> {
    let vm_id = generate_vm_id(vm_name);

    if json {
        println!("{}", serde_json::json!({ "vmID": vm_id.to_string() }));
        return Ok(());
    }

    println!("VM ID: {}", type_colorize(&vm_id.to_string()));

    Ok(())
}

// Parse vm subcommand
pub(crate) fn parse(x: VmCommand, json: bool) -> Result<(), CliError> {
    match x.command {
        VmSubcommands::EncodeGenesis {
            genesis_file,
            vm_type,
        } => encode_genesis(&genesis_file, vm_type, json),
        VmSubcommands::GenerateId { vm_name } => generate_id(&vm_name, json),
    }
}
