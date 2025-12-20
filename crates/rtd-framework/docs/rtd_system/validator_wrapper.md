---
title: Module `rtd_system::validator_wrapper`
---



-  [Struct `ValidatorWrapper`](#rtd_system_validator_wrapper_ValidatorWrapper)
-  [Constants](#@Constants_0)
-  [Function `create_v1`](#rtd_system_validator_wrapper_create_v1)
-  [Function `load_validator_maybe_upgrade`](#rtd_system_validator_wrapper_load_validator_maybe_upgrade)
-  [Function `destroy`](#rtd_system_validator_wrapper_destroy)
-  [Function `upgrade_to_latest`](#rtd_system_validator_wrapper_upgrade_to_latest)
-  [Function `version`](#rtd_system_validator_wrapper_version)


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
<b>use</b> <a href="../rtd/versioned.md#rtd_versioned">rtd::versioned</a>;
<b>use</b> <a href="../rtd_system/staking_pool.md#rtd_system_staking_pool">rtd_system::staking_pool</a>;
<b>use</b> <a href="../rtd_system/validator.md#rtd_system_validator">rtd_system::validator</a>;
<b>use</b> <a href="../rtd_system/validator_cap.md#rtd_system_validator_cap">rtd_system::validator_cap</a>;
</code></pre>



<a name="rtd_system_validator_wrapper_ValidatorWrapper"></a>

## Struct `ValidatorWrapper`



<pre><code><b>public</b> <b>struct</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_ValidatorWrapper">ValidatorWrapper</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>inner: <a href="../rtd/versioned.md#rtd_versioned_Versioned">rtd::versioned::Versioned</a></code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="rtd_system_validator_wrapper_EInvalidVersion"></a>



<pre><code><b>const</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_EInvalidVersion">EInvalidVersion</a>: u64 = 0;
</code></pre>



<a name="rtd_system_validator_wrapper_create_v1"></a>

## Function `create_v1`



<pre><code><b>public</b>(package) <b>fun</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_create_v1">create_v1</a>(<a href="../rtd_system/validator.md#rtd_system_validator">validator</a>: <a href="../rtd_system/validator.md#rtd_system_validator_Validator">rtd_system::validator::Validator</a>, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_ValidatorWrapper">rtd_system::validator_wrapper::ValidatorWrapper</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>fun</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_create_v1">create_v1</a>(<a href="../rtd_system/validator.md#rtd_system_validator">validator</a>: Validator, ctx: &<b>mut</b> TxContext): <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_ValidatorWrapper">ValidatorWrapper</a> {
    <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_ValidatorWrapper">ValidatorWrapper</a> {
        inner: versioned::create(1, <a href="../rtd_system/validator.md#rtd_system_validator">validator</a>, ctx),
    }
}
</code></pre>



</details>

<a name="rtd_system_validator_wrapper_load_validator_maybe_upgrade"></a>

## Function `load_validator_maybe_upgrade`

This function should always return the latest supported version.
If the inner version is old, we upgrade it lazily in-place.


<pre><code><b>public</b>(package) <b>fun</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_load_validator_maybe_upgrade">load_validator_maybe_upgrade</a>(self: &<b>mut</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_ValidatorWrapper">rtd_system::validator_wrapper::ValidatorWrapper</a>): &<b>mut</b> <a href="../rtd_system/validator.md#rtd_system_validator_Validator">rtd_system::validator::Validator</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>fun</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_load_validator_maybe_upgrade">load_validator_maybe_upgrade</a>(self: &<b>mut</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_ValidatorWrapper">ValidatorWrapper</a>): &<b>mut</b> Validator {
    self.<a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_upgrade_to_latest">upgrade_to_latest</a>();
    self.inner.load_value_mut()
}
</code></pre>



</details>

<a name="rtd_system_validator_wrapper_destroy"></a>

## Function `destroy`

Destroy the wrapper and retrieve the inner validator object.


<pre><code><b>public</b>(package) <b>fun</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_destroy">destroy</a>(self: <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_ValidatorWrapper">rtd_system::validator_wrapper::ValidatorWrapper</a>): <a href="../rtd_system/validator.md#rtd_system_validator_Validator">rtd_system::validator::Validator</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>fun</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_destroy">destroy</a>(self: <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_ValidatorWrapper">ValidatorWrapper</a>): Validator {
    <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_upgrade_to_latest">upgrade_to_latest</a>(&self);
    <b>let</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_ValidatorWrapper">ValidatorWrapper</a> { inner } = self;
    inner.<a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_destroy">destroy</a>()
}
</code></pre>



</details>

<a name="rtd_system_validator_wrapper_upgrade_to_latest"></a>

## Function `upgrade_to_latest`



<pre><code><b>fun</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_upgrade_to_latest">upgrade_to_latest</a>(self: &<a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_ValidatorWrapper">rtd_system::validator_wrapper::ValidatorWrapper</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_upgrade_to_latest">upgrade_to_latest</a>(self: &<a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_ValidatorWrapper">ValidatorWrapper</a>) {
    <b>let</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_version">version</a> = self.<a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_version">version</a>();
    // TODO: When new versions are added, we need to explicitly upgrade here.
    <b>assert</b>!(<a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_version">version</a> == 1, <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_EInvalidVersion">EInvalidVersion</a>);
}
</code></pre>



</details>

<a name="rtd_system_validator_wrapper_version"></a>

## Function `version`



<pre><code><b>fun</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_version">version</a>(self: &<a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_ValidatorWrapper">rtd_system::validator_wrapper::ValidatorWrapper</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_version">version</a>(self: &<a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_ValidatorWrapper">ValidatorWrapper</a>): u64 {
    self.inner.<a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper_version">version</a>()
}
</code></pre>



</details>
