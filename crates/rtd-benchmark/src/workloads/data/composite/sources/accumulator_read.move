// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module basics::accumulator_read;

use rtd::accumulator::AccumulatorRoot;
use rtd::balance;
use rtd::event;
use rtd::rtd::RTD;

public struct SettledBalanceEvent has copy, drop {
    value: u64,
}

entry fun read_settled_balance(root: &AccumulatorRoot, addr: address) {
    let value = balance::settled_funds_value<RTD>(root, addr);
    event::emit(SettledBalanceEvent { value });
}
