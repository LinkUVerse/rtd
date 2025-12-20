---
title: Module `rtd_system::stake_subsidy`
---



-  [Struct `StakeSubsidy`](#rtd_system_stake_subsidy_StakeSubsidy)
-  [Constants](#@Constants_0)
-  [Function `create`](#rtd_system_stake_subsidy_create)
-  [Function `advance_epoch`](#rtd_system_stake_subsidy_advance_epoch)
-  [Function `current_epoch_subsidy_amount`](#rtd_system_stake_subsidy_current_epoch_subsidy_amount)
-  [Function `get_distribution_counter`](#rtd_system_stake_subsidy_get_distribution_counter)


<pre><code><b>use</b> <a href="../std/address.md#std_address">std::address</a>;
<b>use</b> <a href="../std/ascii.md#std_ascii">std::ascii</a>;
<b>use</b> <a href="../std/bcs.md#std_bcs">std::bcs</a>;
<b>use</b> <a href="../std/internal.md#std_internal">std::internal</a>;
<b>use</b> <a href="../std/option.md#std_option">std::option</a>;
<b>use</b> <a href="../std/string.md#std_string">std::string</a>;
<b>use</b> <a href="../std/type_name.md#std_type_name">std::type_name</a>;
<b>use</b> <a href="../std/u64.md#std_u64">std::u64</a>;
<b>use</b> <a href="../std/vector.md#std_vector">std::vector</a>;
<b>use</b> <a href="../rtd/accumulator.md#rtd_accumulator">rtd::accumulator</a>;
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
</code></pre>



<a name="rtd_system_stake_subsidy_StakeSubsidy"></a>

## Struct `StakeSubsidy`



<pre><code><b>public</b> <b>struct</b> <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_StakeSubsidy">StakeSubsidy</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>balance: <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;<a href="../rtd/rtd.md#rtd_rtd_RTD">rtd::rtd::RTD</a>&gt;</code>
</dt>
<dd>
 Balance of RTD set aside for stake subsidies that will be drawn down over time.
</dd>
<dt>
<code>distribution_counter: u64</code>
</dt>
<dd>
 Count of the number of times stake subsidies have been distributed.
</dd>
<dt>
<code>current_distribution_amount: u64</code>
</dt>
<dd>
 The amount of stake subsidy to be drawn down per distribution.
 This amount decays and decreases over time.
</dd>
<dt>
<code>stake_subsidy_period_length: u64</code>
</dt>
<dd>
 Number of distributions to occur before the distribution amount decays.
</dd>
<dt>
<code>stake_subsidy_decrease_rate: u16</code>
</dt>
<dd>
 The rate at which the distribution amount decays at the end of each
 period. Expressed in basis points.
</dd>
<dt>
<code>extra_fields: <a href="../rtd/bag.md#rtd_bag_Bag">rtd::bag::Bag</a></code>
</dt>
<dd>
 Any extra fields that's not defined statically.
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="rtd_system_stake_subsidy_ESubsidyDecreaseRateTooLarge"></a>



<pre><code><b>const</b> <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_ESubsidyDecreaseRateTooLarge">ESubsidyDecreaseRateTooLarge</a>: u64 = 0;
</code></pre>



<a name="rtd_system_stake_subsidy_BASIS_POINT_DENOMINATOR"></a>



<pre><code><b>const</b> <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_BASIS_POINT_DENOMINATOR">BASIS_POINT_DENOMINATOR</a>: u128 = 10000;
</code></pre>



<a name="rtd_system_stake_subsidy_create"></a>

## Function `create`



<pre><code><b>public</b>(package) <b>fun</b> <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_create">create</a>(balance: <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;<a href="../rtd/rtd.md#rtd_rtd_RTD">rtd::rtd::RTD</a>&gt;, initial_distribution_amount: u64, stake_subsidy_period_length: u64, stake_subsidy_decrease_rate: u16, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_StakeSubsidy">rtd_system::stake_subsidy::StakeSubsidy</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>fun</b> <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_create">create</a>(
    balance: Balance&lt;RTD&gt;,
    initial_distribution_amount: u64,
    stake_subsidy_period_length: u64,
    stake_subsidy_decrease_rate: u16,
    ctx: &<b>mut</b> TxContext,
): <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_StakeSubsidy">StakeSubsidy</a> {
    // Rate can't be higher than 100%.
    <b>assert</b>!(
        stake_subsidy_decrease_rate &lt;= <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_BASIS_POINT_DENOMINATOR">BASIS_POINT_DENOMINATOR</a> <b>as</b> u16,
        <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_ESubsidyDecreaseRateTooLarge">ESubsidyDecreaseRateTooLarge</a>,
    );
    <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_StakeSubsidy">StakeSubsidy</a> {
        balance,
        distribution_counter: 0,
        current_distribution_amount: initial_distribution_amount,
        stake_subsidy_period_length,
        stake_subsidy_decrease_rate,
        extra_fields: bag::new(ctx),
    }
}
</code></pre>



</details>

<a name="rtd_system_stake_subsidy_advance_epoch"></a>

## Function `advance_epoch`

Advance the epoch counter and draw down the subsidy for the epoch.


<pre><code><b>public</b>(package) <b>fun</b> <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_advance_epoch">advance_epoch</a>(self: &<b>mut</b> <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_StakeSubsidy">rtd_system::stake_subsidy::StakeSubsidy</a>): <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;<a href="../rtd/rtd.md#rtd_rtd_RTD">rtd::rtd::RTD</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>fun</b> <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_advance_epoch">advance_epoch</a>(self: &<b>mut</b> <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_StakeSubsidy">StakeSubsidy</a>): Balance&lt;RTD&gt; {
    // Take the minimum of the reward amount and the remaining balance in
    // order to ensure we don't overdraft the remaining stake subsidy
    // balance
    <b>let</b> to_withdraw = self.current_distribution_amount.min(self.balance.value());
    // Drawn down the subsidy <b>for</b> this epoch.
    <b>let</b> <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy">stake_subsidy</a> = self.balance.split(to_withdraw);
    self.distribution_counter = self.distribution_counter + 1;
    // Decrease the subsidy amount only when the current period ends.
    <b>if</b> (self.distribution_counter % self.stake_subsidy_period_length == 0) {
        <b>let</b> decrease_amount =
            self.current_distribution_amount <b>as</b> u128
            * (self.stake_subsidy_decrease_rate <b>as</b> u128) / <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_BASIS_POINT_DENOMINATOR">BASIS_POINT_DENOMINATOR</a>;
        self.current_distribution_amount =
            self.current_distribution_amount - (decrease_amount <b>as</b> u64)
    };
    <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy">stake_subsidy</a>
}
</code></pre>



</details>

<a name="rtd_system_stake_subsidy_current_epoch_subsidy_amount"></a>

## Function `current_epoch_subsidy_amount`

Returns the amount of stake subsidy to be added at the end of the current epoch.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_current_epoch_subsidy_amount">current_epoch_subsidy_amount</a>(self: &<a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_StakeSubsidy">rtd_system::stake_subsidy::StakeSubsidy</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_current_epoch_subsidy_amount">current_epoch_subsidy_amount</a>(self: &<a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_StakeSubsidy">StakeSubsidy</a>): u64 {
    self.current_distribution_amount.min(self.balance.value())
}
</code></pre>



</details>

<a name="rtd_system_stake_subsidy_get_distribution_counter"></a>

## Function `get_distribution_counter`

Returns the number of distributions that have occurred.


<pre><code><b>public</b>(package) <b>fun</b> <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_get_distribution_counter">get_distribution_counter</a>(self: &<a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_StakeSubsidy">rtd_system::stake_subsidy::StakeSubsidy</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>fun</b> <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_get_distribution_counter">get_distribution_counter</a>(self: &<a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_StakeSubsidy">StakeSubsidy</a>): u64 {
    self.distribution_counter
}
</code></pre>



</details>
