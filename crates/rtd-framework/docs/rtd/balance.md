---
title: Module `rtd::balance`
---

A storable handler for Balances in general. Is used in the <code>Coin</code>
module to allow balance operations and can be used to implement
custom coins with <code><a href="../rtd/balance.md#rtd_balance_Supply">Supply</a></code> and <code><a href="../rtd/balance.md#rtd_balance_Balance">Balance</a></code>s.


-  [Struct `Supply`](#rtd_balance_Supply)
-  [Struct `Balance`](#rtd_balance_Balance)
-  [Constants](#@Constants_0)
-  [Function `value`](#rtd_balance_value)
-  [Function `supply_value`](#rtd_balance_supply_value)
-  [Function `create_supply`](#rtd_balance_create_supply)
-  [Function `increase_supply`](#rtd_balance_increase_supply)
-  [Function `decrease_supply`](#rtd_balance_decrease_supply)
-  [Function `zero`](#rtd_balance_zero)
-  [Function `join`](#rtd_balance_join)
-  [Function `split`](#rtd_balance_split)
-  [Function `withdraw_all`](#rtd_balance_withdraw_all)
-  [Function `destroy_zero`](#rtd_balance_destroy_zero)
-  [Function `send_funds`](#rtd_balance_send_funds)
-  [Function `redeem_funds`](#rtd_balance_redeem_funds)
-  [Function `withdraw_funds_from_object`](#rtd_balance_withdraw_funds_from_object)
-  [Function `create_supply_internal`](#rtd_balance_create_supply_internal)
-  [Function `create_staking_rewards`](#rtd_balance_create_staking_rewards)
-  [Function `destroy_storage_rebates`](#rtd_balance_destroy_storage_rebates)
-  [Function `destroy_supply`](#rtd_balance_destroy_supply)


<pre><code><b>use</b> <a href="../rtd/accumulator.md#rtd_accumulator">rtd::accumulator</a>;
<b>use</b> <a href="../rtd/address.md#rtd_address">rtd::address</a>;
<b>use</b> <a href="../rtd/dynamic_field.md#rtd_dynamic_field">rtd::dynamic_field</a>;
<b>use</b> <a href="../rtd/funds_accumulator.md#rtd_funds_accumulator">rtd::funds_accumulator</a>;
<b>use</b> <a href="../rtd/hex.md#rtd_hex">rtd::hex</a>;
<b>use</b> <a href="../rtd/object.md#rtd_object">rtd::object</a>;
<b>use</b> <a href="../rtd/party.md#rtd_party">rtd::party</a>;
<b>use</b> <a href="../rtd/protocol_config.md#rtd_protocol_config">rtd::protocol_config</a>;
<b>use</b> <a href="../rtd/transfer.md#rtd_transfer">rtd::transfer</a>;
<b>use</b> <a href="../rtd/tx_context.md#rtd_tx_context">rtd::tx_context</a>;
<b>use</b> <a href="../rtd/vec_map.md#rtd_vec_map">rtd::vec_map</a>;
<b>use</b> <a href="../std/address.md#std_address">std::address</a>;
<b>use</b> <a href="../std/ascii.md#std_ascii">std::ascii</a>;
<b>use</b> <a href="../std/bcs.md#std_bcs">std::bcs</a>;
<b>use</b> <a href="../std/internal.md#std_internal">std::internal</a>;
<b>use</b> <a href="../std/option.md#std_option">std::option</a>;
<b>use</b> <a href="../std/string.md#std_string">std::string</a>;
<b>use</b> <a href="../std/type_name.md#std_type_name">std::type_name</a>;
<b>use</b> <a href="../std/vector.md#std_vector">std::vector</a>;
</code></pre>



<a name="rtd_balance_Supply"></a>

## Struct `Supply`

A Supply of T. Used for minting and burning.
Wrapped into a <code>TreasuryCap</code> in the <code>Coin</code> module.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/balance.md#rtd_balance_Supply">Supply</a>&lt;<b>phantom</b> T&gt; <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="../rtd/balance.md#rtd_balance_value">value</a>: u64</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="rtd_balance_Balance"></a>

## Struct `Balance`

Storable balance - an inner struct of a Coin type.
Can be used to store coins which don't need the key ability.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;<b>phantom</b> T&gt; <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="../rtd/balance.md#rtd_balance_value">value</a>: u64</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="rtd_balance_ENonZero"></a>

For when trying to destroy a non-zero balance.


<pre><code><b>const</b> <a href="../rtd/balance.md#rtd_balance_ENonZero">ENonZero</a>: u64 = 0;
</code></pre>



<a name="rtd_balance_EOverflow"></a>

For when an overflow is happening on Supply operations.


<pre><code><b>const</b> <a href="../rtd/balance.md#rtd_balance_EOverflow">EOverflow</a>: u64 = 1;
</code></pre>



<a name="rtd_balance_ENotEnough"></a>

For when trying to withdraw more than there is.


<pre><code><b>const</b> <a href="../rtd/balance.md#rtd_balance_ENotEnough">ENotEnough</a>: u64 = 2;
</code></pre>



<a name="rtd_balance_ENotSystemAddress"></a>

Sender is not @0x0 the system address.


<pre><code><b>const</b> <a href="../rtd/balance.md#rtd_balance_ENotSystemAddress">ENotSystemAddress</a>: u64 = 3;
</code></pre>



<a name="rtd_balance_ENotRTD"></a>

System operation performed for a coin other than RTD


<pre><code><b>const</b> <a href="../rtd/balance.md#rtd_balance_ENotRTD">ENotRTD</a>: u64 = 4;
</code></pre>



<a name="rtd_balance_RTD_TYPE_NAME"></a>



<pre><code><b>const</b> <a href="../rtd/balance.md#rtd_balance_RTD_TYPE_NAME">RTD_TYPE_NAME</a>: vector&lt;u8&gt; = vector[48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 50, 58, 58, 114, 116, 100, 58, 58, 82, 84, 68];
</code></pre>



<a name="rtd_balance_value"></a>

## Function `value`

Get the amount stored in a <code><a href="../rtd/balance.md#rtd_balance_Balance">Balance</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_value">value</a>&lt;T&gt;(self: &<a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_value">value</a>&lt;T&gt;(self: &<a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt;): u64 {
    self.<a href="../rtd/balance.md#rtd_balance_value">value</a>
}
</code></pre>



</details>

<a name="rtd_balance_supply_value"></a>

## Function `supply_value`

Get the <code><a href="../rtd/balance.md#rtd_balance_Supply">Supply</a></code> value.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_supply_value">supply_value</a>&lt;T&gt;(supply: &<a href="../rtd/balance.md#rtd_balance_Supply">rtd::balance::Supply</a>&lt;T&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_supply_value">supply_value</a>&lt;T&gt;(supply: &<a href="../rtd/balance.md#rtd_balance_Supply">Supply</a>&lt;T&gt;): u64 {
    supply.<a href="../rtd/balance.md#rtd_balance_value">value</a>
}
</code></pre>



</details>

<a name="rtd_balance_create_supply"></a>

## Function `create_supply`

Create a new supply for type T.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_create_supply">create_supply</a>&lt;T: drop&gt;(_: T): <a href="../rtd/balance.md#rtd_balance_Supply">rtd::balance::Supply</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_create_supply">create_supply</a>&lt;T: drop&gt;(_: T): <a href="../rtd/balance.md#rtd_balance_Supply">Supply</a>&lt;T&gt; {
    <a href="../rtd/balance.md#rtd_balance_Supply">Supply</a> { <a href="../rtd/balance.md#rtd_balance_value">value</a>: 0 }
}
</code></pre>



</details>

<a name="rtd_balance_increase_supply"></a>

## Function `increase_supply`

Increase supply by <code><a href="../rtd/balance.md#rtd_balance_value">value</a></code> and create a new <code><a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt;</code> with this value.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_increase_supply">increase_supply</a>&lt;T&gt;(self: &<b>mut</b> <a href="../rtd/balance.md#rtd_balance_Supply">rtd::balance::Supply</a>&lt;T&gt;, <a href="../rtd/balance.md#rtd_balance_value">value</a>: u64): <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_increase_supply">increase_supply</a>&lt;T&gt;(self: &<b>mut</b> <a href="../rtd/balance.md#rtd_balance_Supply">Supply</a>&lt;T&gt;, <a href="../rtd/balance.md#rtd_balance_value">value</a>: u64): <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt; {
    <b>assert</b>!(<a href="../rtd/balance.md#rtd_balance_value">value</a> &lt; (18446744073709551615u64 - self.<a href="../rtd/balance.md#rtd_balance_value">value</a>), <a href="../rtd/balance.md#rtd_balance_EOverflow">EOverflow</a>);
    self.<a href="../rtd/balance.md#rtd_balance_value">value</a> = self.<a href="../rtd/balance.md#rtd_balance_value">value</a> + <a href="../rtd/balance.md#rtd_balance_value">value</a>;
    <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a> { <a href="../rtd/balance.md#rtd_balance_value">value</a> }
}
</code></pre>



</details>

<a name="rtd_balance_decrease_supply"></a>

## Function `decrease_supply`

Burn a Balance<T> and decrease Supply<T>.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_decrease_supply">decrease_supply</a>&lt;T&gt;(self: &<b>mut</b> <a href="../rtd/balance.md#rtd_balance_Supply">rtd::balance::Supply</a>&lt;T&gt;, <a href="../rtd/balance.md#rtd_balance">balance</a>: <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_decrease_supply">decrease_supply</a>&lt;T&gt;(self: &<b>mut</b> <a href="../rtd/balance.md#rtd_balance_Supply">Supply</a>&lt;T&gt;, <a href="../rtd/balance.md#rtd_balance">balance</a>: <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt;): u64 {
    <b>let</b> <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a> { <a href="../rtd/balance.md#rtd_balance_value">value</a> } = <a href="../rtd/balance.md#rtd_balance">balance</a>;
    <b>assert</b>!(self.<a href="../rtd/balance.md#rtd_balance_value">value</a> &gt;= <a href="../rtd/balance.md#rtd_balance_value">value</a>, <a href="../rtd/balance.md#rtd_balance_EOverflow">EOverflow</a>);
    self.<a href="../rtd/balance.md#rtd_balance_value">value</a> = self.<a href="../rtd/balance.md#rtd_balance_value">value</a> - <a href="../rtd/balance.md#rtd_balance_value">value</a>;
    <a href="../rtd/balance.md#rtd_balance_value">value</a>
}
</code></pre>



</details>

<a name="rtd_balance_zero"></a>

## Function `zero`

Create a zero <code><a href="../rtd/balance.md#rtd_balance_Balance">Balance</a></code> for type <code>T</code>.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_zero">zero</a>&lt;T&gt;(): <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_zero">zero</a>&lt;T&gt;(): <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt; {
    <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a> { <a href="../rtd/balance.md#rtd_balance_value">value</a>: 0 }
}
</code></pre>



</details>

<a name="rtd_balance_join"></a>

## Function `join`

Join two balances together.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_join">join</a>&lt;T&gt;(self: &<b>mut</b> <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;, <a href="../rtd/balance.md#rtd_balance">balance</a>: <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_join">join</a>&lt;T&gt;(self: &<b>mut</b> <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt;, <a href="../rtd/balance.md#rtd_balance">balance</a>: <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt;): u64 {
    <b>let</b> <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a> { <a href="../rtd/balance.md#rtd_balance_value">value</a> } = <a href="../rtd/balance.md#rtd_balance">balance</a>;
    self.<a href="../rtd/balance.md#rtd_balance_value">value</a> = self.<a href="../rtd/balance.md#rtd_balance_value">value</a> + <a href="../rtd/balance.md#rtd_balance_value">value</a>;
    self.<a href="../rtd/balance.md#rtd_balance_value">value</a>
}
</code></pre>



</details>

<a name="rtd_balance_split"></a>

## Function `split`

Split a <code><a href="../rtd/balance.md#rtd_balance_Balance">Balance</a></code> and take a sub balance from it.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_split">split</a>&lt;T&gt;(self: &<b>mut</b> <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;, <a href="../rtd/balance.md#rtd_balance_value">value</a>: u64): <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_split">split</a>&lt;T&gt;(self: &<b>mut</b> <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt;, <a href="../rtd/balance.md#rtd_balance_value">value</a>: u64): <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt; {
    <b>assert</b>!(self.<a href="../rtd/balance.md#rtd_balance_value">value</a> &gt;= <a href="../rtd/balance.md#rtd_balance_value">value</a>, <a href="../rtd/balance.md#rtd_balance_ENotEnough">ENotEnough</a>);
    self.<a href="../rtd/balance.md#rtd_balance_value">value</a> = self.<a href="../rtd/balance.md#rtd_balance_value">value</a> - <a href="../rtd/balance.md#rtd_balance_value">value</a>;
    <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a> { <a href="../rtd/balance.md#rtd_balance_value">value</a> }
}
</code></pre>



</details>

<a name="rtd_balance_withdraw_all"></a>

## Function `withdraw_all`

Withdraw all balance. After this the remaining balance must be 0.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_withdraw_all">withdraw_all</a>&lt;T&gt;(self: &<b>mut</b> <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;): <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_withdraw_all">withdraw_all</a>&lt;T&gt;(self: &<b>mut</b> <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt;): <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt; {
    <b>let</b> <a href="../rtd/balance.md#rtd_balance_value">value</a> = self.<a href="../rtd/balance.md#rtd_balance_value">value</a>;
    <a href="../rtd/balance.md#rtd_balance_split">split</a>(self, <a href="../rtd/balance.md#rtd_balance_value">value</a>)
}
</code></pre>



</details>

<a name="rtd_balance_destroy_zero"></a>

## Function `destroy_zero`

Destroy a zero <code><a href="../rtd/balance.md#rtd_balance_Balance">Balance</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_destroy_zero">destroy_zero</a>&lt;T&gt;(<a href="../rtd/balance.md#rtd_balance">balance</a>: <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_destroy_zero">destroy_zero</a>&lt;T&gt;(<a href="../rtd/balance.md#rtd_balance">balance</a>: <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt;) {
    <b>assert</b>!(<a href="../rtd/balance.md#rtd_balance">balance</a>.<a href="../rtd/balance.md#rtd_balance_value">value</a> == 0, <a href="../rtd/balance.md#rtd_balance_ENonZero">ENonZero</a>);
    <b>let</b> <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a> { <a href="../rtd/balance.md#rtd_balance_value">value</a>: _ } = <a href="../rtd/balance.md#rtd_balance">balance</a>;
}
</code></pre>



</details>

<a name="rtd_balance_send_funds"></a>

## Function `send_funds`

Send a <code><a href="../rtd/balance.md#rtd_balance_Balance">Balance</a></code> to an address's funds accumulator.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_send_funds">send_funds</a>&lt;T&gt;(<a href="../rtd/balance.md#rtd_balance">balance</a>: <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;, recipient: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_send_funds">send_funds</a>&lt;T&gt;(<a href="../rtd/balance.md#rtd_balance">balance</a>: <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt;, recipient: <b>address</b>) {
    <a href="../rtd/funds_accumulator.md#rtd_funds_accumulator_add_impl">rtd::funds_accumulator::add_impl</a>(<a href="../rtd/balance.md#rtd_balance">balance</a>, recipient);
}
</code></pre>



</details>

<a name="rtd_balance_redeem_funds"></a>

## Function `redeem_funds`

Redeem a <code>Withdrawal&lt;<a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt;&gt;</code> to get the underlying <code><a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt;</code> from an address's funds
accumulator.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_redeem_funds">redeem_funds</a>&lt;T&gt;(withdrawal: <a href="../rtd/funds_accumulator.md#rtd_funds_accumulator_Withdrawal">rtd::funds_accumulator::Withdrawal</a>&lt;<a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;&gt;): <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_redeem_funds">redeem_funds</a>&lt;T&gt;(withdrawal: <a href="../rtd/funds_accumulator.md#rtd_funds_accumulator_Withdrawal">rtd::funds_accumulator::Withdrawal</a>&lt;<a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt;&gt;): <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt; {
    withdrawal.redeem(internal::permit())
}
</code></pre>



</details>

<a name="rtd_balance_withdraw_funds_from_object"></a>

## Function `withdraw_funds_from_object`

Create a <code>Withdrawal&lt;<a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt;&gt;</code> from an object to withdraw funds from it.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_withdraw_funds_from_object">withdraw_funds_from_object</a>&lt;T&gt;(obj: &<b>mut</b> <a href="../rtd/object.md#rtd_object_UID">rtd::object::UID</a>, <a href="../rtd/balance.md#rtd_balance_value">value</a>: u64): <a href="../rtd/funds_accumulator.md#rtd_funds_accumulator_Withdrawal">rtd::funds_accumulator::Withdrawal</a>&lt;<a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/balance.md#rtd_balance_withdraw_funds_from_object">withdraw_funds_from_object</a>&lt;T&gt;(obj: &<b>mut</b> UID, <a href="../rtd/balance.md#rtd_balance_value">value</a>: u64): Withdrawal&lt;<a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt;&gt; {
    <a href="../rtd/funds_accumulator.md#rtd_funds_accumulator_withdraw_from_object">rtd::funds_accumulator::withdraw_from_object</a>(obj, <a href="../rtd/balance.md#rtd_balance_value">value</a> <b>as</b> u256)
}
</code></pre>



</details>

<a name="rtd_balance_create_supply_internal"></a>

## Function `create_supply_internal`



<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/balance.md#rtd_balance_create_supply_internal">create_supply_internal</a>&lt;T&gt;(): <a href="../rtd/balance.md#rtd_balance_Supply">rtd::balance::Supply</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/balance.md#rtd_balance_create_supply_internal">create_supply_internal</a>&lt;T&gt;(): <a href="../rtd/balance.md#rtd_balance_Supply">Supply</a>&lt;T&gt; {
    <a href="../rtd/balance.md#rtd_balance_Supply">Supply</a> { <a href="../rtd/balance.md#rtd_balance_value">value</a>: 0 }
}
</code></pre>



</details>

<a name="rtd_balance_create_staking_rewards"></a>

## Function `create_staking_rewards`

CAUTION: this function creates a <code><a href="../rtd/balance.md#rtd_balance_Balance">Balance</a></code> without increasing the supply.
It should only be called by the epoch change system txn to create staking rewards,
and nowhere else.


<pre><code><b>fun</b> <a href="../rtd/balance.md#rtd_balance_create_staking_rewards">create_staking_rewards</a>&lt;T&gt;(<a href="../rtd/balance.md#rtd_balance_value">value</a>: u64, ctx: &<a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../rtd/balance.md#rtd_balance_create_staking_rewards">create_staking_rewards</a>&lt;T&gt;(<a href="../rtd/balance.md#rtd_balance_value">value</a>: u64, ctx: &TxContext): <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt; {
    <b>assert</b>!(ctx.sender() == @0x0, <a href="../rtd/balance.md#rtd_balance_ENotSystemAddress">ENotSystemAddress</a>);
    <b>assert</b>!(
        <a href="../std/type_name.md#std_type_name_with_defining_ids">std::type_name::with_defining_ids</a>&lt;T&gt;().into_string().into_bytes() == <a href="../rtd/balance.md#rtd_balance_RTD_TYPE_NAME">RTD_TYPE_NAME</a>,
        <a href="../rtd/balance.md#rtd_balance_ENotRTD">ENotRTD</a>,
    );
    <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a> { <a href="../rtd/balance.md#rtd_balance_value">value</a> }
}
</code></pre>



</details>

<a name="rtd_balance_destroy_storage_rebates"></a>

## Function `destroy_storage_rebates`

CAUTION: this function destroys a <code><a href="../rtd/balance.md#rtd_balance_Balance">Balance</a></code> without decreasing the supply.
It should only be called by the epoch change system txn to destroy storage rebates,
and nowhere else.


<pre><code><b>fun</b> <a href="../rtd/balance.md#rtd_balance_destroy_storage_rebates">destroy_storage_rebates</a>&lt;T&gt;(self: <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;, ctx: &<a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../rtd/balance.md#rtd_balance_destroy_storage_rebates">destroy_storage_rebates</a>&lt;T&gt;(self: <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a>&lt;T&gt;, ctx: &TxContext) {
    <b>assert</b>!(ctx.sender() == @0x0, <a href="../rtd/balance.md#rtd_balance_ENotSystemAddress">ENotSystemAddress</a>);
    <b>assert</b>!(
        <a href="../std/type_name.md#std_type_name_with_defining_ids">std::type_name::with_defining_ids</a>&lt;T&gt;().into_string().into_bytes() == <a href="../rtd/balance.md#rtd_balance_RTD_TYPE_NAME">RTD_TYPE_NAME</a>,
        <a href="../rtd/balance.md#rtd_balance_ENotRTD">ENotRTD</a>,
    );
    <b>let</b> <a href="../rtd/balance.md#rtd_balance_Balance">Balance</a> { <a href="../rtd/balance.md#rtd_balance_value">value</a>: _ } = self;
}
</code></pre>



</details>

<a name="rtd_balance_destroy_supply"></a>

## Function `destroy_supply`

Destroy a <code><a href="../rtd/balance.md#rtd_balance_Supply">Supply</a></code> preventing any further minting and burning.


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/balance.md#rtd_balance_destroy_supply">destroy_supply</a>&lt;T&gt;(self: <a href="../rtd/balance.md#rtd_balance_Supply">rtd::balance::Supply</a>&lt;T&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/balance.md#rtd_balance_destroy_supply">destroy_supply</a>&lt;T&gt;(self: <a href="../rtd/balance.md#rtd_balance_Supply">Supply</a>&lt;T&gt;): u64 {
    <b>let</b> <a href="../rtd/balance.md#rtd_balance_Supply">Supply</a> { <a href="../rtd/balance.md#rtd_balance_value">value</a> } = self;
    <a href="../rtd/balance.md#rtd_balance_value">value</a>
}
</code></pre>



</details>
