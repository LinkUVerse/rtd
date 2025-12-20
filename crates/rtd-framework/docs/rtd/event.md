---
title: Module `rtd::event`
---

Events module. Defines the <code><a href="../rtd/event.md#rtd_event_emit">rtd::event::emit</a></code> function which
creates and sends a custom MoveEvent as a part of the effects
certificate of the transaction.

Every MoveEvent has the following properties:
- sender
- type signature (<code>T</code>)
- event data (the value of <code>T</code>)
- timestamp (local to a node)
- transaction digest

Example:
```
module my::marketplace {
use rtd::event;
/* ... */
struct ItemPurchased has copy, drop {
item_id: ID, buyer: address
}
entry fun buy(/* .... */) {
/* ... */
event::emit(ItemPurchased { item_id: ..., buyer: .... })
}
}
```


-  [Function `emit`](#rtd_event_emit)
-  [Function `emit_authenticated`](#rtd_event_emit_authenticated)
-  [Function `emit_authenticated_impl`](#rtd_event_emit_authenticated_impl)


<pre><code><b>use</b> <a href="../rtd/accumulator.md#rtd_accumulator">rtd::accumulator</a>;
<b>use</b> <a href="../rtd/accumulator_metadata.md#rtd_accumulator_metadata">rtd::accumulator_metadata</a>;
<b>use</b> <a href="../rtd/accumulator_settlement.md#rtd_accumulator_settlement">rtd::accumulator_settlement</a>;
<b>use</b> <a href="../rtd/address.md#rtd_address">rtd::address</a>;
<b>use</b> <a href="../rtd/bag.md#rtd_bag">rtd::bag</a>;
<b>use</b> <a href="../rtd/bcs.md#rtd_bcs">rtd::bcs</a>;
<b>use</b> <a href="../rtd/dynamic_field.md#rtd_dynamic_field">rtd::dynamic_field</a>;
<b>use</b> <a href="../rtd/hash.md#rtd_hash">rtd::hash</a>;
<b>use</b> <a href="../rtd/hex.md#rtd_hex">rtd::hex</a>;
<b>use</b> <a href="../rtd/object.md#rtd_object">rtd::object</a>;
<b>use</b> <a href="../rtd/party.md#rtd_party">rtd::party</a>;
<b>use</b> <a href="../rtd/transfer.md#rtd_transfer">rtd::transfer</a>;
<b>use</b> <a href="../rtd/tx_context.md#rtd_tx_context">rtd::tx_context</a>;
<b>use</b> <a href="../rtd/vec_map.md#rtd_vec_map">rtd::vec_map</a>;
<b>use</b> <a href="../std/address.md#std_address">std::address</a>;
<b>use</b> <a href="../std/ascii.md#std_ascii">std::ascii</a>;
<b>use</b> <a href="../std/bcs.md#std_bcs">std::bcs</a>;
<b>use</b> <a href="../std/option.md#std_option">std::option</a>;
<b>use</b> <a href="../std/string.md#std_string">std::string</a>;
<b>use</b> <a href="../std/type_name.md#std_type_name">std::type_name</a>;
<b>use</b> <a href="../std/vector.md#std_vector">std::vector</a>;
</code></pre>



<a name="rtd_event_emit"></a>

## Function `emit`

Emit a custom Move event, sending the data offchain.

Used for creating custom indexes and tracking onchain
activity in a way that rtdts a specific application the most.

The type <code>T</code> is the main way to index the event, and can contain
phantom parameters, eg <code><a href="../rtd/event.md#rtd_event_emit">emit</a>(MyEvent&lt;<b>phantom</b> T&gt;)</code>.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/event.md#rtd_event_emit">emit</a>&lt;T: <b>copy</b>, drop&gt;(<a href="../rtd/event.md#rtd_event">event</a>: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="../rtd/event.md#rtd_event_emit">emit</a>&lt;T: <b>copy</b> + drop&gt;(<a href="../rtd/event.md#rtd_event">event</a>: T);
</code></pre>



</details>

<a name="rtd_event_emit_authenticated"></a>

## Function `emit_authenticated`

Emits a custom Move event which can be authenticated by a light client.

This method emits the authenticated event to the event stream for the Move package that
defines the event type <code>T</code>.
Only the package that defines the type <code>T</code> can emit authenticated events to this stream.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/event.md#rtd_event_emit_authenticated">emit_authenticated</a>&lt;T: <b>copy</b>, drop&gt;(<a href="../rtd/event.md#rtd_event">event</a>: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/event.md#rtd_event_emit_authenticated">emit_authenticated</a>&lt;T: <b>copy</b> + drop&gt;(<a href="../rtd/event.md#rtd_event">event</a>: T) {
    <b>let</b> stream_id = type_name::original_id&lt;T&gt;();
    <b>let</b> accumulator_addr = <a href="../rtd/accumulator.md#rtd_accumulator_accumulator_address">accumulator::accumulator_address</a>&lt;EventStreamHead&gt;(stream_id);
    <a href="../rtd/event.md#rtd_event_emit_authenticated_impl">emit_authenticated_impl</a>&lt;EventStreamHead, T&gt;(accumulator_addr, stream_id, <a href="../rtd/event.md#rtd_event">event</a>);
}
</code></pre>



</details>

<a name="rtd_event_emit_authenticated_impl"></a>

## Function `emit_authenticated_impl`



<pre><code><b>fun</b> <a href="../rtd/event.md#rtd_event_emit_authenticated_impl">emit_authenticated_impl</a>&lt;StreamHeadT, T: <b>copy</b>, drop&gt;(accumulator_id: <b>address</b>, stream: <b>address</b>, <a href="../rtd/event.md#rtd_event">event</a>: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="../rtd/event.md#rtd_event_emit_authenticated_impl">emit_authenticated_impl</a>&lt;StreamHeadT, T: <b>copy</b> + drop&gt;(
    accumulator_id: <b>address</b>,
    stream: <b>address</b>,
    <a href="../rtd/event.md#rtd_event">event</a>: T,
);
</code></pre>



</details>
