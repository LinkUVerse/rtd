// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

fn main() {
    cynic_codegen::register_schema("rpc")
        .from_sdl_file("../rtd-indexer-alt-graphql/schema.graphql")
        .expect("Failed to find GraphQL Schema")
        .as_default()
        .unwrap();
}
