// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

use ash_sdk::avalanche::{
    blockchains::AvalancheBlockchain,
    nodes::AvalancheNode,
    subnets::{AvalancheSubnet, AvalancheSubnetType, AvalancheSubnetValidator},
};
use colored::{ColoredString, Colorize};
use indoc::formatdoc;

// Module that contains templating functions for info strings

// Get the type of a variable
fn type_of<T>(_: T) -> &'static str {
    std::any::type_name::<T>()
}

// Generate a colored string from the given variable
// The color is determined by the variable's type
pub(crate) fn type_colorize<T>(var: &T) -> ColoredString
where
    T: std::fmt::Display,
{
    match type_of(var).split(':').last().unwrap() {
        "String" | "&&str" => var.to_string().yellow(),
        "&u64" | "&u32" | "&u16" | "&u8" | "&usize" => var.to_string().cyan(),
        "&i64" | "&i32" | "&i16" | "&i8" | "&isize" => var.to_string().cyan(),
        "&f64" | "&f32" | "IpAddr" => var.to_string().magenta(),
        "&bool" => var.to_string().blue(),
        "Id" => var.to_string().green(),
        _ => var.to_string().bright_white(),
    }
}

pub(crate) fn template_horizontal_rule(character: char, length: usize) -> String {
    format!("{character}").repeat(length)
}

pub(crate) fn template_blockchain_info(
    blockchain: &AvalancheBlockchain,
    list: bool,
    indent: u8,
) -> String {
    let mut info = String::new();

    if list {
        info.push_str(&formatdoc!(
            "
            - {}:
               ID:      {}
               VM type: {}
               RPC URL: {}",
            type_colorize(&blockchain.name),
            type_colorize(&blockchain.id),
            type_colorize(&blockchain.vm_type),
            type_colorize(&blockchain.rpc_url),
        ));
    } else {
        info.push_str(&formatdoc!(
            "
            Blockchain '{}':
              ID:      {}
              VM type: {}
              RPC URL: {}",
            type_colorize(&blockchain.name),
            type_colorize(&blockchain.id),
            type_colorize(&blockchain.vm_type),
            type_colorize(&blockchain.rpc_url),
        ));
    }

    indent::indent_all_by(indent.into(), info)
}

pub(crate) fn template_validator_info(
    validator: &AvalancheSubnetValidator,
    subnet: &AvalancheSubnet,
    list: bool,
    indent: u8,
    extended: bool,
) -> String {
    let mut info = String::new();

    let common_info = &formatdoc!(
        "
        Tx ID:            {}
        Start time:       {}
        End time:         {}
        ",
        type_colorize(&validator.tx_id),
        type_colorize(&validator.start_time),
        type_colorize(&validator.end_time),
    );

    let permissioned_subnet_info = &formatdoc!(
        "
        Weight:           {}",
        type_colorize(&validator.weight.unwrap_or_default()),
    );

    let elastic_subnet_info = &formatdoc!(
        "
        Connected:        {}
        Uptime:           {}
        Stake amount:     {}
        Potential reward: {}
        Delegation fee:   {}
        Validation reward owner:
          Locktime: {}
          Threshold: {}
          Addresses: {}
        Delegator count:  {}
        Delegator weight: {}
        Delegation reward owner:
          Locktime: {}
          Threshold: {}
          Addresses: {}",
        type_colorize(&validator.connected),
        type_colorize(&validator.uptime.unwrap_or_default()),
        type_colorize(&validator.stake_amount.unwrap_or_default()),
        type_colorize(&validator.potential_reward.unwrap_or_default()),
        type_colorize(&validator.delegation_fee.unwrap_or_default()),
        type_colorize(
            &validator
                .validation_reward_owner
                .clone()
                .unwrap_or_default()
                .locktime
        ),
        type_colorize(
            &validator
                .validation_reward_owner
                .clone()
                .unwrap_or_default()
                .threshold
        ),
        type_colorize(&format!(
            "{:?}",
            validator
                .validation_reward_owner
                .clone()
                .unwrap_or_default()
                .addresses
        )),
        type_colorize(&validator.delegator_count.unwrap_or_default()),
        type_colorize(&validator.delegator_weight.unwrap_or_default()),
        type_colorize(
            &validator
                .delegation_reward_owner
                .clone()
                .unwrap_or_default()
                .locktime
        ),
        type_colorize(
            &validator
                .delegation_reward_owner
                .clone()
                .unwrap_or_default()
                .threshold
        ),
        type_colorize(&format!(
            "{:?}",
            validator
                .delegation_reward_owner
                .clone()
                .unwrap_or_default()
                .addresses
        )),
    );

    if list {
        // If extended is true, we want to show all the information
        if extended {
            info.push_str(&formatdoc!(
                "
                - {}:
                ",
                type_colorize(&validator.node_id),
            ));

            info.push_str(&indent::indent_all_by(4, common_info));

            // Display extra information if the validator is a primary validator
            match subnet.subnet_type {
                AvalancheSubnetType::Permissioned => {
                    info.push_str(&indent::indent_all_by(4, permissioned_subnet_info));
                }
                AvalancheSubnetType::Elastic | AvalancheSubnetType::PrimaryNetwork => {
                    info.push_str(&indent::indent_all_by(4, elastic_subnet_info));
                }
            }
        } else {
            info.push_str(&formatdoc!(
                "
            - {}",
                type_colorize(&validator.node_id),
            ));
        }
    } else {
        info.push_str(&formatdoc!(
            "
            Validator '{}' on Subnet '{}':
            ",
            type_colorize(&validator.node_id),
            type_colorize(&validator.subnet_id),
        ));

        info.push_str(&indent::indent_all_by(2, common_info));

        // Display extra information if the validator is a primary validator
        match subnet.subnet_type {
            AvalancheSubnetType::Permissioned => {
                info.push_str(&indent::indent_all_by(2, permissioned_subnet_info));
            }
            AvalancheSubnetType::Elastic | AvalancheSubnetType::PrimaryNetwork => {
                info.push_str(&indent::indent_all_by(2, elastic_subnet_info));
            }
        }
    }

    indent::indent_all_by(indent.into(), info)
}

