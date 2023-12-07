// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains file utility functions

use crate::utils::error::CliError;
use atty::Stream;
use base64::{engine, Engine};
use std::{
    fs,
    io::{stdin, Read},
    path::PathBuf,
};

// Read a file and return its content as a string
pub(crate) fn read_file(file_path: PathBuf) -> Result<String, CliError> {
    let file_content = fs::read_to_string(file_path)
        .map_err(|e| CliError::dataerr(format!("Error reading file: {e}")))?;

    Ok(file_content)
}

// Read a file and return its content as a Base64-encoded string
pub(crate) fn read_file_base64(file_path: PathBuf) -> Result<String, CliError> {
    let file_content = read_file(file_path)?;

    Ok(engine::general_purpose::STANDARD.encode(file_content))
}

// Read content from stdin and return it as a string
pub(crate) fn read_stdin() -> Result<String, CliError> {
    let mut content = String::new();

    // Fail if stdin is a TTY
    if atty::is(Stream::Stdin) {
        return Err(CliError::dataerr(
            "Error reading from stdin: stdin was not redirected".to_string(),
        ));
    }

    stdin()
        .read_to_string(&mut content)
        .map_err(|e| CliError::dataerr(format!("Error reading stdin: {e}")))?;

    Ok(content)
}

// Read content from a file or stdin or return the input string unchanged
pub(crate) fn read_file_or_stdin(input_str: &str) -> Result<String, CliError> {
    let file_path = PathBuf::from(input_str);

    let output_str = if file_path.exists() {
        read_file(file_path)?
    } else {
        if input_str == "-" {
            read_stdin()?
        } else {
            input_str.to_string()
        }
    };

    Ok(output_str)
}
