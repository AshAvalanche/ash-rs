// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

use ash_sdk::{
    avalanche::{
        blockchains::AvalancheBlockchain,
        nodes::AvalancheNode,
        subnets::{AvalancheSubnet, AvalancheSubnetType, AvalancheSubnetValidator},
        vms::subnet_evm::warp::{AddressedPayload, SubnetEVMWarpMessage},
        wallets::AvalancheWalletInfo,
        warp::{
            VerifiedWarpMessage, WarpMessage, WarpMessageNodeSignature, WarpMessagePayload,
            WarpMessageStatus,
        },
        AvalancheXChainBalance,
    },
    console,
};
use chrono::{DateTime, NaiveDateTime, Utc};
use colored::{ColoredString, Colorize};
use indoc::formatdoc;
use prettytable::{format, Table};
use std::collections::HashMap;

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
        "Id" | "Uuid" => var.to_string().green(),
        _ => var.to_string().bright_white(),
    }
}

pub(crate) fn human_readable_timestamp(timestamp: u64) -> String {
    DateTime::<Utc>::from_naive_utc_and_offset(
        NaiveDateTime::from_timestamp_opt(timestamp as i64, 0).unwrap(),
        Utc,
    )
    .format("%Y-%m-%d %H:%M:%S")
    .to_string()
}

pub(crate) fn template_horizontal_rule(character: char, length: usize) -> String {
    format!("{character}").repeat(length)
}

pub(crate) fn template_blockchain_info(
    blockchain: &AvalancheBlockchain,
    list: bool,
    indent: usize,
) -> String {
    let mut info_str = String::new();

    if list {
        info_str.push_str(&formatdoc!(
            "
            - '{}':
              ID:      {}
              VM ID:   {}
              VM type: {}{}",
            type_colorize(&blockchain.name),
            type_colorize(&blockchain.id),
            type_colorize(&blockchain.vm_id),
            type_colorize(&blockchain.vm_type),
            if !blockchain.rpc_url.is_empty() {
                indent::indent_all_by(
                    2,
                    formatdoc!(
                        "

                    RPC URL: {}",
                        type_colorize(&blockchain.rpc_url)
                    ),
                )
            } else {
                "".to_string()
            }
        ));
    } else {
        info_str.push_str(&formatdoc!(
            "
            Blockchain '{}':
              ID:      {}
              VM ID:   {}
              VM type: {}{}",
            type_colorize(&blockchain.name),
            type_colorize(&blockchain.id),
            type_colorize(&blockchain.vm_id),
            type_colorize(&blockchain.vm_type),
            if !blockchain.rpc_url.is_empty() {
                indent::indent_all_by(
                    2,
                    formatdoc!(
                        "

                    RPC URL: {}",
                        type_colorize(&blockchain.rpc_url)
                    ),
                )
            } else {
                "".to_string()
            }
        ));
    }

    indent::indent_all_by(indent, info_str)
}

pub(crate) fn template_validator_info(
    validator: &AvalancheSubnetValidator,
    subnet: &AvalancheSubnet,
    list: bool,
    extended: bool,
    indent: usize,
) -> String {
    let mut info_str = String::new();

    let common_info = &formatdoc!(
        "
        Tx ID:            {}
        Start time (UTC): {}
        End time (UTC):   {}
        ",
        type_colorize(&validator.tx_id),
        type_colorize(&human_readable_timestamp(validator.start_time)),
        type_colorize(&human_readable_timestamp(validator.end_time)),
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
        Validation reward owner:
          Locktime: {}
          Threshold: {}
          Addresses: {}
        Delegator count:  {}
        Delegator weight: {}
        Delegation fee:   {}%
        Delegation reward owner:
          Locktime: {}
          Threshold: {}
          Addresses: {}",
        type_colorize(&validator.connected),
        type_colorize(&validator.uptime.unwrap_or_default()),
        type_colorize(&validator.stake_amount.unwrap_or_default()),
        type_colorize(&validator.potential_reward.unwrap_or_default()),
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
        type_colorize(&validator.delegation_fee.unwrap_or_default()),
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
            info_str.push_str(&formatdoc!(
                "
                - '{}':
                ",
                type_colorize(&validator.node_id),
            ));

            info_str.push_str(&indent::indent_all_by(4, common_info));

            // Display extra information if the validator is a primary validator
            match subnet.subnet_type {
                AvalancheSubnetType::Permissioned => {
                    info_str.push_str(&indent::indent_all_by(4, permissioned_subnet_info));
                }
                AvalancheSubnetType::Elastic | AvalancheSubnetType::PrimaryNetwork => {
                    info_str.push_str(&indent::indent_all_by(4, elastic_subnet_info));
                }
            }
        } else {
            info_str.push_str(&formatdoc!(
                "
            - {}",
                type_colorize(&validator.node_id),
            ));
        }
    } else {
        info_str.push_str(&formatdoc!(
            "
            Validator '{}' on Subnet '{}':
            ",
            type_colorize(&validator.node_id),
            type_colorize(&validator.subnet_id),
        ));

        info_str.push_str(&indent::indent_all_by(2, common_info));

        // Display extra information if the validator is a primary validator
        match subnet.subnet_type {
            AvalancheSubnetType::Permissioned => {
                info_str.push_str(&indent::indent_all_by(2, permissioned_subnet_info));
            }
            AvalancheSubnetType::Elastic | AvalancheSubnetType::PrimaryNetwork => {
                info_str.push_str(&indent::indent_all_by(2, elastic_subnet_info));
            }
        }
    }

    indent::indent_all_by(indent, info_str)
}

