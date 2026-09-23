# To add a new native Move function

1. Add a module under `crates/rtd-framework/packages/rtd-framework/sources/` or use an existing one.
2. Add the native function declaration in that Move module.
3. Implement the Rust function under `rtd-execution/latest/rtd-move-natives/src/`.
4. Register it in `rtd-execution/latest/rtd-move-natives/src/lib.rs::all_natives`.
5. Write some tests in `{name}_tests.move` and pass `run_framework_move_unit_tests`.
6. Update `crates/rtd-core/src/unit_tests/gas_tests.rs` if the new native changes gas metering.
7. Optionally, run `cargo insta test` and `cargo insta review` since the rtd-framework build will change the empty genesis config.

Note: The gas metering for native functions is currently a WIP; use a dummy value for now and please open an issue with `move` label.
