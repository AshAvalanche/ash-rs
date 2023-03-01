// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

// Module that contains error types

pub struct CliError {
    pub message: String,
    pub exit_code: exitcode::ExitCode,
}

impl CliError {
    pub fn dataerr(message: String) -> Self {
        Self {
            message,
            exit_code: exitcode::DATAERR,
        }
    }

    pub fn configerr(message: String) -> Self {
        Self {
            message,
            exit_code: exitcode::CONFIG,
        }
    }

    pub fn cantcreat(message: String) -> Self {
        Self {
            message,
            exit_code: exitcode::CANTCREAT,
        }
    }
}
