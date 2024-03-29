// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

use colored::Colorize;
use inquire::Confirm;

pub(crate) fn confirm_action(entity_type: &str, action: Option<&str>) -> bool {
    let action = action.unwrap_or("delete");

    let confirmation = Confirm::new(&format!(
        "Are you sure you want to {action} this {entity_type}?"
    ))
    .with_default(false)
    .with_help_message("This action is irreversible!")
    .prompt();

    match confirmation {
        Ok(true) => true,
        Ok(false) => {
            println!("Aborting {} action.", action.yellow());
            false
        }
        Err(_) => {
            println!(
                "{}",
                format!("Error parsing answer. Aborting {} action.", action).red()
            );
            false
        }
    }
}

pub(crate) fn confirm_restart(resource_type: &str) -> bool {
    let confirmation = Confirm::new(&format!(
        "Are you sure you want to restart this {resource_type}?"
    ))
    .with_default(false)
    .with_help_message("This action might significanlty impact the uptime of the resource!")
    .prompt();

    match confirmation {
        Ok(true) => true,
        Ok(false) => {
            println!("Aborting restart.");
            false
        }
        Err(_) => {
            println!("{}", "Error parsing answer. Aborting restart.".red());
            false
        }
    }
}
