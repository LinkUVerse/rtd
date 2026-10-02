// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use move_cli::base::{
    self,
    test::{self, UnitTestResult},
};
use move_package_alt_compilation::build_config::BuildConfig;
use move_unit_test::{UnitTestingConfig, vm_test_setup::VMTestSetup};
use move_vm_config::runtime::VMConfig;
use move_vm_runtime::natives::extensions::NativeContextExtensions;
use rtd_adapter::gas_meter::RtdGasMeter;
use rtd_move_build::decorate_warnings;
use rtd_move_natives::{
    NativesCostTable, object_runtime::ObjectRuntime, scratch::ScratchRuntime,
    test_scenario::InMemoryTestStore, transaction_context::TransactionContext,
};
use rtd_package_alt::{RtdFlavor, find_environment};
use rtd_protocol_config::ProtocolConfig;
use rtd_sdk::wallet_context::WalletContext;
use rtd_types::{
    base_types::{RtdAddress, TxContext},
    digests::TransactionDigest,
    gas::{RtdGasStatus, RtdGasStatusAPI},
    gas_model::{tables::GasStatus, units_types::Gas},
    metrics::ExecutionMetrics,
};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    ops::{Deref, DerefMut},
    path::Path,
    rc::Rc,
    sync::{Arc, LazyLock},
};

// Move unit tests will halt after executing this many steps. This is a protection to avoid divergence
pub static MAX_UNIT_TEST_INSTRUCTIONS: LazyLock<u64> =
    LazyLock::new(|| ProtocolConfig::get_for_max_version_UNSAFE().max_tx_gas());

/// Gas price used for the meter during Move unit tests.
const TEST_GAS_PRICE: u64 = 500;

#[derive(Parser)]
#[group(id = "rtd-move-test")]
pub struct Test {
    #[clap(flatten)]
    pub test: test::Test,
}

impl Test {
    pub async fn execute(
        self,
        path: Option<&Path>,
        mut build_config: BuildConfig,
        wallet: &WalletContext,
        flavor: RtdFlavor,
    ) -> anyhow::Result<UnitTestResult> {
        let compute_coverage = self.test.compute_coverage;
        if !cfg!(feature = "tracing") && compute_coverage {
            return Err(anyhow::anyhow!(
                "The --coverage flag is currently supported only in builds built with the `tracing` feature enabled. \
                Please build the Rtd CLI from source with `--features tracing` to use this flag."
            ));
        }
        // save disassembly if trace execution is enabled
        let save_disassembly = self.test.trace.is_some();
        // set the default flavor to Rtd if not already set by the user
        if build_config.default_flavor.is_none() {
            build_config.default_flavor = Some(move_compiler::editions::Flavor::Rtd);
        }

        // find manifest file directory from a given path or (if missing) from current dir
        let rerooted_path = base::reroot_path(path)?;

        // If no gas limit is set, set it to the default max. This allows
        // users to provide custom configs but not have to worry about setting a gas limit unless that
        // is what they care about.
        let unit_test_config = self
            .test
            .unit_test_config(Some(*MAX_UNIT_TEST_INSTRUCTIONS));

        // set the environment (this is a little janky: we get it from the manifest here, then pass
        // it as the optional argument in the build-config, which then looks it up again, but it
        // should be ok.
        let environment =
            find_environment(&rerooted_path, build_config.environment, wallet, false).await?;
        build_config.environment = Some(environment.name);

        run_move_unit_tests(
            &rerooted_path,
            build_config,
            Some(unit_test_config),
            compute_coverage,
            save_disassembly,
            flavor,
        )
        .await
    }
}

/// This function returns a result of UnitTestResult. The outer result indicates whether it
/// successfully started running the test, and the inner result indicatests whether all tests pass.
pub async fn run_move_unit_tests(
    path: &Path,
    build_config: BuildConfig,
    config: Option<UnitTestingConfig>,
    compute_coverage: bool,
    save_disassembly: bool,
    flavor: RtdFlavor,
) -> anyhow::Result<UnitTestResult> {
    let config = config.unwrap_or_else(|| {
        UnitTestingConfig::default_with_bound(Some(*MAX_UNIT_TEST_INSTRUCTIONS))
    });

    let result = move_cli::base::test::run_move_unit_tests(
        path,
        build_config,
        UnitTestingConfig {
            report_stacktrace_on_abort: true,
            ..config
        },
        flavor,
        RtdVMTestSetup::new(),
        compute_coverage,
        save_disassembly,
        &mut std::io::stdout(),
    )
    .await;

    result.map(|(test_result, warning_diags)| {
        if test_result == UnitTestResult::Success
            && let Some(diags) = warning_diags
        {
            decorate_warnings(diags, None);
        }
        test_result
    })
}

