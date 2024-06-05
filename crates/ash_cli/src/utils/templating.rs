// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

use crate::console::blueprint::{Blueprint, BlueprintProject};
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
use indicatif::ProgressBar;
use indoc::formatdoc;
use prettytable::{format, Table};
use std::collections::HashMap;
use std::time::Duration;

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

pub(crate) fn spinner_with_message(message: String) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_message(message);
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner
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
        Signer (BLS):
          Public key:     {}
          PoP:            {}
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
        type_colorize(&match validator.signer {
            Some(ref signer) => format!("0x{}", hex::encode(signer.public_key.clone())),
            None => String::from("None"),
        }),
        type_colorize(&match validator.signer {
            Some(ref signer) => format!("0x{}", hex::encode(signer.proof_of_possession.clone())),
            None => String::from("None"),
        }),
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
          Signer (BLS):
            Public key:  {}
            PoP:         {}
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
        type_colorize(&match node.signer {
            Some(ref signer) => format!("0x{}", hex::encode(signer.public_key.clone())),
            None => String::from("None"),
        }),
        type_colorize(&match node.signer {
            Some(ref signer) => format!("0x{}", hex::encode(signer.proof_of_possession.clone())),
            None => String::from("None"),
        }),
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
          Hex private key:  {}
          CB58 private key: {}
          X-Chain address:  {}
          P-Chain address:  {}
          EVM address:      {}",
        type_colorize(&wallet_info.hex_private_key),
        type_colorize(&wallet_info.cb58_private_key),
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
    format!("{}...{}", &uuid[..4], &uuid[uuid.len() - 4..])
}

