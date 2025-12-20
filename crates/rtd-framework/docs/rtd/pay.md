---
title: Module `rtd::pay`
---

This module provides handy functionality for wallets and <code>rtd::Coin</code> management.


-  [Constants](#@Constants_0)
-  [Function `keep`](#rtd_pay_keep)
-  [Function `split`](#rtd_pay_split)
-  [Function `split_vec`](#rtd_pay_split_vec)
-  [Function `split_and_transfer`](#rtd_pay_split_and_transfer)
-  [Function `divide_and_keep`](#rtd_pay_divide_and_keep)
-  [Function `join`](#rtd_pay_join)
-  [Function `join_vec`](#rtd_pay_join_vec)
-  [Function `join_vec_and_transfer`](#rtd_pay_join_vec_and_transfer)


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



<a name="@Constants_0"></a>

## Constants


<a name="rtd_pay_ENoCoins"></a>

For when empty vector is supplied into join function.


<pre><code><b>const</b> <a href="../rtd/pay.md#rtd_pay_ENoCoins">ENoCoins</a>: u64 = 0;
</code></pre>



<a name="rtd_pay_keep"></a>

## Function `keep`

Transfer <code>c</code> to the sender of the current transaction


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/pay.md#rtd_pay_keep">keep</a>&lt;T&gt;(c: <a href="../rtd/coin.md#rtd_coin_Coin">rtd::coin::Coin</a>&lt;T&gt;, ctx: &<a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/pay.md#rtd_pay_keep">keep</a>&lt;T&gt;(c: Coin&lt;T&gt;, ctx: &TxContext) {
    <a href="../rtd/transfer.md#rtd_transfer_public_transfer">transfer::public_transfer</a>(c, ctx.sender())
}
</code></pre>



</details>

<a name="rtd_pay_split"></a>

## Function `split`

Split <code><a href="../rtd/coin.md#rtd_coin">coin</a></code> to two coins, one with balance <code>split_amount</code>,
and the remaining balance is left in <code><a href="../rtd/coin.md#rtd_coin">coin</a></code>.


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../rtd/pay.md#rtd_pay_split">split</a>&lt;T&gt;(<a href="../rtd/coin.md#rtd_coin">coin</a>: &<b>mut</b> <a href="../rtd/coin.md#rtd_coin_Coin">rtd::coin::Coin</a>&lt;T&gt;, split_amount: u64, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../rtd/pay.md#rtd_pay_split">split</a>&lt;T&gt;(<a href="../rtd/coin.md#rtd_coin">coin</a>: &<b>mut</b> Coin&lt;T&gt;, split_amount: u64, ctx: &<b>mut</b> TxContext) {
    <a href="../rtd/pay.md#rtd_pay_keep">keep</a>(<a href="../rtd/coin.md#rtd_coin">coin</a>.<a href="../rtd/pay.md#rtd_pay_split">split</a>(split_amount, ctx), ctx)
}
</code></pre>



</details>

<a name="rtd_pay_split_vec"></a>

## Function `split_vec`

Split coin <code>self</code> into multiple coins, each with balance specified
in <code>split_amounts</code>. Remaining balance is left in <code>self</code>.


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../rtd/pay.md#rtd_pay_split_vec">split_vec</a>&lt;T&gt;(self: &<b>mut</b> <a href="../rtd/coin.md#rtd_coin_Coin">rtd::coin::Coin</a>&lt;T&gt;, split_amounts: vector&lt;u64&gt;, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../rtd/pay.md#rtd_pay_split_vec">split_vec</a>&lt;T&gt;(self: &<b>mut</b> Coin&lt;T&gt;, split_amounts: vector&lt;u64&gt;, ctx: &<b>mut</b> TxContext) {
    split_amounts.do!(|amount| <a href="../rtd/pay.md#rtd_pay_split">split</a>(self, amount, ctx));
}
</code></pre>



</details>

<a name="rtd_pay_split_and_transfer"></a>

## Function `split_and_transfer`

Send <code>amount</code> units of <code>c</code> to <code>recipient</code>
Aborts with <code><a href="../rtd/balance.md#rtd_balance_ENotEnough">rtd::balance::ENotEnough</a></code> if <code>amount</code> is greater than the balance in <code>c</code>


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../rtd/pay.md#rtd_pay_split_and_transfer">split_and_transfer</a>&lt;T&gt;(c: &<b>mut</b> <a href="../rtd/coin.md#rtd_coin_Coin">rtd::coin::Coin</a>&lt;T&gt;, amount: u64, recipient: <b>address</b>, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../rtd/pay.md#rtd_pay_split_and_transfer">split_and_transfer</a>&lt;T&gt;(
    c: &<b>mut</b> Coin&lt;T&gt;,
    amount: u64,
    recipient: <b>address</b>,
    ctx: &<b>mut</b> TxContext,
) {
    <a href="../rtd/transfer.md#rtd_transfer_public_transfer">transfer::public_transfer</a>(c.<a href="../rtd/pay.md#rtd_pay_split">split</a>(amount, ctx), recipient)
}
</code></pre>



</details>

<a name="rtd_pay_divide_and_keep"></a>

## Function `divide_and_keep`

Divide coin <code>self</code> into <code>n - 1</code> coins with equal balances. If the balance is
not evenly divisible by <code>n</code>, the remainder is left in <code>self</code>.


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../rtd/pay.md#rtd_pay_divide_and_keep">divide_and_keep</a>&lt;T&gt;(self: &<b>mut</b> <a href="../rtd/coin.md#rtd_coin_Coin">rtd::coin::Coin</a>&lt;T&gt;, n: u64, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../rtd/pay.md#rtd_pay_divide_and_keep">divide_and_keep</a>&lt;T&gt;(self: &<b>mut</b> Coin&lt;T&gt;, n: u64, ctx: &<b>mut</b> TxContext) {
    self.divide_into_n(n, ctx).destroy!(|<a href="../rtd/coin.md#rtd_coin">coin</a>| <a href="../rtd/transfer.md#rtd_transfer_public_transfer">transfer::public_transfer</a>(<a href="../rtd/coin.md#rtd_coin">coin</a>, ctx.sender()));
}
</code></pre>



</details>

<a name="rtd_pay_join"></a>

## Function `join`

Join <code><a href="../rtd/coin.md#rtd_coin">coin</a></code> into <code>self</code>. Re-exports <code><a href="../rtd/coin.md#rtd_coin_join">coin::join</a></code> function.
Deprecated: you should call <code><a href="../rtd/coin.md#rtd_coin">coin</a>.<a href="../rtd/pay.md#rtd_pay_join">join</a>(other)</code> directly.


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../rtd/pay.md#rtd_pay_join">join</a>&lt;T&gt;(self: &<b>mut</b> <a href="../rtd/coin.md#rtd_coin_Coin">rtd::coin::Coin</a>&lt;T&gt;, <a href="../rtd/coin.md#rtd_coin">coin</a>: <a href="../rtd/coin.md#rtd_coin_Coin">rtd::coin::Coin</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../rtd/pay.md#rtd_pay_join">join</a>&lt;T&gt;(self: &<b>mut</b> Coin&lt;T&gt;, <a href="../rtd/coin.md#rtd_coin">coin</a>: Coin&lt;T&gt;) {
    self.<a href="../rtd/pay.md#rtd_pay_join">join</a>(<a href="../rtd/coin.md#rtd_coin">coin</a>)
}
</code></pre>



</details>

<a name="rtd_pay_join_vec"></a>

## Function `join_vec`

Join everything in <code>coins</code> with <code>self</code>


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../rtd/pay.md#rtd_pay_join_vec">join_vec</a>&lt;T&gt;(self: &<b>mut</b> <a href="../rtd/coin.md#rtd_coin_Coin">rtd::coin::Coin</a>&lt;T&gt;, coins: vector&lt;<a href="../rtd/coin.md#rtd_coin_Coin">rtd::coin::Coin</a>&lt;T&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../rtd/pay.md#rtd_pay_join_vec">join_vec</a>&lt;T&gt;(self: &<b>mut</b> Coin&lt;T&gt;, coins: vector&lt;Coin&lt;T&gt;&gt;) {
    coins.destroy!(|<a href="../rtd/coin.md#rtd_coin">coin</a>| self.<a href="../rtd/pay.md#rtd_pay_join">join</a>(<a href="../rtd/coin.md#rtd_coin">coin</a>));
}
</code></pre>



</details>

<a name="rtd_pay_join_vec_and_transfer"></a>

## Function `join_vec_and_transfer`

Join a vector of <code>Coin</code> into a single object and transfer it to <code>receiver</code>.


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../rtd/pay.md#rtd_pay_join_vec_and_transfer">join_vec_and_transfer</a>&lt;T&gt;(coins: vector&lt;<a href="../rtd/coin.md#rtd_coin_Coin">rtd::coin::Coin</a>&lt;T&gt;&gt;, receiver: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../rtd/pay.md#rtd_pay_join_vec_and_transfer">join_vec_and_transfer</a>&lt;T&gt;(<b>mut</b> coins: vector&lt;Coin&lt;T&gt;&gt;, receiver: <b>address</b>) {
    <b>assert</b>!(coins.length() &gt; 0, <a href="../rtd/pay.md#rtd_pay_ENoCoins">ENoCoins</a>);
    <b>let</b> <b>mut</b> self = coins.pop_back();
    <a href="../rtd/pay.md#rtd_pay_join_vec">join_vec</a>(&<b>mut</b> self, coins);
    <a href="../rtd/transfer.md#rtd_transfer_public_transfer">transfer::public_transfer</a>(self, receiver)
}
</code></pre>



</details>
