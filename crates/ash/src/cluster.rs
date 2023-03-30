// SPDX-License-Identifier: BSD-3-Clause
// Copyright (c) 2023, E36 Knots

use serde::{Deserialize, Serialize};
use typify::import_types;

import_types!(
    schema = "json-schemas/ash.avalanche.schema.json",
    struct_builder = true
);

pub fn dump_default_conf() {
    let node_conf = NodeConf::builder();
}
