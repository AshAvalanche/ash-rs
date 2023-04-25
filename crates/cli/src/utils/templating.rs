// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

use ash::avalanche::subnets::{AvalancheSubnet, AvalancheSubnetValidator};
use ash::avalanche::{blockchains::AvalancheBlockchain, nodes::AvalancheNode};
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
        "String" => var.to_string().yellow(),
        "&u64" | "&u32" | "&u16" | "&u8" | "&usize" => var.to_string().cyan(),
        "&i64" | "&i32" | "&i16" | "&i8" | "&isize" => var.to_string().cyan(),
        "&f64" | "&f32" => var.to_string().magenta(),
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
    list: bool,
    indent: u8,
    extended: bool,
) -> String {
    let mut info = String::new();

    if list {
        // If extended is true, we want to show all the information
        if extended {
            info.push_str(&formatdoc!(
                "
                - {}:
                  Tx ID:            {}
                  Start time:       {}
                  End time:         {}
                  Stake amount:     {}
                  Weight:           {}
                  Potential reward: {}
                  Delegation fee:   {}
                  Connected:        {}
                  Uptime:           {}
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
                type_colorize(&validator.node_id),
                type_colorize(&validator.tx_id),
                type_colorize(&validator.start_time),
                type_colorize(&validator.end_time),
                type_colorize(&validator.stake_amount),
                type_colorize(&validator.weight),
                type_colorize(&validator.potential_reward),
                type_colorize(&validator.delegation_fee),
                type_colorize(&validator.connected),
                type_colorize(&validator.uptime),
                type_colorize(&validator.validation_reward_owner.locktime),
                type_colorize(&validator.validation_reward_owner.threshold),
                type_colorize(&format!(
                    "{:?}",
                    validator.validation_reward_owner.addresses
                )),
                type_colorize(&validator.delegator_count),
                type_colorize(&validator.delegator_weight),
                type_colorize(&validator.delegation_reward_owner.locktime),
                type_colorize(&validator.delegation_reward_owner.threshold),
                type_colorize(&format!(
                    "{:?}",
                    validator.delegation_reward_owner.addresses
                )),
            ));
        } else {
            info.push_str(&formatdoc!(
                "
            - {}",
                validator.node_id,
            ));
        }
    } else {
        info.push_str(&formatdoc!(
            "
            Validator '{}' on Subnet '{}':
              Tx ID:            {}
              Start time:       {}
              End time:         {}
              Stake amount:     {}
              Weight:           {}
              Potential reward: {}
              Delegation fee:   {}
              Connected:        {}
              Uptime:           {}
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
            type_colorize(&validator.node_id),
            type_colorize(&validator.subnet_id),
            type_colorize(&validator.tx_id),
            type_colorize(&validator.start_time),
            type_colorize(&validator.end_time),
            type_colorize(&validator.stake_amount),
            type_colorize(&validator.weight),
            type_colorize(&validator.potential_reward),
            type_colorize(&validator.delegation_fee),
            type_colorize(&validator.connected),
            type_colorize(&validator.uptime),
            type_colorize(&validator.validation_reward_owner.locktime),
            type_colorize(&validator.validation_reward_owner.threshold),
            type_colorize(&format!(
                "{:?}",
                validator.validation_reward_owner.addresses
            )),
            type_colorize(&validator.delegator_count),
            type_colorize(&validator.delegator_weight),
            type_colorize(&validator.delegation_reward_owner.locktime),
            type_colorize(&validator.delegation_reward_owner.threshold),
            type_colorize(&format!(
                "{:?}",
                validator.delegation_reward_owner.addresses
            )),
        ));
    }

    indent::indent_all_by(indent.into(), info)
}

pub(crate) fn template_subnet_info(subnet: &AvalancheSubnet, list: bool, indent: u8) -> String {
    let mut info = String::new();

    let subindent = match list {
        true => 3,
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
            template_validator_info(validator, true, subindent, false)
        ));
    }

    if list {
        info.push_str(&formatdoc!(
            "
            {}
            - {}:
                Number of blockchains: {}
                Control keys:          {}
                Threshold:             {}
                Blockchains: {}",
            template_horizontal_rule('-', format!("- '{}':", subnet.id).len()),
            type_colorize(&subnet.id),
            type_colorize(&subnet.blockchains.len()),
            type_colorize(&format!("{:?}", subnet.control_keys)),
            type_colorize(&subnet.threshold),
            match blockchains_info.is_empty() {
                true => String::from("None"),
                false => blockchains_info,
            }
        ));
    } else {
        info.push_str(&formatdoc!(
            "
            Subnet '{}':
              Number of blockchains: {}
              Control keys:          {}
              Threshold:             {}
              Blockchains: {}
              Validators: {}",
            type_colorize(&subnet.id),
            type_colorize(&subnet.blockchains.len()),
            type_colorize(&format!("{:?}", subnet.control_keys)),
            type_colorize(&subnet.threshold),
            match blockchains_info.is_empty() {
                true => String::from("None"),
                false => blockchains_info,
            },
            match validators_info.is_empty() {
                true => String::from("None"),
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
          Public IP:     {}
          Stacking port: {}
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
