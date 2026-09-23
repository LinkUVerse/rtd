---
title: Module `rtd::clock`
---

APIs for accessing time from move calls, via the <code><a href="../rtd/clock.md#rtd_clock_Clock">Clock</a></code>: a unique
shared object that is created at 0x6 during genesis.


-  [Struct `Clock`](#rtd_clock_Clock)
-  [Constants](#@Constants_0)
-  [Function `timestamp_ms`](#rtd_clock_timestamp_ms)
-  [Function `create`](#rtd_clock_create)
-  [Function `consensus_commit_prologue`](#rtd_clock_consensus_commit_prologue)


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



<a name="rtd_clock_Clock"></a>

## Struct `Clock`

Singleton shared object that exposes time to Move calls.  This
object is found at address 0x6, and can only be read (accessed
via an immutable reference) by entry functions.

Entry Functions that attempt to accept <code><a href="../rtd/clock.md#rtd_clock_Clock">Clock</a></code> by mutable
reference or value will fail to verify, and honest validators
will not sign or execute transactions that use <code><a href="../rtd/clock.md#rtd_clock_Clock">Clock</a></code> as an
input parameter, unless it is passed by immutable reference.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/clock.md#rtd_clock_Clock">Clock</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="../rtd/object.md#rtd_object_UID">rtd::object::UID</a></code>
</dt>
<dd>
</dd>
<dt>
<code><a href="../rtd/clock.md#rtd_clock_timestamp_ms">timestamp_ms</a>: u64</code>
</dt>
<dd>
 The clock's timestamp, which is set automatically by a
 system transaction every time consensus commits a
 schedule, or by <code>rtd::clock::increment_for_testing</code> during
 testing.
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="rtd_clock_ENotSystemAddress"></a>

Sender is not @0x0 the system address.


<pre><code><b>const</b> <a href="../rtd/clock.md#rtd_clock_ENotSystemAddress">ENotSystemAddress</a>: u64 = 0;
</code></pre>



<a name="rtd_clock_timestamp_ms"></a>

## Function `timestamp_ms`

The <code><a href="../rtd/clock.md#rtd_clock">clock</a></code>'s current timestamp as a running total of
milliseconds since an arbitrary point in the past.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/clock.md#rtd_clock_timestamp_ms">timestamp_ms</a>(<a href="../rtd/clock.md#rtd_clock">clock</a>: &<a href="../rtd/clock.md#rtd_clock_Clock">rtd::clock::Clock</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/clock.md#rtd_clock_timestamp_ms">timestamp_ms</a>(<a href="../rtd/clock.md#rtd_clock">clock</a>: &<a href="../rtd/clock.md#rtd_clock_Clock">Clock</a>): u64 {
    <a href="../rtd/clock.md#rtd_clock">clock</a>.<a href="../rtd/clock.md#rtd_clock_timestamp_ms">timestamp_ms</a>
}
</code></pre>



</details>

<a name="rtd_clock_create"></a>

## Function `create`

Create and share the singleton Clock -- this function is
called exactly once, during genesis.


<pre><code><b>fun</b> <a href="../rtd/clock.md#rtd_clock_create">create</a>(ctx: &<a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../rtd/clock.md#rtd_clock_create">create</a>(ctx: &TxContext) {
    <b>assert</b>!(ctx.sender() == @0x0, <a href="../rtd/clock.md#rtd_clock_ENotSystemAddress">ENotSystemAddress</a>);
    <a href="../rtd/transfer.md#rtd_transfer_share_object">transfer::share_object</a>(<a href="../rtd/clock.md#rtd_clock_Clock">Clock</a> {
        id: <a href="../rtd/object.md#rtd_object_clock">object::clock</a>(),
        // Initialised to zero, but set to a real timestamp by a
        // system transaction before it can be witnessed by a <b>move</b>
        // call.
        <a href="../rtd/clock.md#rtd_clock_timestamp_ms">timestamp_ms</a>: 0,
    })
}
</code></pre>



</details>

<a name="rtd_clock_consensus_commit_prologue"></a>

## Function `consensus_commit_prologue`



<pre><code><b>fun</b> <a href="../rtd/clock.md#rtd_clock_consensus_commit_prologue">consensus_commit_prologue</a>(<a href="../rtd/clock.md#rtd_clock">clock</a>: &<b>mut</b> <a href="../rtd/clock.md#rtd_clock_Clock">rtd::clock::Clock</a>, <a href="../rtd/clock.md#rtd_clock_timestamp_ms">timestamp_ms</a>: u64, ctx: &<a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../rtd/clock.md#rtd_clock_consensus_commit_prologue">consensus_commit_prologue</a>(<a href="../rtd/clock.md#rtd_clock">clock</a>: &<b>mut</b> <a href="../rtd/clock.md#rtd_clock_Clock">Clock</a>, <a href="../rtd/clock.md#rtd_clock_timestamp_ms">timestamp_ms</a>: u64, ctx: &TxContext) {
    // Validator will make a special system call with sender set <b>as</b> 0x0.
    <b>assert</b>!(ctx.sender() == @0x0, <a href="../rtd/clock.md#rtd_clock_ENotSystemAddress">ENotSystemAddress</a>);
    <a href="../rtd/clock.md#rtd_clock">clock</a>.<a href="../rtd/clock.md#rtd_clock_timestamp_ms">timestamp_ms</a> = <a href="../rtd/clock.md#rtd_clock_timestamp_ms">timestamp_ms</a>
}
</code></pre>



</details>