pub(crate) fn template_subnet_info(subnet: &AvalancheSubnet, list: bool, indent: u8) -> String {
    let mut info = String::new();

    let subindent = match list {
        true => 2,
        false => 2,
    };

    let mut blockchains_info = String::new();
    for blockchain in subnet.blockchains.iter() {
        blockchains_info.push_str(&format!(
            "\n{}",
            template_blockchain_info(blockchain, true, subindent)
        ));
    }

    let mut validators_info = String::new();
    for validator in subnet.validators.iter() {
        validators_info.push_str(&format!(
            "\n{}",
            template_validator_info(validator, subnet, true, subindent, false)
        ));
    }

    let permissioned_subnet_info = &formatdoc!(
        "
        Control keys: {}
        Threshold:    {}
        ",
        type_colorize(&format!("{:?}", subnet.control_keys)),
        type_colorize(&subnet.threshold),
    );

    if list {
        info.push_str(&formatdoc!(
            "
            {}
            - {}:
              Type: {}
            {}  Blockchains list ({}): {}",
            template_horizontal_rule('-', format!("- '{}':", subnet.id).len()),
            type_colorize(&subnet.id),
            type_colorize(&subnet.subnet_type.to_string()),
            match subnet.subnet_type {
                AvalancheSubnetType::Permissioned =>
                    indent::indent_all_by(subindent.into(), permissioned_subnet_info),
                _ => "".to_string(),
            },
            type_colorize(&subnet.blockchains.len()),
            match blockchains_info.is_empty() {
                true => String::from("[]"),
                false => blockchains_info,
            }
        ));
    } else {
        info.push_str(&formatdoc!(
            "
            Subnet '{}':
              Type: {}
            {}  Blockchains list ({}): {}
              Validators list ({}): {}",
            type_colorize(&subnet.id),
            type_colorize(&subnet.subnet_type.to_string()),
            match subnet.subnet_type {
                AvalancheSubnetType::Permissioned =>
                    indent::indent_all_by(2, permissioned_subnet_info),
                _ => "".to_string(),
            },
            type_colorize(&subnet.blockchains.len()),
            match blockchains_info.is_empty() {
                true => String::from("[]"),
                false => blockchains_info,
            },
            type_colorize(&subnet.validators.len()),
            match validators_info.is_empty() {
                true => String::from("[]"),
                false => validators_info,
            }
        ));
    }

    indent::indent_all_by(indent.into(), info)
}

pub(crate) fn template_avalanche_node_info(node: &AvalancheNode, indent: u8) -> String {
    let mut info = String::new();

    info.push_str(&formatdoc!(
        "
        Node '{}:{}':
          ID:            {}
          Network:       {}
          Public IP:     {}
          Staking port:  {}
          Versions:
            AvalancheGo: {}
            Database:    {}
            Git commit:  {}
            VMs:
              AVM:        {}
              EVM:        {}
              PlatformVM: {}
          Uptime:
            Rewarding stake:  {}%
            Weighted average: {}%",
        type_colorize(&node.http_host),
        type_colorize(&node.http_port),
        type_colorize(&node.id),
        type_colorize(&node.network),
        type_colorize(&node.public_ip),
        type_colorize(&node.staking_port),
        type_colorize(&node.versions.avalanchego_version),
        type_colorize(&node.versions.database_version),
        type_colorize(&node.versions.git_commit),
        type_colorize(&node.versions.vm_versions.avm),
        type_colorize(&node.versions.vm_versions.evm),
        type_colorize(&node.versions.vm_versions.platform),
        type_colorize(&node.uptime.rewarding_stake_percentage),
        type_colorize(&node.uptime.weighted_average_percentage),
    ));

    indent::indent_all_by(indent.into(), info)
}

pub(crate) fn template_chain_is_bootstrapped(
    node: &AvalancheNode,
    chain: &str,
    is_bootstrapped: bool,
    indent: u8,
) -> String {
    let mut info = String::new();

    info.push_str(&formatdoc!(
        "Chain '{}' on node '{}:{}': {}",
        type_colorize(&chain),
        type_colorize(&node.http_host),
        type_colorize(&node.http_port),
        match is_bootstrapped {
            true => "Bootstrapped ✓".green(),
            false => "Not yet bootstrapped ✗".red(),
        }
    ));

    indent::indent_all_by(indent.into(), info)
}
