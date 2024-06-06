// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains the helper subcommand parser

use crate::{
    console::{
        create_api_config_with_access_token, load_console, project::get_current_project_id_or_name,
    },
    utils::{error::CliError, templating::*, version_tx_cmd},
};
use ash_sdk::{
    avalanche::vms::AvalancheVmType,
    console::{self, api_models::project},
    ids::Id,
};
use async_std::task;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

/// Avalanche blockchain
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AvalancheBlockchain {
    #[serde(default)]
    pub id: Id,
    pub name: String,
    #[serde(default)]
    pub vm_id: Id,
    pub vm_type: AvalancheVmType,
}

/// Ash Console helper
#[derive(Parser)]
#[command()]
pub(crate) struct HelperCommand {
    #[command(subcommand)]
    command: HelperSubcommands,
    /// Console project ID or name
    /// Defaults to the current project
    #[arg(
        long,
        short = 'p',
        default_value = "current",
        global = true,
        env = "ASH_CONSOLE_PROJECT"
    )]
    project_id_or_name: String,
}

#[derive(Subcommand)]
enum HelperSubcommands {
    /// Show helpful information to stake on an Avalanche node
    #[command(version = version_tx_cmd(false))]
    Stake {
        /// Node resource ID or name
        node_resource_id_or_name: String,
    },
    /// Show helpful information about the RPC endpoint(s) of a Subnet blockchain(s)
    #[command(version = version_tx_cmd(false))]
    Rpc {
        /// Endpoint node resource ID or name
        node_resource_id_or_name: String,
        /// Subnet resource ID or name
        subnet_resource_id_or_name: String,
    },
    /// Show helpful information about the URL of a Blockscout
    #[command(version = version_tx_cmd(false))]
    Blockscout {
        /// Blockscout resource ID or name
        blockscout_id_or_name: String,
    },
}

// Show helpful information to stake on an Avalanche node
fn staking_helper(
    project_id_or_name: &str,
    node_resource_id_or_name: &str,
    config: Option<&str>,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let spinner = spinner_with_message("Fetching node information...".to_string());

    let project_response = task::block_on(async {
        console::api::get_project_by_id_or_name(&api_config, project_id_or_name)
            .await
            .map_err(|e| CliError::dataerr(format!("Error getting project: {e}")))
    })?;

    if project_response.network.unwrap() == console::api_models::project::Network::Local {
        return Err(CliError::dataerr(
            "Staking is not supported on local (devnet) projects".to_string(),
        ));
    }

    let node_response = task::block_on(async {
        console::api::get_project_resource_by_id_or_name(
            &api_config,
            project_id_or_name,
            node_resource_id_or_name,
        )
        .await
        .map_err(|e| CliError::dataerr(format!("Error getting node resource: {e}")))
    })?;

    let node_id_secret_id = match *node_response.resource_type.unwrap() {
        console::api_models::ResourceType::AvalancheNode => {
            node_response.node_id_secret_id.unwrap()
        }
        _ => {
            return Err(CliError::dataerr(
                "Resource is not an `avalancheNode`!".to_string(),
            ))
        }
    };

    let node_id_secret_response = task::block_on(async {
        console::api::get_secret_by_id_or_name(&api_config, &node_id_secret_id)
            .await
            .map_err(|e| CliError::dataerr(format!("Error getting node ID secret: {e}")))
    })?;

    let node_id = node_id_secret_response.node_id.unwrap();

    spinner.finish_and_clear();

    println!(
        "To stake on your node '{}':\n1. Navigate to the Core 'Stake/Validate' wizard at https://{}core.app/stake/validate/\n2. Select the staking amount\n3. Provide the following Node ID: {}\n4. Complete the 'Stake/Validate' form and sign the transaction!",
        type_colorize(&node_resource_id_or_name),
        if project_response.network.unwrap() == project::Network::Testnet {
            "test.".to_string()
        } else { "".to_string() },
        type_colorize(&node_id)
    );

    Ok(())
}