pub(crate) fn truncate_string(string: &str, length: usize) -> String {
    if length > string.len() {
        return string.to_string();
    }

    format!("{}...", &string[..length])
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
        "Secret name".bold(),
        "Secret ID".bold(),
        "Type".bold(),
        "Created at".bold(),
        "Used by".bold(),
    ]);

    for secret in secrets {
        secrets_table.add_row(row![
            type_colorize(&secret.name.unwrap_or_default()),
            match extended {
                true => type_colorize(&secret.id.unwrap_or_default()),
                false => type_colorize(&truncate_uuid(&secret.id.unwrap_or_default().to_string())),
            },
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
        "Project name".bold(),
        "Project ID".bold(),
        "Network".bold(),
        "Cloud regions".bold(),
        "Resources".bold(),
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

        // Count the number of resources in the project grouped by type
        let mut resources_count: HashMap<String, usize> = HashMap::new();
        for resource in project
            .resources_ids
            .unwrap_or_default()
            .as_object()
            .unwrap()
        {
            let resource_type = resource.1.as_str().unwrap();
            let count = resources_count
                .entry(resource_type.to_string())
                .or_insert(0);
            *count += 1;
        }
        let mut resources_table = Table::new();
        resources_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        for (resource_type, count) in resources_count {
            resources_table.add_row(row![format!(
                "{}: {}",
                resource_type,
                type_colorize(&count)
            ),]);
        }

        projects_table.add_row(row![
            type_colorize(&project.name.unwrap_or_default()),
            match extended {
                true => type_colorize(&project.id.unwrap_or_default()),
                false => type_colorize(&truncate_uuid(&project.id.unwrap_or_default().to_string())),
            },
            type_colorize(&format!("{:?}", project.network.unwrap_or_default())),
            regions_table,
            resources_table,
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
        "Cloud region".bold(),
        "Region ID".bold(),
        "Cloud creds secret ID".bold(),
        "Created at".bold(),
        "Status".bold()
    ]);

    for region in regions {
        regions_table.add_row(row![
            type_colorize(&format!(
                "{}/{}",
                serde_json::to_value(region.cloud_provider.unwrap_or_default())
                    .unwrap()
                    .as_str()
                    .unwrap(),
                region.region.unwrap_or_default()
            )),
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
            match region.status.unwrap_or_default() {
                console::api_models::cloud_region::Status::Available => "Available".green(),
                console::api_models::cloud_region::Status::Destroying => "Destroying".yellow(),
                console::api_models::cloud_region::Status::Suspended => "Suspended".red(),
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

pub(crate) fn template_operations_table(
    operations: Vec<console::api_models::Operation>,
    extended: bool,
    indent: usize,
) -> String {
    let mut operations_table = Table::new();

    operations_table.set_titles(row![
        "Operation ID".bold(),
        "Logged at".bold(),
        "User ID".bold(),
        "Operation type".bold(),
        "Target ID".bold(),
        "Target value".bold(),
        "Result".bold(),
    ]);

    for operation in operations {
        operations_table.add_row(row![
            match extended {
                true => type_colorize(&operation.id.unwrap_or_default()),
                false => type_colorize(&truncate_uuid(
                    &operation.id.unwrap_or_default().to_string()
                )),
            },
            match extended {
                true => type_colorize(&operation.logged.unwrap_or_default()),
                false => type_colorize(&truncate_datetime(&operation.logged.unwrap_or_default())),
            },
            match extended {
                true => type_colorize(&operation.owner_id.unwrap_or_default()),
                false => type_colorize(&truncate_uuid(
                    &operation.owner_id.unwrap_or_default().to_string()
                )),
            },
            type_colorize(&operation.operation_type.unwrap_or_default()),
            type_colorize(&operation.target_id.unwrap_or_default()),
            match extended {
                true => type_colorize(&operation.target_value.unwrap_or_default()),
                false => type_colorize(&truncate_string(
                    &operation.target_value.unwrap_or_default(),
                    20
                )),
            },
            match operation.result.unwrap_or_default() {
                console::api_models::operation::Result::Success => "Success".green(),
                console::api_models::operation::Result::Failure => "Failure".red(),
            },
        ]);
    }

    indent::indent_all_by(indent, operations_table.to_string())
}

pub(crate) fn template_avalanche_node_props_table(
    avalanche_node: &console::api_models::GetAllProjectResources200ResponseInner,
) -> Table {
    let mut props_table = Table::new();
    props_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    props_table.add_row(row![
        "IP address".bold(),
        type_colorize(&avalanche_node.node_ip.clone().unwrap_or_default()),
    ]);
    props_table.add_row(row![
        "Running".bold(),
        type_colorize(
            &avalanche_node
                .node_status
                .clone()
                .unwrap()
                .running
                .unwrap_or_default()
        ),
    ]);
    props_table.add_row(row![
        "Bootstrapped".bold(),
        format!(
            "{:?}",
            &avalanche_node
                .node_status
                .clone()
                .unwrap()
                .bootstrapped
                .unwrap_or_default()
                .as_object()
                .unwrap()
                .values()
                .map(|b| serde_json::from_value::<bool>(b.clone()).unwrap_or_default())
                .collect::<Vec<bool>>()
        ),
    ]);
    props_table.add_row(row![
        "Healthy".bold(),
        format!(
            "{:?}",
            &avalanche_node
                .node_status
                .clone()
                .unwrap()
                .healthy
                .unwrap_or_default()
                .as_object()
                .unwrap()
                .values()
                .map(|b| serde_json::from_value::<bool>(b.clone()).unwrap_or_default())
                .collect::<Vec<bool>>()
        ),
    ]);
    props_table.add_row(row![
        "Restart req.".bold(),
        type_colorize(
            &avalanche_node
                .node_status
                .clone()
                .unwrap()
                .restart_required
                .unwrap_or_default()
        ),
    ]);

    props_table
}

pub(crate) fn template_avalanche_subnet_props_table(
    avalanche_subnet: &console::api_models::GetAllProjectResources200ResponseInner,
) -> Table {
    let mut props_table = Table::new();
    props_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    props_table.add_row(row![
        "ID".bold(),
        type_colorize(
            &avalanche_subnet
                .subnet_status
                .clone()
                .unwrap()
                .id
                .clone()
                .unwrap_or_default()
        ),
    ]);
    props_table.add_row(row![
        "Validators",
        type_colorize(
            &avalanche_subnet
                .subnet_status
                .clone()
                .unwrap()
                .validators
                .unwrap()
                .len()
        ),
    ]);

    // TODO: Add the rest of the Subnet properties

    props_table
}

pub(crate) fn template_blockscout_props_table(
    blockscout: &console::api_models::GetAllProjectResources200ResponseInner,
) -> Table {
    let mut props_table = Table::new();
    props_table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    props_table.add_row(row![
        "IP address".bold(),
        type_colorize(&blockscout.blockscout_ip.clone().unwrap_or_default()),
    ]);
    props_table.add_row(row![
        "Running".bold(),
        type_colorize(
            &blockscout
                .blockscout_status
                .clone()
                .unwrap()
                .running
                .unwrap_or_default()
        ),
    ]);
    props_table
}

pub(crate) fn template_resources_table(
    resources: Vec<console::api_models::GetAllProjectResources200ResponseInner>,
    project: console::api_models::Project,
    extended: bool,
    indent: usize,
) -> String {
    use console::api_models::get_all_project_resources_200_response_inner::Status;

    let mut resources_table = Table::new();

    resources_table.set_titles(row![
        "Resource name".bold(),
        "Resource ID".bold(),
        "Type".bold(),
        "Cloud region".bold(),
        "Size".bold(),
        "Created at".bold(),
        "Status".bold(),
        "Resource specific".bold(),
    ]);

    for resource in resources {
        resources_table.add_row(row![
            type_colorize(&resource.name.clone().unwrap_or_default()),
            match extended {
                true => type_colorize(&resource.id.unwrap_or_default()),
                false =>
                    type_colorize(&truncate_uuid(&resource.id.unwrap_or_default().to_string())),
            },
            type_colorize(&format!(
                "{:?}",
                resource.resource_type.clone().unwrap_or_default()
            )),
            type_colorize(
                // Get the cloud region name from the project
                // It has to be found by the cloud region ID (value of the cloud_regions_ids object)
                &project
                    .cloud_regions_ids
                    .clone()
                    .unwrap_or_default()
                    .as_object()
                    .unwrap()
                    .into_iter()
                    .find(|(_, region_id)| region_id.as_str().unwrap()
                        == resource.cloud_region_id.as_ref().unwrap())
                    .unwrap()
                    .0
            ),
            type_colorize(&format!("{:?}", resource.size.unwrap_or_default())),
            match extended {
                true => type_colorize(&resource.created.clone().unwrap_or_default()),
                false => type_colorize(&truncate_datetime(
                    &resource.created.clone().unwrap_or_default()
                )),
            },
            match resource.status.unwrap_or_default() {
                Status::Pending => "Pending".yellow(),
                Status::Configuring => "Configuring".blue(),
                Status::Running => "Running".green(),
                Status::Error => "Error".red(),
                Status::Destroying => "Destroying".yellow(),
                Status::Stopped => "Stopped".bright_black(),
            },
            match *resource.resource_type.clone().unwrap_or_default() {
                console::api_models::ResourceType::AvalancheNode => {
                    template_avalanche_node_props_table(&resource.clone())
                }
                console::api_models::ResourceType::AvalancheSubnet => {
                    template_avalanche_subnet_props_table(&resource.clone())
                },
                console::api_models::ResourceType::Blockscout => {
                    template_blockscout_props_table(&resource.clone())
                }
            },
        ]);
    }

    indent::indent_all_by(indent, resources_table.to_string())
}

fn template_blueprint_secrets_list(
    secrets: &[console::api_models::CreateSecretRequest],
) -> ColoredString {
    type_colorize(
        &secrets
            .iter()
            .map(|s| s.name.clone())
            .collect::<Vec<String>>()
            .join(", "),
    )
}

fn template_blueprint_projects_list(projects: &[BlueprintProject]) -> String {
    let mut projects_str = String::new();
    for project in projects.iter() {
        projects_str.push_str(&format!(
            "\n- '{}':{}{}{}{}",
            type_colorize(&project.project.name).bold(),
            match project.regions.len() {
                0 => ColoredString::from(""),
                _ => "\n    Regions: ".to_string().bold(),
            },
            type_colorize(
                &project
                    .regions
                    .iter()
                    .map(|r| format!(
                        "{}/{}",
                        serde_json::to_value(r.cloud_provider.unwrap_or_default())
                            .unwrap()
                            .as_str()
                            .unwrap(),
                        r.region.clone().unwrap_or_default()
                    ))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            match project.resources.len() {
                0 => ColoredString::from(""),
                _ => "\n    Resources: ".to_string().bold(),
            },
            type_colorize(
                &project
                    .resources
                    .iter()
                    .map(|r| r.name.clone())
                    .collect::<Vec<String>>()
                    .join(", "),
            )
        ));
    }

    indent::indent_all_by(2, projects_str)
}

pub(crate) fn template_blueprint_summary(to_create: &Blueprint, to_update: &Blueprint) -> String {
    let mut summary_str = String::new();

    summary_str.push_str(&formatdoc!(
        "
        {}
        {}
          {} to create: {}
          {} to update: {}
        {}
          {} to create:{}
          {} to update:{}",
        "Blueprint summary".bold(),
        "Secrets".bold(),
        type_colorize(&to_create.secrets.len()),
        template_blueprint_secrets_list(&to_create.secrets),
        type_colorize(&to_update.secrets.len()),
        template_blueprint_secrets_list(&to_update.secrets),
        "Projects".bold(),
        type_colorize(&to_create.projects.len()),
        template_blueprint_projects_list(&to_create.projects),
        type_colorize(&to_update.projects.len()),
        template_blueprint_projects_list(&to_update.projects),
    ));

    summary_str
}
