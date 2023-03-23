// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

use ethers::prelude::*;
use std::env;
use std::path::Path;

// Generate Rust bindings for a contract using Abigen
fn gen_contract_bindings(contract: &str, module: &str) {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(module);

    Abigen::new(contract, format!("./contracts/{contract}.json"))
        .unwrap()
        .generate()
        .unwrap()
        .write_to_file(dest_path)
        .unwrap();
}

fn main() {
    // Generate bindings for Ash contracts
    gen_contract_bindings("AshRouter", "ash_router_abigen.rs");

    println!("cargo:rerun-if-changed=build.rs, ./contracts");
}
