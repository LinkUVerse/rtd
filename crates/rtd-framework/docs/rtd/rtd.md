---
title: Module `rtd::rtd`
---

Coin<RTD> is the token used to pay for gas in Rtd.
It has 9 decimals, and the smallest unit (10^-9) is called "mist".


-  [Struct `RTD`](#rtd_rtd_RTD)
-  [Constants](#@Constants_0)
-  [Function `new`](#rtd_rtd_new)
-  [Function `transfer`](#rtd_rtd_transfer)


<pre><code><b>use</b> <a href="../rtd/accumulator.md#rtd_accumulator">rtd::accumulator</a>;
<b>use</b> <a href="../rtd/accumulator_metadata.md#rtd_accumulator_metadata">rtd::accumulator_metadata</a>;
<b>use</b> <a href="../rtd/accumulator_settlement.md#rtd_accumulator_settlement">rtd::accumulator_settlement</a>;
<b>use</b> <a href="../rtd/address.md#rtd_address">rtd::address</a>;
<b>use</b> <a href="../rtd/bag.md#rtd_bag">rtd::bag</a>;
<b>use</b> <a href="../rtd/balance.md#rtd_balance">rtd::balance</a>;
<b>use</b> <a href="../rtd/bcs.md#rtd_bcs">rtd::bcs</a>;
<b>use</b> <a href="../rtd/coin.md#rtd_coin">rtd::coin</a>;
<b>use</b> <a href="../rtd/config.md#rtd_config">rtd::config</a>;
<b>use</b> <a href="../rtd/deny_list.md#rtd_deny_list">rtd::deny_list</a>;
<b>use</b> <a href="../rtd/dynamic_field.md#rtd_dynamic_field">rtd::dynamic_field</a>;
<b>use</b> <a href="../rtd/dynamic_object_field.md#rtd_dynamic_object_field">rtd::dynamic_object_field</a>;
<b>use</b> <a href="../rtd/event.md#rtd_event">rtd::event</a>;
<b>use</b> <a href="../rtd/funds_accumulator.md#rtd_funds_accumulator">rtd::funds_accumulator</a>;
<b>use</b> <a href="../rtd/hash.md#rtd_hash">rtd::hash</a>;
<b>use</b> <a href="../rtd/hex.md#rtd_hex">rtd::hex</a>;
<b>use</b> <a href="../rtd/object.md#rtd_object">rtd::object</a>;
<b>use</b> <a href="../rtd/party.md#rtd_party">rtd::party</a>;
<b>use</b> <a href="../rtd/protocol_config.md#rtd_protocol_config">rtd::protocol_config</a>;
<b>use</b> <a href="../rtd/table.md#rtd_table">rtd::table</a>;
<b>use</b> <a href="../rtd/transfer.md#rtd_transfer">rtd::transfer</a>;
<b>use</b> <a href="../rtd/tx_context.md#rtd_tx_context">rtd::tx_context</a>;
<b>use</b> <a href="../rtd/types.md#rtd_types">rtd::types</a>;
<b>use</b> <a href="../rtd/url.md#rtd_url">rtd::url</a>;
<b>use</b> <a href="../rtd/vec_map.md#rtd_vec_map">rtd::vec_map</a>;
<b>use</b> <a href="../rtd/vec_set.md#rtd_vec_set">rtd::vec_set</a>;
<b>use</b> <a href="../std/address.md#std_address">std::address</a>;
<b>use</b> <a href="../std/ascii.md#std_ascii">std::ascii</a>;
<b>use</b> <a href="../std/bcs.md#std_bcs">std::bcs</a>;
<b>use</b> <a href="../std/internal.md#std_internal">std::internal</a>;
<b>use</b> <a href="../std/option.md#std_option">std::option</a>;
<b>use</b> <a href="../std/string.md#std_string">std::string</a>;
<b>use</b> <a href="../std/type_name.md#std_type_name">std::type_name</a>;
<b>use</b> <a href="../std/vector.md#std_vector">std::vector</a>;
</code></pre>



<a name="rtd_rtd_RTD"></a>

## Struct `RTD`

Name of the coin


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/rtd.md#rtd_rtd_RTD">RTD</a> <b>has</b> drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="rtd_rtd_EAlreadyMinted"></a>



<pre><code><b>const</b> <a href="../rtd/rtd.md#rtd_rtd_EAlreadyMinted">EAlreadyMinted</a>: u64 = 0;
</code></pre>



<a name="rtd_rtd_ENotSystemAddress"></a>

Sender is not @0x0 the system address.


<pre><code><b>const</b> <a href="../rtd/rtd.md#rtd_rtd_ENotSystemAddress">ENotSystemAddress</a>: u64 = 1;
</code></pre>



<a name="rtd_rtd_MIST_PER_RTD"></a>

The amount of Mist per Rtd token based on the fact that mist is
10^-9 of a Rtd token


<pre><code><b>const</b> <a href="../rtd/rtd.md#rtd_rtd_MIST_PER_RTD">MIST_PER_RTD</a>: u64 = 1000000000;
</code></pre>



<a name="rtd_rtd_TOTAL_SUPPLY_RTD"></a>

The total supply of Rtd denominated in whole Rtd tokens (10 Billion)


<pre><code><b>const</b> <a href="../rtd/rtd.md#rtd_rtd_TOTAL_SUPPLY_RTD">TOTAL_SUPPLY_RTD</a>: u64 = 10000000000;
</code></pre>



<a name="rtd_rtd_TOTAL_SUPPLY_MIST"></a>

The total supply of Rtd denominated in Mist (10 Billion * 10^9)


<pre><code><b>const</b> <a href="../rtd/rtd.md#rtd_rtd_TOTAL_SUPPLY_MIST">TOTAL_SUPPLY_MIST</a>: u64 = 10000000000000000000;
</code></pre>



<a name="rtd_rtd_new"></a>

## Function `new`

Register the <code><a href="../rtd/rtd.md#rtd_rtd_RTD">RTD</a></code> Coin to acquire its <code>Supply</code>.
This should be called only once during genesis creation.


<pre><code><b>fun</b> <a href="../rtd/rtd.md#rtd_rtd_new">new</a>(ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;<a href="../rtd/rtd.md#rtd_rtd_RTD">rtd::rtd::RTD</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../rtd/rtd.md#rtd_rtd_new">new</a>(ctx: &<b>mut</b> TxContext): Balance&lt;<a href="../rtd/rtd.md#rtd_rtd_RTD">RTD</a>&gt; {
    <b>assert</b>!(ctx.sender() == @0x0, <a href="../rtd/rtd.md#rtd_rtd_ENotSystemAddress">ENotSystemAddress</a>);
    <b>assert</b>!(ctx.epoch() == 0, <a href="../rtd/rtd.md#rtd_rtd_EAlreadyMinted">EAlreadyMinted</a>);
    <b>let</b> (treasury, metadata) = <a href="../rtd/coin.md#rtd_coin_create_currency">coin::create_currency</a>(
        <a href="../rtd/rtd.md#rtd_rtd_RTD">RTD</a> {},
        9,
        b"<a href="../rtd/rtd.md#rtd_rtd_RTD">RTD</a>",
        b"Rtd",
        // TODO: add appropriate description and logo <a href="../rtd/url.md#rtd_url">url</a>
        b"",
        option::none(),
        ctx,
    );
    <a href="../rtd/transfer.md#rtd_transfer_public_freeze_object">transfer::public_freeze_object</a>(metadata);
    <b>let</b> <b>mut</b> supply = treasury.treasury_into_supply();
    <b>let</b> total_rtd = supply.increase_supply(<a href="../rtd/rtd.md#rtd_rtd_TOTAL_SUPPLY_MIST">TOTAL_SUPPLY_MIST</a>);
    supply.destroy_supply();
    total_rtd
}
</code></pre>



</details>

<a name="rtd_rtd_transfer"></a>

## Function `transfer`



<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../rtd/transfer.md#rtd_transfer">transfer</a>(c: <a href="../rtd/coin.md#rtd_coin_Coin">rtd::coin::Coin</a>&lt;<a href="../rtd/rtd.md#rtd_rtd_RTD">rtd::rtd::RTD</a>&gt;, recipient: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../rtd/transfer.md#rtd_transfer">transfer</a>(c: <a href="../rtd/coin.md#rtd_coin_Coin">coin::Coin</a>&lt;<a href="../rtd/rtd.md#rtd_rtd_RTD">RTD</a>&gt;, recipient: <b>address</b>) {
    <a href="../rtd/transfer.md#rtd_transfer_public_transfer">transfer::public_transfer</a>(c, recipient)
}
</code></pre>



</details>
