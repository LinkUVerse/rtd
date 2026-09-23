#!/bin/bash

RTD_FRAMEWORK_DIR="../../../../crates/rtd-framework/packages/rtd-framework/**/*.move"
STDLIB_DIR="../../../../rtd-framework/packages/move-stdlib/**/*.move"

tree-sitter generate --no-bindings
tree-sitter parse -q -t tests/*.move
tree-sitter parse -q -t tree-sitter $RTD_FRAMEWORK_DIR