pub struct RtdVMTestSetup {
    gas_price: u64,
    reference_gas_price: u64,
    protocol_config: ProtocolConfig,
    native_function_table: move_vm_runtime::natives::functions::NativeFunctionTable,
}

impl Default for RtdVMTestSetup {
    fn default() -> Self {
        Self::new()
    }
}

impl RtdVMTestSetup {
    pub fn new() -> Self {
        let protocol_config = ProtocolConfig::get_for_max_version_UNSAFE();
        let native_function_table =
            rtd_move_natives::all_natives(/* silent */ false, &protocol_config);
        Self {
            gas_price: TEST_GAS_PRICE,
            reference_gas_price: TEST_GAS_PRICE,
            protocol_config,
            native_function_table,
        }
    }

    pub fn max_gas_budget(&self) -> u64 {
        self.protocol_config.max_tx_gas()
    }
}

/// Bundles the in-memory test store with a borrowed protocol config, which is what lets the
/// protocol config be threaded into the native context extensions.
pub struct RtdExtensionsBuilder<'a> {
    store: InMemoryTestStore,
    protocol_config: &'a ProtocolConfig,
}

impl VMTestSetup for RtdVMTestSetup {
    type Meter<'a> = RtdGasMeter<RtdGasStatusTestWrapper>;
    type ExtensionsBuilder<'a> = RtdExtensionsBuilder<'a>;

    fn new_meter<'a>(&'a self, execution_bound: Option<u64>) -> Self::Meter<'a> {
        RtdGasMeter(RtdGasStatusTestWrapper(
            RtdGasStatus::new(
                execution_bound.unwrap_or(*MAX_UNIT_TEST_INSTRUCTIONS),
                self.gas_price,
                self.reference_gas_price,
                &self.protocol_config,
            )
            .unwrap(),
        ))
    }

    fn used_gas<'a>(&'a self, execution_bound: u64, meter: Self::Meter<'a>) -> u64 {
        let gas_status = &meter.0;
        Gas::new(execution_bound)
            .checked_sub(gas_status.remaining_gas())
            .unwrap()
            .into()
    }

    fn vm_config(&self) -> VMConfig {
        rtd_adapter::adapter::vm_config(&self.protocol_config)
    }

    fn native_function_table(&self) -> move_vm_runtime::natives::functions::NativeFunctionTable {
        self.native_function_table.clone()
    }

    fn new_extensions_builder(&self) -> RtdExtensionsBuilder<'_> {
        RtdExtensionsBuilder {
            store: InMemoryTestStore::default(),
            protocol_config: &self.protocol_config,
        }
    }

    fn new_native_context_extensions<'a, 'ext>(
        &'a self,
        builder: &'ext RtdExtensionsBuilder<'a>,
    ) -> NativeContextExtensions<'ext> {
        let mut ext = NativeContextExtensions::default();
        // Use a throwaway metrics registry for testing.
        let registry = prometheus::Registry::new();
        let metrics = Arc::new(ExecutionMetrics::new(&registry));

        let protocol_config = builder.protocol_config;
        ext.add(ObjectRuntime::new(
            &builder.store,
            &builder.store,
            BTreeMap::new(),
            false,
            protocol_config,
            metrics,
            0, // epoch id
        ));
        ext.add(NativesCostTable::from_protocol_config(protocol_config));
        ext.add(ScratchRuntime::new(protocol_config));
        let tx_context = TxContext::new_from_components(
            &RtdAddress::ZERO,
            &TransactionDigest::default(),
            &0,
            0,
            0,
            0,
            0,
            None,
            &self.protocol_config,
        );
        ext.add(TransactionContext::new_for_testing(Rc::new(RefCell::new(
            tx_context,
        ))));
        ext.add(&builder.store);
        ext
    }
}

// Massaging to get traits to line up.
pub struct RtdGasStatusTestWrapper(RtdGasStatus);

impl Deref for RtdGasStatusTestWrapper {
    type Target = GasStatus;

    fn deref(&self) -> &Self::Target {
        self.0.move_gas_status()
    }
}

impl DerefMut for RtdGasStatusTestWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.move_gas_status_mut()
    }
}
