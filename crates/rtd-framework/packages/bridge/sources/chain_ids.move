// Copyright (c) LinkU Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module bridge::chain_ids;

// Chain IDs
const RTD_MAINNET: u8 = 0;
const RTD_TESTNET: u8 = 1;
const RTD_CUSTOM: u8 = 2;

const ETH_MAINNET: u8 = 10;
const ETH_SEPOLIA: u8 = 11;
const ETH_CUSTOM: u8 = 12;

const EInvalidBridgeRoute: u64 = 0;

//////////////////////////////////////////////////////
// Types
//

public struct BridgeRoute has copy, drop, store {
    source: u8,
    destination: u8,
}

//////////////////////////////////////////////////////
// Public functions
//

public fun rtd_mainnet(): u8 { RTD_MAINNET }

public fun rtd_testnet(): u8 { RTD_TESTNET }

public fun rtd_custom(): u8 { RTD_CUSTOM }

public fun eth_mainnet(): u8 { ETH_MAINNET }

public fun eth_sepolia(): u8 { ETH_SEPOLIA }

public fun eth_custom(): u8 { ETH_CUSTOM }

public use fun route_source as BridgeRoute.source;

public fun route_source(route: &BridgeRoute): &u8 {
    &route.source
}

public use fun route_destination as BridgeRoute.destination;

public fun route_destination(route: &BridgeRoute): &u8 {
    &route.destination
}

public fun assert_valid_chain_id(id: u8) {
    assert!(
        id == RTD_MAINNET ||
        id == RTD_TESTNET ||
        id == RTD_CUSTOM ||
        id == ETH_MAINNET ||
        id == ETH_SEPOLIA ||
        id == ETH_CUSTOM,
        EInvalidBridgeRoute,
    )
}

public fun valid_routes(): vector<BridgeRoute> {
    vector[
        BridgeRoute { source: RTD_MAINNET, destination: ETH_MAINNET },
        BridgeRoute { source: ETH_MAINNET, destination: RTD_MAINNET },
        BridgeRoute { source: RTD_TESTNET, destination: ETH_SEPOLIA },
        BridgeRoute { source: RTD_TESTNET, destination: ETH_CUSTOM },
        BridgeRoute { source: RTD_CUSTOM, destination: ETH_CUSTOM },
        BridgeRoute { source: RTD_CUSTOM, destination: ETH_SEPOLIA },
        BridgeRoute { source: ETH_SEPOLIA, destination: RTD_TESTNET },
        BridgeRoute { source: ETH_SEPOLIA, destination: RTD_CUSTOM },
        BridgeRoute { source: ETH_CUSTOM, destination: RTD_TESTNET },
        BridgeRoute { source: ETH_CUSTOM, destination: RTD_CUSTOM },
    ]
}

public fun is_valid_route(source: u8, destination: u8): bool {
    let route = BridgeRoute { source, destination };
    valid_routes().contains(&route)
}

// Checks and return BridgeRoute if the route is supported by the bridge.
public fun get_route(source: u8, destination: u8): BridgeRoute {
    let route = BridgeRoute { source, destination };
    assert!(valid_routes().contains(&route), EInvalidBridgeRoute);
    route
}

//////////////////////////////////////////////////////
// Test functions
//

#[test]
fun test_chains_ok() {
    assert_valid_chain_id(RTD_MAINNET);
    assert_valid_chain_id(RTD_TESTNET);
    assert_valid_chain_id(RTD_CUSTOM);
    assert_valid_chain_id(ETH_MAINNET);
    assert_valid_chain_id(ETH_SEPOLIA);
    assert_valid_chain_id(ETH_CUSTOM);
}

#[test, expected_failure(abort_code = EInvalidBridgeRoute)]
fun test_chains_error() {
    assert_valid_chain_id(100);
}

#[test, expected_failure(abort_code = EInvalidBridgeRoute)]
fun test_rtd_chains_error() {
    // this will break if we add one more rtd chain id and should be corrected
    assert_valid_chain_id(4);
}

#[test, expected_failure(abort_code = EInvalidBridgeRoute)]
fun test_eth_chains_error() {
    // this will break if we add one more eth chain id and should be corrected
    assert_valid_chain_id(13);
}

#[test]
fun test_routes() {
    let valid_routes = vector[
        BridgeRoute { source: RTD_MAINNET, destination: ETH_MAINNET },
        BridgeRoute { source: ETH_MAINNET, destination: RTD_MAINNET },
        BridgeRoute { source: RTD_TESTNET, destination: ETH_SEPOLIA },
        BridgeRoute { source: RTD_TESTNET, destination: ETH_CUSTOM },
        BridgeRoute { source: RTD_CUSTOM, destination: ETH_CUSTOM },
        BridgeRoute { source: RTD_CUSTOM, destination: ETH_SEPOLIA },
        BridgeRoute { source: ETH_SEPOLIA, destination: RTD_TESTNET },
        BridgeRoute { source: ETH_SEPOLIA, destination: RTD_CUSTOM },
        BridgeRoute { source: ETH_CUSTOM, destination: RTD_TESTNET },
        BridgeRoute { source: ETH_CUSTOM, destination: RTD_CUSTOM },
    ];
    let mut size = valid_routes.length();
    while (size > 0) {
        size = size - 1;
        let route = valid_routes[size];
        assert!(is_valid_route(route.source, route.destination)); // sould not assert
    }
}

#[test, expected_failure(abort_code = EInvalidBridgeRoute)]
fun test_routes_err_rtd_1() {
    get_route(RTD_MAINNET, RTD_MAINNET);
}

#[test, expected_failure(abort_code = EInvalidBridgeRoute)]
fun test_routes_err_rtd_2() {
    get_route(RTD_MAINNET, RTD_TESTNET);
}

#[test, expected_failure(abort_code = EInvalidBridgeRoute)]
fun test_routes_err_rtd_3() {
    get_route(RTD_MAINNET, ETH_SEPOLIA);
}

#[test, expected_failure(abort_code = EInvalidBridgeRoute)]
fun test_routes_err_rtd_4() {
    get_route(RTD_MAINNET, ETH_CUSTOM);
}

#[test, expected_failure(abort_code = EInvalidBridgeRoute)]
fun test_routes_err_eth_1() {
    get_route(ETH_MAINNET, ETH_MAINNET);
}

#[test, expected_failure(abort_code = EInvalidBridgeRoute)]
fun test_routes_err_eth_2() {
    get_route(ETH_MAINNET, ETH_CUSTOM);
}

#[test, expected_failure(abort_code = EInvalidBridgeRoute)]
fun test_routes_err_eth_3() {
    get_route(ETH_MAINNET, RTD_CUSTOM);
}

#[test, expected_failure(abort_code = EInvalidBridgeRoute)]
fun test_routes_err_eth_4() {
    get_route(ETH_MAINNET, RTD_TESTNET);
}
