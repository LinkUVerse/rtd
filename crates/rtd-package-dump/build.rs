// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

fn main() {
    cynic_codegen::register_schema("rtd")
        .from_sdl_file("../rtd-graphql-rpc/schema.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
