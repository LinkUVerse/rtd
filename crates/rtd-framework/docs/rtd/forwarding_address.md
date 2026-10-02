---
title: Module `rtd::forwarding_address`
---

Registry for forwarding addresses.

A forwarding address is an off-chain-derived alias that forwards deposits to a
registered master address at resolution time. This module currently defines only the
singleton registry object; registration and resolution APIs are added in later steps.


-  [Struct `ForwardingAddressRegistry`](#rtd_forwarding_address_ForwardingAddressRegistry)
-  [Constants](#@Constants_0)
-  [Function `create`](#rtd_forwarding_address_create)


<pre><code><b>use</b> <a href="../rtd/address.md#rtd_address">rtd::address</a>;
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



<a name="rtd_forwarding_address_ForwardingAddressRegistry"></a>

## Struct `ForwardingAddressRegistry`

Singleton shared object which will hold forwarding address registrations.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/forwarding_address.md#rtd_forwarding_address_ForwardingAddressRegistry">ForwardingAddressRegistry</a> <b>has</b> key
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

<a name="@Constants_0"></a>

## Constants


<a name="rtd_forwarding_address_ENotSystemAddress"></a>



<pre><code>#[error]
<b>const</b> <a href="../rtd/forwarding_address.md#rtd_forwarding_address_ENotSystemAddress">ENotSystemAddress</a>: vector&lt;u8&gt; = b"Only the system can <a href="../rtd/forwarding_address.md#rtd_forwarding_address_create">create</a> the forwarding <b>address</b> registry.";
</code></pre>



<a name="rtd_forwarding_address_create"></a>

## Function `create`

Create and share the <code><a href="../rtd/forwarding_address.md#rtd_forwarding_address_ForwardingAddressRegistry">ForwardingAddressRegistry</a></code> object. This function is called exactly
once, when the registry object is first created. Can only be called by genesis or
change_epoch transactions.


<pre><code><b>fun</b> <a href="../rtd/forwarding_address.md#rtd_forwarding_address_create">create</a>(ctx: &<a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../rtd/forwarding_address.md#rtd_forwarding_address_create">create</a>(ctx: &TxContext) {
    <b>assert</b>!(ctx.sender() == @0x0, <a href="../rtd/forwarding_address.md#rtd_forwarding_address_ENotSystemAddress">ENotSystemAddress</a>);
    <b>let</b> self = <a href="../rtd/forwarding_address.md#rtd_forwarding_address_ForwardingAddressRegistry">ForwardingAddressRegistry</a> {
        id: <a href="../rtd/object.md#rtd_object_forwarding_address_registry">object::forwarding_address_registry</a>(),
    };
    <a href="../rtd/transfer.md#rtd_transfer_share_object">transfer::share_object</a>(self);
}
</code></pre>



</details>
