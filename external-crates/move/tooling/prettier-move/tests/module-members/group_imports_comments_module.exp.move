// options:
// printWidth: 80
// useModuleLabel: true
// autoGroupImports: module

module prettier::group_imports_comments_module;

use std::ascii::String as ASCII;
use std::string::String;

// stays in place in module mode too
use rtd::coin::Coin;
use rtd::{balance::Balance, rtd::RTD}; // kept with its comment

fun f(_: Coin<u64>, _: Balance<u64>, _: RTD, _: String, _: ASCII) {
    abort 0
}