// Show helpful information about the RPC endpoint(s) of a Subnet blockchain(s)
fn rpc_helper(
    project_id_or_name: &str,
    node_resource_id_or_name: &str,
    subnet_resource_id_or_name: &str,
    config: Option<&str>,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let mut spinner = spinner_with_message("Fetching node information...".to_string());

    let node_response = task::block_on(async {
        console::api::get_project_resource_by_id_or_name(
            &api_config,
            project_id_or_name,
            node_resource_id_or_name,
        )
        .await
        .map_err(|e| CliError::dataerr(format!("Error getting node resource: {e}")))
    })?;

    if *node_response.resource_type.unwrap() != console::api_models::ResourceType::AvalancheNode {
        return Err(CliError::dataerr(
            "Resource is not an `avalancheNode`!".to_string(),
        ));
    };

    spinner.finish_and_clear();
    spinner = spinner_with_message("Fetching Subnet information...".to_string());

    let subnet_response = task::block_on(async {
        console::api::get_project_resource_by_id_or_name(
            &api_config,
            project_id_or_name,
            subnet_resource_id_or_name,
        )
        .await
        .map_err(|e| CliError::dataerr(format!("Error getting Subnet resource: {e}")))
    })?;

    let subnet_chains = match *subnet_response.resource_type.unwrap() {
        console::api_models::ResourceType::AvalancheSubnet => subnet_response
            .subnet_status
            .unwrap()
            .blockchains
            .unwrap_or_default(),
        _ => {
            return Err(CliError::dataerr(
                "Resource is not an `avalancheSubnet`!".to_string(),
            ))
        }
    };

    spinner.finish_and_clear();

    for blockchain_value in subnet_chains.iter() {
        let blockchain: AvalancheBlockchain = serde_json::from_value(blockchain_value.clone())
            .map_err(|e| CliError::dataerr(format!("Error parsing blockchain info: {e}")))?;
        println!(
            "{} RPC endpoint:\n  {}",
            type_colorize(&blockchain.name),
            type_colorize(&format!(
                "http://{}:9650/ext/bc/{}/rpc",
                node_response.node_ip.clone().unwrap_or_default(),
                blockchain.id
            ))
        );
    }

    Ok(())
}

/// Show helpful information about the URL of a Blockscout
fn blockscout_helper(
    project_id_or_name: &str,
    blockscout_id_or_name: &str,
    config: Option<&str>,
) -> Result<(), CliError> {
    let mut console = load_console(config)?;

    let api_config = create_api_config_with_access_token(&mut console)?;

    let spinner = spinner_with_message("Fetching blockscout information...".to_string());

    let blockscout_response = task::block_on(async {
        console::api::get_project_resource_by_id_or_name(
            &api_config,
            project_id_or_name,
            blockscout_id_or_name,
        )
        .await
        .map_err(|e| CliError::dataerr(format!("Error getting blockscout resource: {e}")))
    })?;

    if *blockscout_response.resource_type.unwrap() != console::api_models::ResourceType::Blockscout {
        return Err(CliError::dataerr(
            "Resource is not a `blockscout`!".to_string(),
        ));
    };

    spinner.finish_and_clear();

    println!(
        "Blockscout URL:\n  {}",
        type_colorize(&format!(
            "http://{}:80",
            blockscout_response.blockscout_ip.clone().unwrap_or_default()
        ))
    );

    Ok(())
}

// Parse helper subcommand
pub(crate) fn parse(operation: HelperCommand, config: Option<&str>) -> Result<(), CliError> {
    let mut project_id_or_name = operation.project_id_or_name;

    // Get the current project ID
    if project_id_or_name == "current" {
        project_id_or_name = get_current_project_id_or_name()?;
    }

    match operation.command {
        HelperSubcommands::Stake {
            node_resource_id_or_name,
        } => staking_helper(&project_id_or_name, &node_resource_id_or_name, config),
        HelperSubcommands::Rpc {
            node_resource_id_or_name,
            subnet_resource_id_or_name,
        } => rpc_helper(
            &project_id_or_name,
            &node_resource_id_or_name,
            &subnet_resource_id_or_name,
            config,
        ),
        HelperSubcommands::Blockscout {
            blockscout_id_or_name,
        } => blockscout_helper(&project_id_or_name, &blockscout_id_or_name, config),
    }
}
