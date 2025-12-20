---
title: Module `rtd::accumulator`
---



-  [Struct `AccumulatorRoot`](#rtd_accumulator_AccumulatorRoot)
-  [Struct `U128`](#rtd_accumulator_U128)
-  [Struct `Key`](#rtd_accumulator_Key)
-  [Constants](#@Constants_0)
-  [Function `create`](#rtd_accumulator_create)
-  [Function `root_id`](#rtd_accumulator_root_id)
-  [Function `root_id_mut`](#rtd_accumulator_root_id_mut)
-  [Function `create_u128`](#rtd_accumulator_create_u128)
-  [Function `destroy_u128`](#rtd_accumulator_destroy_u128)
-  [Function `update_u128`](#rtd_accumulator_update_u128)
-  [Function `is_zero_u128`](#rtd_accumulator_is_zero_u128)
-  [Function `accumulator_key`](#rtd_accumulator_accumulator_key)
-  [Function `accumulator_address`](#rtd_accumulator_accumulator_address)
-  [Function `root_has_accumulator`](#rtd_accumulator_root_has_accumulator)
-  [Function `root_add_accumulator`](#rtd_accumulator_root_add_accumulator)
-  [Function `root_borrow_accumulator_mut`](#rtd_accumulator_root_borrow_accumulator_mut)
-  [Function `root_remove_accumulator`](#rtd_accumulator_root_remove_accumulator)
-  [Function `emit_deposit_event`](#rtd_accumulator_emit_deposit_event)
-  [Function `emit_withdraw_event`](#rtd_accumulator_emit_withdraw_event)


<pre><code><b>use</b> <a href="../rtd/address.md#rtd_address">rtd::address</a>;
<b>use</b> <a href="../rtd/dynamic_field.md#rtd_dynamic_field">rtd::dynamic_field</a>;
<b>use</b> <a href="../rtd/hex.md#rtd_hex">rtd::hex</a>;
<b>use</b> <a href="../rtd/object.md#rtd_object">rtd::object</a>;
<b>use</b> <a href="../rtd/party.md#rtd_party">rtd::party</a>;
<b>use</b> <a href="../rtd/transfer.md#rtd_transfer">rtd::transfer</a>;
<b>use</b> <a href="../rtd/tx_context.md#rtd_tx_context">rtd::tx_context</a>;
<b>use</b> <a href="../rtd/vec_map.md#rtd_vec_map">rtd::vec_map</a>;
<b>use</b> <a href="../std/ascii.md#std_ascii">std::ascii</a>;
<b>use</b> <a href="../std/bcs.md#std_bcs">std::bcs</a>;
<b>use</b> <a href="../std/option.md#std_option">std::option</a>;
<b>use</b> <a href="../std/string.md#std_string">std::string</a>;
<b>use</b> <a href="../std/vector.md#std_vector">std::vector</a>;
</code></pre>



<a name="rtd_accumulator_AccumulatorRoot"></a>

## Struct `AccumulatorRoot`



<pre><code><b>public</b> <b>struct</b> <a href="../rtd/accumulator.md#rtd_accumulator_AccumulatorRoot">AccumulatorRoot</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="../rtd/object.md#rtd_object_UID">rtd::object::UID</a></code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="rtd_accumulator_U128"></a>

## Struct `U128`

Storage for 128-bit accumulator values.

Currently only used to represent the sum of 64 bit values (such as <code>Balance&lt;T&gt;</code>).
The additional bits are necessary to prevent overflow, as it would take 2^64 deposits of U64_MAX
to cause an overflow.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/accumulator.md#rtd_accumulator_U128">U128</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>value: u128</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="rtd_accumulator_Key"></a>

## Struct `Key`

<code><a href="../rtd/accumulator.md#rtd_accumulator_Key">Key</a></code> is used only for computing the field id of accumulator objects.
<code>T</code> is the type of the accumulated value, e.g. <code>Balance&lt;RTD&gt;</code>


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/accumulator.md#rtd_accumulator_Key">Key</a>&lt;<b>phantom</b> T&gt; <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><b>address</b>: <b>address</b></code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="rtd_accumulator_ENotSystemAddress"></a>



<pre><code><b>const</b> <a href="../rtd/accumulator.md#rtd_accumulator_ENotSystemAddress">ENotSystemAddress</a>: u64 = 0;
</code></pre>



<a name="rtd_accumulator_create"></a>

## Function `create`



<pre><code><b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_create">create</a>(ctx: &<a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_create">create</a>(ctx: &TxContext) {
    <b>assert</b>!(ctx.sender() == @0x0, <a href="../rtd/accumulator.md#rtd_accumulator_ENotSystemAddress">ENotSystemAddress</a>);
    <a href="../rtd/transfer.md#rtd_transfer_share_object">transfer::share_object</a>(<a href="../rtd/accumulator.md#rtd_accumulator_AccumulatorRoot">AccumulatorRoot</a> {
        id: <a href="../rtd/object.md#rtd_object_rtd_accumulator_root_object_id">object::rtd_accumulator_root_object_id</a>(),
    })
}
</code></pre>



</details>

<a name="rtd_accumulator_root_id"></a>

## Function `root_id`



<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_root_id">root_id</a>(accumulator_root: &<a href="../rtd/accumulator.md#rtd_accumulator_AccumulatorRoot">rtd::accumulator::AccumulatorRoot</a>): &<a href="../rtd/object.md#rtd_object_UID">rtd::object::UID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_root_id">root_id</a>(accumulator_root: &<a href="../rtd/accumulator.md#rtd_accumulator_AccumulatorRoot">AccumulatorRoot</a>): &UID {
    &accumulator_root.id
}
</code></pre>



</details>

<a name="rtd_accumulator_root_id_mut"></a>

## Function `root_id_mut`



<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_root_id_mut">root_id_mut</a>(accumulator_root: &<b>mut</b> <a href="../rtd/accumulator.md#rtd_accumulator_AccumulatorRoot">rtd::accumulator::AccumulatorRoot</a>): &<b>mut</b> <a href="../rtd/object.md#rtd_object_UID">rtd::object::UID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_root_id_mut">root_id_mut</a>(accumulator_root: &<b>mut</b> <a href="../rtd/accumulator.md#rtd_accumulator_AccumulatorRoot">AccumulatorRoot</a>): &<b>mut</b> UID {
    &<b>mut</b> accumulator_root.id
}
</code></pre>



</details>

<a name="rtd_accumulator_create_u128"></a>

## Function `create_u128`



<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_create_u128">create_u128</a>(value: u128): <a href="../rtd/accumulator.md#rtd_accumulator_U128">rtd::accumulator::U128</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_create_u128">create_u128</a>(value: u128): <a href="../rtd/accumulator.md#rtd_accumulator_U128">U128</a> {
    <a href="../rtd/accumulator.md#rtd_accumulator_U128">U128</a> { value }
}
</code></pre>



</details>

<a name="rtd_accumulator_destroy_u128"></a>

## Function `destroy_u128`



<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_destroy_u128">destroy_u128</a>(u128: <a href="../rtd/accumulator.md#rtd_accumulator_U128">rtd::accumulator::U128</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_destroy_u128">destroy_u128</a>(u128: <a href="../rtd/accumulator.md#rtd_accumulator_U128">U128</a>) {
    <b>let</b> <a href="../rtd/accumulator.md#rtd_accumulator_U128">U128</a> { value: _ } = u128;
}
</code></pre>



</details>

<a name="rtd_accumulator_update_u128"></a>

## Function `update_u128`



<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_update_u128">update_u128</a>(u128: &<b>mut</b> <a href="../rtd/accumulator.md#rtd_accumulator_U128">rtd::accumulator::U128</a>, merge: u128, split: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_update_u128">update_u128</a>(u128: &<b>mut</b> <a href="../rtd/accumulator.md#rtd_accumulator_U128">U128</a>, merge: u128, split: u128) {
    u128.value = u128.value + merge - split;
}
</code></pre>



</details>

<a name="rtd_accumulator_is_zero_u128"></a>

## Function `is_zero_u128`



<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_is_zero_u128">is_zero_u128</a>(u128: &<a href="../rtd/accumulator.md#rtd_accumulator_U128">rtd::accumulator::U128</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_is_zero_u128">is_zero_u128</a>(u128: &<a href="../rtd/accumulator.md#rtd_accumulator_U128">U128</a>): bool {
    u128.value == 0
}
</code></pre>



</details>

<a name="rtd_accumulator_accumulator_key"></a>

## Function `accumulator_key`



<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_accumulator_key">accumulator_key</a>&lt;T&gt;(<b>address</b>: <b>address</b>): <a href="../rtd/accumulator.md#rtd_accumulator_Key">rtd::accumulator::Key</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_accumulator_key">accumulator_key</a>&lt;T&gt;(<b>address</b>: <b>address</b>): <a href="../rtd/accumulator.md#rtd_accumulator_Key">Key</a>&lt;T&gt; {
    <a href="../rtd/accumulator.md#rtd_accumulator_Key">Key</a> { <b>address</b> }
}
</code></pre>



</details>

<a name="rtd_accumulator_accumulator_address"></a>

## Function `accumulator_address`



<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_accumulator_address">accumulator_address</a>&lt;T&gt;(<b>address</b>: <b>address</b>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_accumulator_address">accumulator_address</a>&lt;T&gt;(<b>address</b>: <b>address</b>): <b>address</b> {
    <b>let</b> key = <a href="../rtd/accumulator.md#rtd_accumulator_Key">Key</a>&lt;T&gt; { <b>address</b> };
    <a href="../rtd/dynamic_field.md#rtd_dynamic_field_hash_type_and_key">dynamic_field::hash_type_and_key</a>(rtd_accumulator_root_address(), key)
}
</code></pre>



</details>

<a name="rtd_accumulator_root_has_accumulator"></a>

## Function `root_has_accumulator`

Balance object methods


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_root_has_accumulator">root_has_accumulator</a>&lt;K, V: store&gt;(accumulator_root: &<a href="../rtd/accumulator.md#rtd_accumulator_AccumulatorRoot">rtd::accumulator::AccumulatorRoot</a>, name: <a href="../rtd/accumulator.md#rtd_accumulator_Key">rtd::accumulator::Key</a>&lt;K&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_root_has_accumulator">root_has_accumulator</a>&lt;K, V: store&gt;(
    accumulator_root: &<a href="../rtd/accumulator.md#rtd_accumulator_AccumulatorRoot">AccumulatorRoot</a>,
    name: <a href="../rtd/accumulator.md#rtd_accumulator_Key">Key</a>&lt;K&gt;,
): bool {
    <a href="../rtd/dynamic_field.md#rtd_dynamic_field_exists_with_type">dynamic_field::exists_with_type</a>&lt;<a href="../rtd/accumulator.md#rtd_accumulator_Key">Key</a>&lt;K&gt;, V&gt;(&accumulator_root.id, name)
}
</code></pre>



</details>

<a name="rtd_accumulator_root_add_accumulator"></a>

## Function `root_add_accumulator`



<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_root_add_accumulator">root_add_accumulator</a>&lt;K, V: store&gt;(accumulator_root: &<b>mut</b> <a href="../rtd/accumulator.md#rtd_accumulator_AccumulatorRoot">rtd::accumulator::AccumulatorRoot</a>, name: <a href="../rtd/accumulator.md#rtd_accumulator_Key">rtd::accumulator::Key</a>&lt;K&gt;, value: V)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_root_add_accumulator">root_add_accumulator</a>&lt;K, V: store&gt;(
    accumulator_root: &<b>mut</b> <a href="../rtd/accumulator.md#rtd_accumulator_AccumulatorRoot">AccumulatorRoot</a>,
    name: <a href="../rtd/accumulator.md#rtd_accumulator_Key">Key</a>&lt;K&gt;,
    value: V,
) {
    <a href="../rtd/dynamic_field.md#rtd_dynamic_field_add">dynamic_field::add</a>(&<b>mut</b> accumulator_root.id, name, value);
}
</code></pre>



</details>

<a name="rtd_accumulator_root_borrow_accumulator_mut"></a>

## Function `root_borrow_accumulator_mut`



<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_root_borrow_accumulator_mut">root_borrow_accumulator_mut</a>&lt;K, V: store&gt;(accumulator_root: &<b>mut</b> <a href="../rtd/accumulator.md#rtd_accumulator_AccumulatorRoot">rtd::accumulator::AccumulatorRoot</a>, name: <a href="../rtd/accumulator.md#rtd_accumulator_Key">rtd::accumulator::Key</a>&lt;K&gt;): &<b>mut</b> V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_root_borrow_accumulator_mut">root_borrow_accumulator_mut</a>&lt;K, V: store&gt;(
    accumulator_root: &<b>mut</b> <a href="../rtd/accumulator.md#rtd_accumulator_AccumulatorRoot">AccumulatorRoot</a>,
    name: <a href="../rtd/accumulator.md#rtd_accumulator_Key">Key</a>&lt;K&gt;,
): &<b>mut</b> V {
    <a href="../rtd/dynamic_field.md#rtd_dynamic_field_borrow_mut">dynamic_field::borrow_mut</a>&lt;<a href="../rtd/accumulator.md#rtd_accumulator_Key">Key</a>&lt;K&gt;, V&gt;(&<b>mut</b> accumulator_root.id, name)
}
</code></pre>



</details>

<a name="rtd_accumulator_root_remove_accumulator"></a>

## Function `root_remove_accumulator`



<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_root_remove_accumulator">root_remove_accumulator</a>&lt;K, V: store&gt;(accumulator_root: &<b>mut</b> <a href="../rtd/accumulator.md#rtd_accumulator_AccumulatorRoot">rtd::accumulator::AccumulatorRoot</a>, name: <a href="../rtd/accumulator.md#rtd_accumulator_Key">rtd::accumulator::Key</a>&lt;K&gt;): V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_root_remove_accumulator">root_remove_accumulator</a>&lt;K, V: store&gt;(
    accumulator_root: &<b>mut</b> <a href="../rtd/accumulator.md#rtd_accumulator_AccumulatorRoot">AccumulatorRoot</a>,
    name: <a href="../rtd/accumulator.md#rtd_accumulator_Key">Key</a>&lt;K&gt;,
): V {
    <a href="../rtd/dynamic_field.md#rtd_dynamic_field_remove">dynamic_field::remove</a>&lt;<a href="../rtd/accumulator.md#rtd_accumulator_Key">Key</a>&lt;K&gt;, V&gt;(&<b>mut</b> accumulator_root.id, name)
}
</code></pre>



</details>

<a name="rtd_accumulator_emit_deposit_event"></a>

## Function `emit_deposit_event`



<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_emit_deposit_event">emit_deposit_event</a>&lt;T&gt;(<a href="../rtd/accumulator.md#rtd_accumulator">accumulator</a>: <b>address</b>, recipient: <b>address</b>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>native</b> <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_emit_deposit_event">emit_deposit_event</a>&lt;T&gt;(
    <a href="../rtd/accumulator.md#rtd_accumulator">accumulator</a>: <b>address</b>,
    recipient: <b>address</b>,
    amount: u64,
);
</code></pre>



</details>

<a name="rtd_accumulator_emit_withdraw_event"></a>

## Function `emit_withdraw_event`



<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_emit_withdraw_event">emit_withdraw_event</a>&lt;T&gt;(<a href="../rtd/accumulator.md#rtd_accumulator">accumulator</a>: <b>address</b>, owner: <b>address</b>, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>native</b> <b>fun</b> <a href="../rtd/accumulator.md#rtd_accumulator_emit_withdraw_event">emit_withdraw_event</a>&lt;T&gt;(
    <a href="../rtd/accumulator.md#rtd_accumulator">accumulator</a>: <b>address</b>,
    owner: <b>address</b>,
    amount: u64,
);
</code></pre>



</details>
