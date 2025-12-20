---
title: Module `rtd_system::storage_fund`
---



-  [Struct `StorageFund`](#rtd_system_storage_fund_StorageFund)
-  [Function `new`](#rtd_system_storage_fund_new)
-  [Function `advance_epoch`](#rtd_system_storage_fund_advance_epoch)
-  [Function `total_object_storage_rebates`](#rtd_system_storage_fund_total_object_storage_rebates)
-  [Function `total_balance`](#rtd_system_storage_fund_total_balance)


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
<b>use</b> <a href="../rtd/rtd.md#rtd_rtd">rtd::rtd</a>;
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



<a name="rtd_system_storage_fund_StorageFund"></a>

## Struct `StorageFund`

Struct representing the storage fund, containing two <code>Balance</code>s:
- <code><a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_total_object_storage_rebates">total_object_storage_rebates</a></code> has the invariant that it's the sum of <code>storage_rebate</code> of
all objects currently stored on-chain. To maintain this invariant, the only inflow of this
balance is storage charges collected from transactions, and the only outflow is storage rebates
of transactions, including both the portion refunded to the transaction senders as well as
the non-refundable portion taken out and put into <code>non_refundable_balance</code>.
- <code>non_refundable_balance</code> contains any remaining inflow of the storage fund that should not
be taken out of the fund.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_StorageFund">StorageFund</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_total_object_storage_rebates">total_object_storage_rebates</a>: <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;<a href="../rtd/rtd.md#rtd_rtd_RTD">rtd::rtd::RTD</a>&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>non_refundable_balance: <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;<a href="../rtd/rtd.md#rtd_rtd_RTD">rtd::rtd::RTD</a>&gt;</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="rtd_system_storage_fund_new"></a>

## Function `new`

Called by <code><a href="../rtd_system/sui_system.md#rtd_system_rtd_system">rtd_system</a></code> at genesis time.


<pre><code><b>public</b>(package) <b>fun</b> <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_new">new</a>(initial_fund: <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;<a href="../rtd/rtd.md#rtd_rtd_RTD">rtd::rtd::RTD</a>&gt;): <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_StorageFund">rtd_system::storage_fund::StorageFund</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>fun</b> <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_new">new</a>(initial_fund: Balance&lt;RTD&gt;): <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_StorageFund">StorageFund</a> {
    <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_StorageFund">StorageFund</a> {
        // At the beginning there's no object in the storage yet
        <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_total_object_storage_rebates">total_object_storage_rebates</a>: balance::zero(),
        non_refundable_balance: initial_fund,
    }
}
</code></pre>



</details>

<a name="rtd_system_storage_fund_advance_epoch"></a>

## Function `advance_epoch`

Called by <code><a href="../rtd_system/sui_system.md#rtd_system_rtd_system">rtd_system</a></code> at epoch change times to process the inflows and outflows of storage fund.


<pre><code><b>public</b>(package) <b>fun</b> <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_advance_epoch">advance_epoch</a>(self: &<b>mut</b> <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_StorageFund">rtd_system::storage_fund::StorageFund</a>, storage_charges: <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;<a href="../rtd/rtd.md#rtd_rtd_RTD">rtd::rtd::RTD</a>&gt;, storage_fund_reinvestment: <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;<a href="../rtd/rtd.md#rtd_rtd_RTD">rtd::rtd::RTD</a>&gt;, leftover_staking_rewards: <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;<a href="../rtd/rtd.md#rtd_rtd_RTD">rtd::rtd::RTD</a>&gt;, storage_rebate_amount: u64, non_refundable_storage_fee_amount: u64): <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;<a href="../rtd/rtd.md#rtd_rtd_RTD">rtd::rtd::RTD</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>fun</b> <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_advance_epoch">advance_epoch</a>(
    self: &<b>mut</b> <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_StorageFund">StorageFund</a>,
    storage_charges: Balance&lt;RTD&gt;,
    storage_fund_reinvestment: Balance&lt;RTD&gt;,
    leftover_staking_rewards: Balance&lt;RTD&gt;,
    storage_rebate_amount: u64,
    non_refundable_storage_fee_amount: u64,
): Balance&lt;RTD&gt; {
    // Both the reinvestment and leftover rewards are not to be refunded so they go to the non-refundable balance.
    self.non_refundable_balance.join(storage_fund_reinvestment);
    self.non_refundable_balance.join(leftover_staking_rewards);
    // The storage charges <b>for</b> the epoch come from the storage rebate of the <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_new">new</a> objects created
    // and the <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_new">new</a> storage rebates of the objects modified during the epoch so we put the charges
    // into `<a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_total_object_storage_rebates">total_object_storage_rebates</a>`.
    self.<a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_total_object_storage_rebates">total_object_storage_rebates</a>.join(storage_charges);
    // Split out the non-refundable portion of the storage rebate and put it into the non-refundable balance.
    <b>let</b> non_refundable_storage_fee = self
        .<a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_total_object_storage_rebates">total_object_storage_rebates</a>
        .split(non_refundable_storage_fee_amount);
    self.non_refundable_balance.join(non_refundable_storage_fee);
    // `storage_rebates` include the already refunded rebates of deleted objects and old rebates of modified objects and
    // should be taken out of the `<a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_total_object_storage_rebates">total_object_storage_rebates</a>`.
    <b>let</b> storage_rebate = self.<a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_total_object_storage_rebates">total_object_storage_rebates</a>.split(storage_rebate_amount);
    // The storage rebate <b>has</b> already been returned to individual transaction senders' gas coins
    // so we <b>return</b> the balance to be burnt at the very end of epoch change.
    storage_rebate
}
</code></pre>



</details>

<a name="rtd_system_storage_fund_total_object_storage_rebates"></a>

## Function `total_object_storage_rebates`



<pre><code><b>public</b> <b>fun</b> <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_total_object_storage_rebates">total_object_storage_rebates</a>(self: &<a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_StorageFund">rtd_system::storage_fund::StorageFund</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_total_object_storage_rebates">total_object_storage_rebates</a>(self: &<a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_StorageFund">StorageFund</a>): u64 {
    self.<a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_total_object_storage_rebates">total_object_storage_rebates</a>.value()
}
</code></pre>



</details>

<a name="rtd_system_storage_fund_total_balance"></a>

## Function `total_balance`



<pre><code><b>public</b> <b>fun</b> <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_total_balance">total_balance</a>(self: &<a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_StorageFund">rtd_system::storage_fund::StorageFund</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_total_balance">total_balance</a>(self: &<a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_StorageFund">StorageFund</a>): u64 {
    self.<a href="../rtd_system/storage_fund.md#rtd_system_storage_fund_total_object_storage_rebates">total_object_storage_rebates</a>.value() + self.non_refundable_balance.value()
}
</code></pre>



</details>
