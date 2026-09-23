---
title: Module `bridge::chain_ids`
---



-  [Struct `BridgeRoute`](#bridge_chain_ids_BridgeRoute)
-  [Constants](#@Constants_0)
-  [Function `rtd_mainnet`](#bridge_chain_ids_rtd_mainnet)
-  [Function `rtd_testnet`](#bridge_chain_ids_rtd_testnet)
-  [Function `rtd_custom`](#bridge_chain_ids_rtd_custom)
-  [Function `eth_mainnet`](#bridge_chain_ids_eth_mainnet)
-  [Function `eth_sepolia`](#bridge_chain_ids_eth_sepolia)
-  [Function `eth_custom`](#bridge_chain_ids_eth_custom)
-  [Function `route_source`](#bridge_chain_ids_route_source)
-  [Function `route_destination`](#bridge_chain_ids_route_destination)
-  [Function `assert_valid_chain_id`](#bridge_chain_ids_assert_valid_chain_id)
-  [Function `valid_routes`](#bridge_chain_ids_valid_routes)
-  [Function `is_valid_route`](#bridge_chain_ids_is_valid_route)
-  [Function `get_route`](#bridge_chain_ids_get_route)


<pre><code><b>use</b> <a href="../std/vector.md#std_vector">std::vector</a>;
</code></pre>



<a name="bridge_chain_ids_BridgeRoute"></a>

## Struct `BridgeRoute`



<pre><code><b>public</b> <b>struct</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>source: u8</code>
</dt>
<dd>
</dd>
<dt>
<code>destination: u8</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="bridge_chain_ids_RTD_MAINNET"></a>



<pre><code><b>const</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_MAINNET">RTD_MAINNET</a>: u8 = 0;
</code></pre>



<a name="bridge_chain_ids_RTD_TESTNET"></a>



<pre><code><b>const</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_TESTNET">RTD_TESTNET</a>: u8 = 1;
</code></pre>



<a name="bridge_chain_ids_RTD_CUSTOM"></a>



<pre><code><b>const</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_CUSTOM">RTD_CUSTOM</a>: u8 = 2;
</code></pre>



<a name="bridge_chain_ids_ETH_MAINNET"></a>



<pre><code><b>const</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_MAINNET">ETH_MAINNET</a>: u8 = 10;
</code></pre>



<a name="bridge_chain_ids_ETH_SEPOLIA"></a>



<pre><code><b>const</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_SEPOLIA">ETH_SEPOLIA</a>: u8 = 11;
</code></pre>



<a name="bridge_chain_ids_ETH_CUSTOM"></a>



<pre><code><b>const</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_CUSTOM">ETH_CUSTOM</a>: u8 = 12;
</code></pre>



<a name="bridge_chain_ids_EInvalidBridgeRoute"></a>



<pre><code><b>const</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_EInvalidBridgeRoute">EInvalidBridgeRoute</a>: u64 = 0;
</code></pre>



<a name="bridge_chain_ids_rtd_mainnet"></a>

## Function `rtd_mainnet`



<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_rtd_mainnet">rtd_mainnet</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_rtd_mainnet">rtd_mainnet</a>(): u8 { <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_MAINNET">RTD_MAINNET</a> }
</code></pre>



</details>

<a name="bridge_chain_ids_rtd_testnet"></a>

## Function `rtd_testnet`



<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_rtd_testnet">rtd_testnet</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_rtd_testnet">rtd_testnet</a>(): u8 { <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_TESTNET">RTD_TESTNET</a> }
</code></pre>



</details>

<a name="bridge_chain_ids_rtd_custom"></a>

## Function `rtd_custom`



<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_rtd_custom">rtd_custom</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_rtd_custom">rtd_custom</a>(): u8 { <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_CUSTOM">RTD_CUSTOM</a> }
</code></pre>



</details>

<a name="bridge_chain_ids_eth_mainnet"></a>

## Function `eth_mainnet`



<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_eth_mainnet">eth_mainnet</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_eth_mainnet">eth_mainnet</a>(): u8 { <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_MAINNET">ETH_MAINNET</a> }
</code></pre>



</details>

<a name="bridge_chain_ids_eth_sepolia"></a>

## Function `eth_sepolia`



<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_eth_sepolia">eth_sepolia</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_eth_sepolia">eth_sepolia</a>(): u8 { <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_SEPOLIA">ETH_SEPOLIA</a> }
</code></pre>



</details>

<a name="bridge_chain_ids_eth_custom"></a>

## Function `eth_custom`



<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_eth_custom">eth_custom</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_eth_custom">eth_custom</a>(): u8 { <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_CUSTOM">ETH_CUSTOM</a> }
</code></pre>



</details>

<a name="bridge_chain_ids_route_source"></a>

## Function `route_source`



<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_route_source">route_source</a>(route: &<a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">bridge::chain_ids::BridgeRoute</a>): &u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_route_source">route_source</a>(route: &<a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a>): &u8 {
    &route.source
}
</code></pre>



</details>

<a name="bridge_chain_ids_route_destination"></a>

## Function `route_destination`



<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_route_destination">route_destination</a>(route: &<a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">bridge::chain_ids::BridgeRoute</a>): &u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_route_destination">route_destination</a>(route: &<a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a>): &u8 {
    &route.destination
}
</code></pre>



</details>

<a name="bridge_chain_ids_assert_valid_chain_id"></a>

## Function `assert_valid_chain_id`



<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_assert_valid_chain_id">assert_valid_chain_id</a>(id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_assert_valid_chain_id">assert_valid_chain_id</a>(id: u8) {
    <b>assert</b>!(
        id == <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_MAINNET">RTD_MAINNET</a> ||
        id == <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_TESTNET">RTD_TESTNET</a> ||
        id == <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_CUSTOM">RTD_CUSTOM</a> ||
        id == <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_MAINNET">ETH_MAINNET</a> ||
        id == <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_SEPOLIA">ETH_SEPOLIA</a> ||
        id == <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_CUSTOM">ETH_CUSTOM</a>,
        <a href="../bridge/chain_ids.md#bridge_chain_ids_EInvalidBridgeRoute">EInvalidBridgeRoute</a>,
    )
}
</code></pre>



</details>

<a name="bridge_chain_ids_valid_routes"></a>

## Function `valid_routes`



<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_valid_routes">valid_routes</a>(): vector&lt;<a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">bridge::chain_ids::BridgeRoute</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_valid_routes">valid_routes</a>(): vector&lt;<a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a>&gt; {
    vector[
        <a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a> { source: <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_MAINNET">RTD_MAINNET</a>, destination: <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_MAINNET">ETH_MAINNET</a> },
        <a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a> { source: <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_MAINNET">ETH_MAINNET</a>, destination: <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_MAINNET">RTD_MAINNET</a> },
        <a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a> { source: <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_TESTNET">RTD_TESTNET</a>, destination: <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_SEPOLIA">ETH_SEPOLIA</a> },
        <a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a> { source: <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_TESTNET">RTD_TESTNET</a>, destination: <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_CUSTOM">ETH_CUSTOM</a> },
        <a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a> { source: <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_CUSTOM">RTD_CUSTOM</a>, destination: <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_CUSTOM">ETH_CUSTOM</a> },
        <a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a> { source: <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_CUSTOM">RTD_CUSTOM</a>, destination: <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_SEPOLIA">ETH_SEPOLIA</a> },
        <a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a> { source: <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_SEPOLIA">ETH_SEPOLIA</a>, destination: <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_TESTNET">RTD_TESTNET</a> },
        <a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a> { source: <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_SEPOLIA">ETH_SEPOLIA</a>, destination: <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_CUSTOM">RTD_CUSTOM</a> },
        <a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a> { source: <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_CUSTOM">ETH_CUSTOM</a>, destination: <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_TESTNET">RTD_TESTNET</a> },
        <a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a> { source: <a href="../bridge/chain_ids.md#bridge_chain_ids_ETH_CUSTOM">ETH_CUSTOM</a>, destination: <a href="../bridge/chain_ids.md#bridge_chain_ids_RTD_CUSTOM">RTD_CUSTOM</a> },
    ]
}
</code></pre>



</details>

<a name="bridge_chain_ids_is_valid_route"></a>

## Function `is_valid_route`



<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_is_valid_route">is_valid_route</a>(source: u8, destination: u8): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_is_valid_route">is_valid_route</a>(source: u8, destination: u8): bool {
    <b>let</b> route = <a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a> { source, destination };
    <a href="../bridge/chain_ids.md#bridge_chain_ids_valid_routes">valid_routes</a>().contains(&route)
}
</code></pre>



</details>

<a name="bridge_chain_ids_get_route"></a>

## Function `get_route`



<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_get_route">get_route</a>(source: u8, destination: u8): <a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">bridge::chain_ids::BridgeRoute</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../bridge/chain_ids.md#bridge_chain_ids_get_route">get_route</a>(source: u8, destination: u8): <a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a> {
    <b>let</b> route = <a href="../bridge/chain_ids.md#bridge_chain_ids_BridgeRoute">BridgeRoute</a> { source, destination };
    <b>assert</b>!(<a href="../bridge/chain_ids.md#bridge_chain_ids_valid_routes">valid_routes</a>().contains(&route), <a href="../bridge/chain_ids.md#bridge_chain_ids_EInvalidBridgeRoute">EInvalidBridgeRoute</a>);
    route
}
</code></pre>



</details>