pub(crate) fn template_subnet_info(
    subnet: &AvalancheSubnet,
    list: bool,
    extended: bool,
    indent: usize,
) -> String {
    let mut info_str = String::new();

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
            template_validator_info(validator, subnet, true, extended, subindent)
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
        info_str.push_str(&formatdoc!(
            "
            {}
            - '{}':
              Type: {}
            {}  Blockchains list ({}): {}",
            template_horizontal_rule('-', format!("- '{}':", subnet.id).len()),
            type_colorize(&subnet.id),
            type_colorize(&subnet.subnet_type.to_string()),
            match subnet.subnet_type {
                AvalancheSubnetType::Permissioned =>
                    indent::indent_all_by(subindent, permissioned_subnet_info),
                _ => "".to_string(),
            },
            type_colorize(&subnet.blockchains.len()),
            match blockchains_info.is_empty() {
                true => String::from("[]"),
                false => blockchains_info,
            }
        ));
    } else {
        info_str.push_str(&formatdoc!(
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

    indent::indent_all_by(indent, info_str)
}

pub(crate) fn template_subnet_creation(subnet: &AvalancheSubnet, wait: bool) -> String {
    if wait {
        formatdoc!(
            "
            Subnet created! (Tx ID: '{}')
            {}",
            type_colorize(&subnet.id),
            template_subnet_info(subnet, false, false, 0)
        )
    } else {
        formatdoc!(
            "
            Initiated subnet creation! (Tx ID: '{}')
            {}",
            type_colorize(&subnet.id),
            template_subnet_info(subnet, false, false, 0)
        )
    }
}

pub(crate) fn template_blockchain_creation(blockchain: &AvalancheBlockchain, wait: bool) -> String {
    if wait {
        formatdoc!(
            "
            Blockchain created! (Tx ID: '{}')
            {}",
            type_colorize(&blockchain.id),
            template_blockchain_info(blockchain, false, 0)
        )
    } else {
        formatdoc!(
            "
            Initiated blockchain creation! (Tx ID: '{}')
            {}",
            type_colorize(&blockchain.id),
            template_blockchain_info(blockchain, false, 0)
        )
    }
}

pub(crate) fn template_validator_add(
    validator: &AvalancheSubnetValidator,
    subnet: &AvalancheSubnet,
    wait: bool,
) -> String {
    if wait {
        formatdoc!(
            "
            Validator added to Subnet! (Tx ID: '{}')
            {}",
            type_colorize(&validator.node_id),
            template_validator_info(validator, subnet, false, true, 0)
        )
    } else {
        formatdoc!(
            "
            Initiated validator addition to Subnet! (Tx ID: '{}')
            {}",
            type_colorize(&validator.node_id),
            template_validator_info(validator, subnet, false, true, 0)
        )
    }
}

pub(crate) fn template_avalanche_node_info(node: &AvalancheNode, indent: usize) -> String {
    let mut info_str = String::new();

    let mut subnet_vm_versions = String::new();
    for (vm, version) in node.versions.vm_versions.subnets.iter() {
        subnet_vm_versions.push_str(&format!(
            "\n'{}': {}",
            type_colorize(vm),
            type_colorize(version),
        ));
    }

    info_str.push_str(&formatdoc!(
        "
        Node '{}:{}':
          ID:            {}
          Network:       {}
          Public IP:     {}
          Staking port:  {}
          Versions:
            AvalancheGo:  {}
            Database:     {}
            RPC Protocol: {}
            Git commit:   {}
            VMs:
              AvalancheVM: {}
              Coreth:      {}
              PlatformVM:  {}
              Subnet VMs:{}
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
        type_colorize(&node.versions.rpc_protocol_version),
        type_colorize(&node.versions.git_commit),
        type_colorize(&node.versions.vm_versions.avm),
        type_colorize(&node.versions.vm_versions.evm),
        type_colorize(&node.versions.vm_versions.platform),
        match subnet_vm_versions.is_empty() {
            true => String::from("  []"),
            false => indent::indent_all_by(8, subnet_vm_versions),
        },
        type_colorize(&node.uptime.rewarding_stake_percentage),
        type_colorize(&node.uptime.weighted_average_percentage),
    ));

    indent::indent_all_by(indent, info_str)
}

pub(crate) fn template_chain_is_bootstrapped(
    node: &AvalancheNode,
    chain: &str,
    is_bootstrapped: bool,
    indent: usize,
) -> String {
    let mut bootstrapped_str = String::new();

    bootstrapped_str.push_str(&formatdoc!(
        "Chain '{}' on node '{}:{}': {}",
        type_colorize(&chain),
        type_colorize(&node.http_host),
        type_colorize(&node.http_port),
        match is_bootstrapped {
            true => "Bootstrapped ✓".green(),
            false => "Not yet bootstrapped ✗".red(),
        }
    ));

    indent::indent_all_by(indent, bootstrapped_str)
}

pub(crate) fn template_generate_private_key(
    private_key_cb58: &str,
    private_key_hex: &str,
    indent: usize,
) -> String {
    let mut private_key_str = String::new();

    private_key_str.push_str(&formatdoc!(
        "
        Private key (CB58): {}
        Private key (hex):  {}",
        type_colorize(&private_key_cb58),
        type_colorize(&private_key_hex),
    ));

    indent::indent_all_by(indent, private_key_str)
}

pub(crate) fn template_wallet_info(wallet_info: &AvalancheWalletInfo, indent: usize) -> String {
    let mut info_str = String::new();

    info_str.push_str(&formatdoc!(
        "
        Wallet information:
          X-Chain address: {}
          P-Chain address: {}
          EVM address:     {}",
        type_colorize(&wallet_info.xchain_address),
        type_colorize(&wallet_info.pchain_address),
        type_colorize(&wallet_info.evm_address),
    ));

    indent::indent_all_by(indent, info_str)
}

pub(crate) fn template_xchain_balance(
    address: &str,
    asset_id: &str,
    balance: &AvalancheXChainBalance,
    indent: usize,
) -> String {
    let mut balance_str = String::new();

    balance_str.push_str(&formatdoc!(
        "Balance of '{}' on X-Chain (asset '{}'):  {}",
        type_colorize(&address),
        type_colorize(&asset_id),
        type_colorize(&(balance.balance as f64 / 1_000_000_000.0)),
    ));

    indent::indent_all_by(indent, balance_str)
}

pub(crate) fn template_xchain_transfer(
    tx_id: &str,
    to: &str,
    asset_id: &str,
    amount: f64,
    wait: bool,
    indent: usize,
) -> String {
    let mut transfer_str = String::new();

    if wait {
        transfer_str.push_str(&formatdoc!(
            "
            Transfered {} of asset '{}' to '{}'!
            Transaction ID: {}",
            type_colorize(&amount),
            type_colorize(&asset_id),
            type_colorize(&to),
            type_colorize(&tx_id),
        ));
    } else {
        transfer_str.push_str(&formatdoc!(
            "
            Initiated transfering {} of asset '{}' to '{}'!
            Transaction ID: {}",
            type_colorize(&amount),
            type_colorize(&asset_id),
            type_colorize(&to),
            type_colorize(&tx_id),
        ));
    }

    indent::indent_all_by(indent, transfer_str)
}

pub(crate) fn template_genesis_encoded(genesis_bytes: Vec<u8>, indent: usize) -> String {
    let mut genesis_str = String::new();

    genesis_str.push_str(&formatdoc!(
        "
        Genesis bytes:
          {}",
        type_colorize(&format!("0x{}", hex::encode(genesis_bytes))),
    ));

    indent::indent_all_by(indent, genesis_str)
}

pub(crate) fn template_warp_message(
    message: &WarpMessage,
    blockchain: &AvalancheBlockchain,
    extended: bool,
    list: bool,
    indent: usize,
) -> String {
    let mut message_str = String::new();
    let sub_indent = match list {
        true => 2,
        false => 0,
    };

    if message.unsigned_message.source_chain_id != blockchain.id {
        return format!(
            "{}Couldn't decode message. Only Warp messages created by AvalancheGo > 1.10.5 are supported.",
            match list {
                true => "- ".to_string(),
                false => "".to_string(),
            }
        ).yellow().to_string();
    }

    let unsigned_message_str = indent::indent_all_by(
        sub_indent,
        formatdoc!(
            "
            Unsigned message:
              ID:            {}
              NetworkID:     {}
              SourceChainID: {}
            {}",
            type_colorize(&message.unsigned_message.id),
            type_colorize(&message.unsigned_message.network_id),
            type_colorize(&message.unsigned_message.source_chain_id),
            match &message.unsigned_message.payload {
                WarpMessagePayload::SubnetEVMAddressedPayload(addressed_payload) =>
                    template_warp_addressed_payload(addressed_payload, 2),
                WarpMessagePayload::Unknown(payload) => format!(
                    "Payload (Unknown): {}",
                    type_colorize(&format!("0x{}", hex::encode(payload)))
                ),
            }
        ),
    );

    message_str.push_str(&formatdoc!(
        "
            {}Message '{}' from '{}':
              Status: {}
            {}
            {}
            {}",
        match list {
            true => "- ".to_string(),
            false => "".to_string(),
        },
        type_colorize(&message.unsigned_message.id),
        type_colorize(&blockchain.name),
        match message.status {
            WarpMessageStatus::Sent => "Sent".yellow(),
            WarpMessageStatus::Signed(num) => format!("Signed by {num} validator nodes").green(),
        },
        unsigned_message_str,
        match &message.verified_message {
            VerifiedWarpMessage::SubnetEVM(verified_message) =>
                template_warp_subnet_evm_message(verified_message, 2),
            VerifiedWarpMessage::Unknown => "".to_string(),
        },
        match extended {
            true => template_warp_node_signatures(&message.node_signatures, 2),
            false => "".to_string(),
        }
    ));

    indent::indent_all_by(indent, message_str)
}

pub(crate) fn template_warp_addressed_payload(payload: &AddressedPayload, indent: usize) -> String {
    let mut payload_str = String::new();

    payload_str.push_str(&formatdoc!(
        "
        Payload ({}):
          SourceAddress:      {}
          DestinationChainID: {}
          DestinationAddress: {}
          Payload:            {}",
        type_colorize(&"AddressedPayload".to_string()),
        type_colorize(&format!("{:?}", payload.source_address)),
        type_colorize(&format!("{:?}", payload.destination_chain_id)),
        type_colorize(&format!("{:?}", payload.destination_address)),
        type_colorize(&payload.payload),
    ));

    indent::indent_all_by(indent, payload_str)
}

pub(crate) fn template_warp_subnet_evm_message(
    message: &SubnetEVMWarpMessage,
    indent: usize,
) -> String {
    let mut message_str = String::new();

    message_str.push_str(&formatdoc!(
        "
        Verified message ({}):
          OriginChainID:       {}
          OriginSenderAddress: {}
          DestinationChainID:  {}
          DestinationAddress:  {}
          Payload:             {}",
        type_colorize(&"Subnet-EVM".to_string()),
        type_colorize(&format!("{:?}", message.origin_chain_id)),
        type_colorize(&format!("{:?}", message.origin_sender_address)),
        type_colorize(&format!("{:?}", message.destination_chain_id)),
        type_colorize(&format!("{:?}", message.destination_address)),
        match message.payload {
            Some(ref payload) => type_colorize(payload),
            None => type_colorize(&"None".to_string()),
        }
    ));

    indent::indent_all_by(indent, message_str)
}

pub(crate) fn template_warp_node_signatures(
    signatures: &Vec<WarpMessageNodeSignature>,
    indent: usize,
) -> String {
    let mut signatures_str = String::new();

    signatures_str.push_str(&formatdoc!(
        "
        Signatures ({}):
        ",
        type_colorize(&signatures.len()),
    ));

    for signature in signatures {
        signatures_str.push_str(&formatdoc!(
            "
            - {}: {}
            ",
            type_colorize(&signature.node_id),
            type_colorize(&format!("0x{}", hex::encode(signature.signature)))
        ))
    }

    indent::indent_all_by(indent, signatures_str)
}

pub(crate) fn truncate_uuid(uuid: &str) -> String {
    format!("{}...{}", &uuid[..6], &uuid[uuid.len() - 6..])
}

/// Truncate a datetime string to the format "YYYY-MM-DD HH:MM"
/// Example: "2021-08-31T14:00:00.000000" -> "2021-08-31T14:00"
pub(crate) fn truncate_datetime(datetime: &str) -> String {
    datetime[..16].to_string()
}

pub(crate) fn template_secrets_table(
    secrets: Vec<console::api_models::Secret>,
    extended: bool,
    indent: usize,
) -> String {
    let mut secrets_table = Table::new();

    secrets_table.set_titles(row![
        "Secret ID".bold(),
        "Owner ID".bold(),
        "Name".bold(),
        "Type".bold(),
        "Created at".bold(),
        "Used by".bold(),
    ]);

    for secret in secrets {
        secrets_table.add_row(row![
            type_colorize(&secret.id.unwrap_or_default()),
            match extended {
                true => type_colorize(&secret.owner_id.unwrap_or_default()),
                false => type_colorize(&truncate_uuid(
                    &secret.owner_id.unwrap_or_default().to_string()
                )),
            },
            type_colorize(&secret.name.unwrap_or_default()),
            type_colorize(&format!("{:?}", secret.secret_type.unwrap_or_default())),
            match extended {
                true => type_colorize(&secret.created.unwrap_or_default()),
                false => type_colorize(&truncate_datetime(&secret.created.unwrap_or_default())),
            },
            type_colorize(
                &serde_json::from_value::<HashMap<String, String>>(
                    secret.used_by.unwrap_or_default()
                )
                .unwrap_or_default()
                .len()
            ),
        ]);
    }

    indent::indent_all_by(indent, secrets_table.to_string())
}

pub(crate) fn template_projects_table(
    projects: Vec<console::api_models::Project>,
    extended: bool,
    indent: usize,
) -> String {
    let mut projects_table = Table::new();

    projects_table.set_titles(row![
        "Project ID".bold(),
        "Owner ID".bold(),
        "Name".bold(),
        "Network".bold(),
        "Cloud regions".bold(),
        "Created at".bold(),
    ]);

    for project in projects {
        let mut regions_table = Table::new();
        regions_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

        for (region_name, _) in project
            .cloud_regions_ids
            .clone()
            .unwrap_or_default()
            .as_object()
            .unwrap()
        {
            regions_table.add_row(row![type_colorize(&region_name),]);
        }

        projects_table.add_row(row![
            type_colorize(&project.id.unwrap_or_default()),
            match extended {
                true => type_colorize(&project.owner_id.unwrap_or_default()),
                false => type_colorize(&truncate_uuid(
                    &project.owner_id.unwrap_or_default().to_string()
                )),
            },
            type_colorize(&project.name.unwrap_or_default()),
            type_colorize(&format!("{:?}", project.network.unwrap_or_default())),
            regions_table,
            match extended {
                true => type_colorize(&project.created.unwrap_or_default()),
                false => type_colorize(&truncate_datetime(&project.created.unwrap_or_default())),
            },
        ]);
    }

    indent::indent_all_by(indent, projects_table.to_string())
}

pub(crate) fn template_regions_table(
    regions: Vec<console::api_models::CloudRegion>,
    extended: bool,
    indent: usize,
) -> String {
    let mut regions_table = Table::new();

    regions_table.set_titles(row![
        "Cloud provider".bold(),
        "Cloud region".bold(),
        "Region ID".bold(),
        "Cloud creds secret ID".bold(),
        "Created at".bold(),
    ]);

    for region in regions {
        regions_table.add_row(row![
            type_colorize(
                &serde_json::to_value(region.cloud_provider.unwrap_or_default())
                    .unwrap()
                    .as_str()
                    .unwrap()
            ),
            type_colorize(&region.region.unwrap_or_default()),
            match extended {
                true => type_colorize(&region.id.unwrap_or_default()),
                false => type_colorize(&truncate_uuid(&region.id.unwrap_or_default().to_string())),
            },
            match extended {
                true => type_colorize(&region.cloud_credentials_secret_id.unwrap_or_default()),
                false => type_colorize(&truncate_uuid(
                    &region
                        .cloud_credentials_secret_id
                        .unwrap_or_default()
                        .to_string()
                )),
            },
            match extended {
                true => type_colorize(&region.created.unwrap_or_default()),
                false => type_colorize(&truncate_datetime(&region.created.unwrap_or_default())),
            },
        ]);
    }

    indent::indent_all_by(indent, regions_table.to_string())
}

pub(crate) fn template_available_regions_table(
    provider_regions: serde_json::Value,
    indent: usize,
) -> String {
    let mut provider_regions_table = Table::new();

    provider_regions_table.set_titles(row!["Cloud provider".bold(), "Available regions".bold(),]);

    for provider in provider_regions.as_object().unwrap() {
        let mut regions_table = Table::new();
        regions_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

        for region in provider.1.as_array().unwrap() {
            regions_table.add_row(row![&region.as_str().unwrap_or_default(),]);
        }

        provider_regions_table.add_row(row![&provider.0, regions_table,]);
    }

    indent::indent_all_by(indent, provider_regions_table.to_string())
}
