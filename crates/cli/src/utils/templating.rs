// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

use ash::avalanche::{blockchains::AvalancheBlockchain, nodes::AvalancheNode, subnets};
use ash::nodes::AshNodeInfo;
use indoc::formatdoc;

// Module that contains templating functions for info strings

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
            blockchain.name,
            blockchain.id,
            blockchain.vm_type,
            blockchain.rpc_url,
        ));
    } else {
        info.push_str(&formatdoc!(
            "
            Blockchain '{}':
              ID:      {}
              VM type: {}
              RPC URL: {}",
            blockchain.name,
            blockchain.id,
            blockchain.vm_type,
            blockchain.rpc_url,
        ));
    }

    indent::indent_all_by(indent.into(), info)
}

pub(crate) fn template_validator_info(
    validator: &subnets::AvalancheSubnetValidator,
    list: bool,
    indent: u8,
) -> String {
    let mut info = String::new();

    if list {
        info.push_str(&formatdoc!(
            "
            - {}",
            validator.node_id,
        ));
    } else {
        info.push_str(&formatdoc!(
            "
            Validator '{}'",
            validator.node_id,
        ));
    }

    indent::indent_all_by(indent.into(), info)
}

pub(crate) fn template_subnet_info(
    subnet: &subnets::AvalancheSubnet,
    list: bool,
    indent: u8,
) -> String {
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
            template_validator_info(validator, true, subindent)
        ));
    }

    if list {
        info.push_str(&formatdoc!(
            "
            {}
            - {}:
               Number of blockchains: {}
               Control keys: {:?}
               Threshold: {}
               Blockchains: {}",
            template_horizontal_rule('-', format!("- '{}':", subnet.id).len()),
            subnet.id,
            subnet.blockchains.len(),
            subnet.control_keys,
            subnet.threshold,
            blockchains_info,
        ));
    } else {
        info.push_str(&formatdoc!(
            "
            Subnet '{}':
              Number of blockchains: {}
              Control keys: {:?}
              Threshold: {}
              Blockchains: {}
              Validators: {}",
            subnet.id,
            subnet.blockchains.len(),
            subnet.control_keys,
            subnet.threshold,
            blockchains_info,
            validators_info,
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
        node.http_host,
        node.http_port,
        node.id,
        node.public_ip,
        node.staking_port,
        node.versions.avalanchego_version,
        node.versions.database_version,
        node.versions.git_commit,
        node.versions.vm_versions.avm,
        node.versions.vm_versions.evm,
        node.versions.vm_versions.platform,
        node.uptime.rewarding_stake_percentage,
        node.uptime.weighted_average_percentage,
    ));

    indent::indent_all_by(indent.into(), info)
}

pub(crate) fn template_ash_node_info(node_info: &AshNodeInfo, list: bool, indent: u8) -> String {
    let mut info = String::new();

    if list {
        info.push_str(&formatdoc!(
            "
            {}
            - {}:
               Node ID (CB58): {}
               RPC URL (hex):  {}
               Bytes:          {:?}",
            template_horizontal_rule('-', format!("- '{}':", node_info.id.p_chain).len()),
            node_info.id.p_chain,
            node_info.id.cb58,
            node_info.id.hex,
            node_info.id.bytes,
        ));
    } else {
        info.push_str(&formatdoc!(
            "
            Node '{}':
              Node ID (CB58): {}
              RPC URL (hex):  {}
              Bytes:          {:?}",
            node_info.id.p_chain,
            node_info.id.cb58,
            node_info.id.hex,
            node_info.id.bytes,
        ));
    }

    indent::indent_all_by(indent.into(), info)
}
