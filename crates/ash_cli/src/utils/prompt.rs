// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

use colored::Colorize;
use inquire::Confirm;

pub(crate) fn confirm_deletion(object_type: &str, action: Option<&str>) -> bool {
    let action = action.unwrap_or("delete");

    let confirmation = Confirm::new(&format!(
        "Are you sure you want to {action} this {object_type}?"
    ))
    .with_default(false)
    .with_help_message("This action is irreversible!")
    .prompt();

    match confirmation {
        Ok(true) => true,
        Ok(false) => {
            println!("Aborting deletion.");
            false
        }
        Err(_) => {
            println!("{}", "Error parsing answer. Aborting deletion.".red());
            false
        }
    }
}
