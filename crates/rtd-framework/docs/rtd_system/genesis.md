---
title: Module `rtd_system::genesis`
---



-  [Struct `GenesisValidatorMetadata`](#rtd_system_genesis_GenesisValidatorMetadata)
-  [Struct `GenesisChainParameters`](#rtd_system_genesis_GenesisChainParameters)
-  [Struct `TokenDistributionSchedule`](#rtd_system_genesis_TokenDistributionSchedule)
-  [Struct `TokenAllocation`](#rtd_system_genesis_TokenAllocation)
-  [Constants](#@Constants_0)
-  [Function `create`](#rtd_system_genesis_create)
-  [Function `allocate_tokens`](#rtd_system_genesis_allocate_tokens)


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
<b>use</b> <a href="../rtd/priority_queue.md#rtd_priority_queue">rtd::priority_queue</a>;
<b>use</b> <a href="../rtd/protocol_config.md#rtd_protocol_config">rtd::protocol_config</a>;
<b>use</b> <a href="../rtd/rtd.md#rtd_rtd">rtd::rtd</a>;
<b>use</b> <a href="../rtd/table.md#rtd_table">rtd::table</a>;
<b>use</b> <a href="../rtd/table_vec.md#rtd_table_vec">rtd::table_vec</a>;
<b>use</b> <a href="../rtd/transfer.md#rtd_transfer">rtd::transfer</a>;
<b>use</b> <a href="../rtd/tx_context.md#rtd_tx_context">rtd::tx_context</a>;
<b>use</b> <a href="../rtd/types.md#rtd_types">rtd::types</a>;
<b>use</b> <a href="../rtd/url.md#rtd_url">rtd::url</a>;
<b>use</b> <a href="../rtd/vec_map.md#rtd_vec_map">rtd::vec_map</a>;
<b>use</b> <a href="../rtd/vec_set.md#rtd_vec_set">rtd::vec_set</a>;
<b>use</b> <a href="../rtd/versioned.md#rtd_versioned">rtd::versioned</a>;
<b>use</b> <a href="../rtd_system/sui_system.md#rtd_system_rtd_system">rtd_system::rtd_system</a>;
<b>use</b> <a href="../rtd_system/sui_system_state_inner.md#rtd_system_rtd_system_state_inner">rtd_system::rtd_system_state_inner</a>;
<b>use</b> <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy">rtd_system::stake_subsidy</a>;
<b>use</b> <a href="../rtd_system/staking_pool.md#rtd_system_staking_pool">rtd_system::staking_pool</a>;
<b>use</b> <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund">rtd_system::storage_fund</a>;
<b>use</b> <a href="../rtd_system/validator.md#rtd_system_validator">rtd_system::validator</a>;
<b>use</b> <a href="../rtd_system/validator_cap.md#rtd_system_validator_cap">rtd_system::validator_cap</a>;
<b>use</b> <a href="../rtd_system/validator_set.md#rtd_system_validator_set">rtd_system::validator_set</a>;
<b>use</b> <a href="../rtd_system/validator_wrapper.md#rtd_system_validator_wrapper">rtd_system::validator_wrapper</a>;
<b>use</b> <a href="../rtd_system/voting_power.md#rtd_system_voting_power">rtd_system::voting_power</a>;
<b>use</b> <a href="../std/address.md#std_address">std::address</a>;
<b>use</b> <a href="../std/ascii.md#std_ascii">std::ascii</a>;
<b>use</b> <a href="../std/bcs.md#std_bcs">std::bcs</a>;
<b>use</b> <a href="../std/internal.md#std_internal">std::internal</a>;
<b>use</b> <a href="../std/option.md#std_option">std::option</a>;
<b>use</b> <a href="../std/string.md#std_string">std::string</a>;
<b>use</b> <a href="../std/type_name.md#std_type_name">std::type_name</a>;
<b>use</b> <a href="../std/u64.md#std_u64">std::u64</a>;
<b>use</b> <a href="../std/vector.md#std_vector">std::vector</a>;
</code></pre>



<a name="rtd_system_genesis_GenesisValidatorMetadata"></a>

## Struct `GenesisValidatorMetadata`



<pre><code><b>public</b> <b>struct</b> <a href="../rtd_system/genesis.md#rtd_system_genesis_GenesisValidatorMetadata">GenesisValidatorMetadata</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>name: vector&lt;u8&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>description: vector&lt;u8&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>image_url: vector&lt;u8&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>project_url: vector&lt;u8&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>rtd_address: <b>address</b></code>
</dt>
<dd>
</dd>
<dt>
<code>gas_price: u64</code>
</dt>
<dd>
</dd>
<dt>
<code>commission_rate: u64</code>
</dt>
<dd>
</dd>
<dt>
<code>protocol_public_key: vector&lt;u8&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>proof_of_possession: vector&lt;u8&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>network_public_key: vector&lt;u8&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>worker_public_key: vector&lt;u8&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>network_address: vector&lt;u8&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>p2p_address: vector&lt;u8&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>primary_address: vector&lt;u8&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>worker_address: vector&lt;u8&gt;</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="rtd_system_genesis_GenesisChainParameters"></a>

## Struct `GenesisChainParameters`



<pre><code><b>public</b> <b>struct</b> <a href="../rtd_system/genesis.md#rtd_system_genesis_GenesisChainParameters">GenesisChainParameters</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>protocol_version: u64</code>
</dt>
<dd>
</dd>
<dt>
<code>chain_start_timestamp_ms: u64</code>
</dt>
<dd>
</dd>
<dt>
<code>epoch_duration_ms: u64</code>
</dt>
<dd>
</dd>
<dt>
<code>stake_subsidy_start_epoch: u64</code>
</dt>
<dd>
 Stake Subsidy parameters
</dd>
<dt>
<code>stake_subsidy_initial_distribution_amount: u64</code>
</dt>
<dd>
</dd>
<dt>
<code>stake_subsidy_period_length: u64</code>
</dt>
<dd>
</dd>
<dt>
<code>stake_subsidy_decrease_rate: u16</code>
</dt>
<dd>
</dd>
<dt>
<code>max_validator_count: u64</code>
</dt>
<dd>
 Validator committee parameters
</dd>
<dt>
<code>min_validator_joining_stake: u64</code>
</dt>
<dd>
</dd>
<dt>
<code>validator_low_stake_threshold: u64</code>
</dt>
<dd>
</dd>
<dt>
<code>validator_very_low_stake_threshold: u64</code>
</dt>
<dd>
</dd>
<dt>
<code>validator_low_stake_grace_period: u64</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="rtd_system_genesis_TokenDistributionSchedule"></a>

## Struct `TokenDistributionSchedule`



<pre><code><b>public</b> <b>struct</b> <a href="../rtd_system/genesis.md#rtd_system_genesis_TokenDistributionSchedule">TokenDistributionSchedule</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>stake_subsidy_fund_mist: u64</code>
</dt>
<dd>
</dd>
<dt>
<code>allocations: vector&lt;<a href="../rtd_system/genesis.md#rtd_system_genesis_TokenAllocation">rtd_system::genesis::TokenAllocation</a>&gt;</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="rtd_system_genesis_TokenAllocation"></a>

## Struct `TokenAllocation`



<pre><code><b>public</b> <b>struct</b> <a href="../rtd_system/genesis.md#rtd_system_genesis_TokenAllocation">TokenAllocation</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>recipient_address: <b>address</b></code>
</dt>
<dd>
</dd>
<dt>
<code>amount_mist: u64</code>
</dt>
<dd>
</dd>
<dt>
<code>staked_with_validator: <a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;<b>address</b>&gt;</code>
</dt>
<dd>
 Indicates if this allocation should be staked at genesis and with which validator
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="rtd_system_genesis_ENotCalledAtGenesis"></a>

The <code><a href="../rtd_system/genesis.md#rtd_system_genesis_create">create</a></code> function was called at a non-genesis epoch.


<pre><code><b>const</b> <a href="../rtd_system/genesis.md#rtd_system_genesis_ENotCalledAtGenesis">ENotCalledAtGenesis</a>: u64 = 0;
</code></pre>



<a name="rtd_system_genesis_EDuplicateValidator"></a>

The <code><a href="../rtd_system/genesis.md#rtd_system_genesis_create">create</a></code> function was called with duplicate validators.


<pre><code><b>const</b> <a href="../rtd_system/genesis.md#rtd_system_genesis_EDuplicateValidator">EDuplicateValidator</a>: u64 = 1;
</code></pre>



<a name="rtd_system_genesis_create"></a>

## Function `create`

This function will be explicitly called once at genesis.
It will create a singleton RtdSystemState object, which contains
all the information we need in the system.


<pre><code><b>fun</b> <a href="../rtd_system/genesis.md#rtd_system_genesis_create">create</a>(rtd_system_state_id: <a href="../rtd/object.md#rtd_object_UID">rtd::object::UID</a>, rtd_supply: <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;<a href="../rtd/rtd.md#rtd_rtd_RTD">rtd::rtd::RTD</a>&gt;, genesis_chain_parameters: <a href="../rtd_system/genesis.md#rtd_system_genesis_GenesisChainParameters">rtd_system::genesis::GenesisChainParameters</a>, genesis_validators: vector&lt;<a href="../rtd_system/genesis.md#rtd_system_genesis_GenesisValidatorMetadata">rtd_system::genesis::GenesisValidatorMetadata</a>&gt;, token_distribution_schedule: <a href="../rtd_system/genesis.md#rtd_system_genesis_TokenDistributionSchedule">rtd_system::genesis::TokenDistributionSchedule</a>, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../rtd_system/genesis.md#rtd_system_genesis_create">create</a>(
    rtd_system_state_id: UID,
    <b>mut</b> rtd_supply: Balance&lt;RTD&gt;,
    genesis_chain_parameters: <a href="../rtd_system/genesis.md#rtd_system_genesis_GenesisChainParameters">GenesisChainParameters</a>,
    genesis_validators: vector&lt;<a href="../rtd_system/genesis.md#rtd_system_genesis_GenesisValidatorMetadata">GenesisValidatorMetadata</a>&gt;,
    token_distribution_schedule: <a href="../rtd_system/genesis.md#rtd_system_genesis_TokenDistributionSchedule">TokenDistributionSchedule</a>,
    ctx: &<b>mut</b> TxContext,
) {
    // Ensure this is only called at <a href="../rtd_system/genesis.md#rtd_system_genesis">genesis</a>
    <b>assert</b>!(ctx.epoch() == 0, <a href="../rtd_system/genesis.md#rtd_system_genesis_ENotCalledAtGenesis">ENotCalledAtGenesis</a>);
    // Create all the `Validator` structs
    <b>let</b> <b>mut</b> validators = vector[];
    genesis_validators.do!(|genesis_validator| {
        <b>let</b> <a href="../rtd_system/genesis.md#rtd_system_genesis_GenesisValidatorMetadata">GenesisValidatorMetadata</a> {
            name,
            description,
            image_url,
            project_url,
            rtd_address,
            gas_price,
            commission_rate,
            protocol_public_key,
            proof_of_possession,
            network_public_key,
            worker_public_key,
            network_address,
            p2p_address,
            primary_address,
            worker_address,
        } = genesis_validator;
        <b>let</b> <a href="../rtd_system/validator.md#rtd_system_validator">validator</a> = <a href="../rtd_system/validator.md#rtd_system_validator_new">validator::new</a>(
            rtd_address,
            protocol_public_key,
            network_public_key,
            worker_public_key,
            proof_of_possession,
            name,
            description,
            image_url,
            project_url,
            network_address,
            p2p_address,
            primary_address,
            worker_address,
            gas_price,
            commission_rate,
            ctx,
        );
        // Ensure that each <a href="../rtd_system/validator.md#rtd_system_validator">validator</a> is unique
        <b>assert</b>!(
            !<a href="../rtd_system/validator_set.md#rtd_system_validator_set_is_duplicate_validator">validator_set::is_duplicate_validator</a>(&validators, &<a href="../rtd_system/validator.md#rtd_system_validator">validator</a>),
            <a href="../rtd_system/genesis.md#rtd_system_genesis_EDuplicateValidator">EDuplicateValidator</a>,
        );
        validators.push_back(<a href="../rtd_system/validator.md#rtd_system_validator">validator</a>);
    });
    <b>let</b> <a href="../rtd_system/genesis.md#rtd_system_genesis_TokenDistributionSchedule">TokenDistributionSchedule</a> {
        stake_subsidy_fund_mist,
        allocations,
    } = token_distribution_schedule;
    <b>let</b> subsidy_fund = rtd_supply.split(stake_subsidy_fund_mist);
    <b>let</b> <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund">storage_fund</a> = balance::zero();
    // Allocate tokens and staking operations
    <a href="../rtd_system/genesis.md#rtd_system_genesis_allocate_tokens">allocate_tokens</a>(rtd_supply, allocations, &<b>mut</b> validators, ctx);
    // Activate all validators
    validators.do_mut!(|<a href="../rtd_system/validator.md#rtd_system_validator">validator</a>| <a href="../rtd_system/validator.md#rtd_system_validator">validator</a>.activate(0));
    <b>let</b> system_parameters = <a href="../rtd_system/sui_system_state_inner.md#rtd_system_rtd_system_state_inner_create_system_parameters">rtd_system_state_inner::create_system_parameters</a>(
        genesis_chain_parameters.epoch_duration_ms,
        genesis_chain_parameters.stake_subsidy_start_epoch,
        // Validator committee parameters
        genesis_chain_parameters.max_validator_count,
        genesis_chain_parameters.min_validator_joining_stake,
        genesis_chain_parameters.validator_low_stake_threshold,
        genesis_chain_parameters.validator_very_low_stake_threshold,
        genesis_chain_parameters.validator_low_stake_grace_period,
        ctx,
    );
    <b>let</b> <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy">stake_subsidy</a> = <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy_create">stake_subsidy::create</a>(
        subsidy_fund,
        genesis_chain_parameters.stake_subsidy_initial_distribution_amount,
        genesis_chain_parameters.stake_subsidy_period_length,
        genesis_chain_parameters.stake_subsidy_decrease_rate,
        ctx,
    );
    rtd_system::create(
        rtd_system_state_id,
        validators,
        <a href="../rtd_system/storage_fund.md#rtd_system_storage_fund">storage_fund</a>,
        genesis_chain_parameters.protocol_version,
        genesis_chain_parameters.chain_start_timestamp_ms,
        system_parameters,
        <a href="../rtd_system/stake_subsidy.md#rtd_system_stake_subsidy">stake_subsidy</a>,
        ctx,
    );
}
</code></pre>



</details>

<a name="rtd_system_genesis_allocate_tokens"></a>

## Function `allocate_tokens`



<pre><code><b>fun</b> <a href="../rtd_system/genesis.md#rtd_system_genesis_allocate_tokens">allocate_tokens</a>(rtd_supply: <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;<a href="../rtd/rtd.md#rtd_rtd_RTD">rtd::rtd::RTD</a>&gt;, allocations: vector&lt;<a href="../rtd_system/genesis.md#rtd_system_genesis_TokenAllocation">rtd_system::genesis::TokenAllocation</a>&gt;, validators: &<b>mut</b> vector&lt;<a href="../rtd_system/validator.md#rtd_system_validator_Validator">rtd_system::validator::Validator</a>&gt;, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../rtd_system/genesis.md#rtd_system_genesis_allocate_tokens">allocate_tokens</a>(
    <b>mut</b> rtd_supply: Balance&lt;RTD&gt;,
    allocations: vector&lt;<a href="../rtd_system/genesis.md#rtd_system_genesis_TokenAllocation">TokenAllocation</a>&gt;,
    validators: &<b>mut</b> vector&lt;Validator&gt;,
    ctx: &<b>mut</b> TxContext,
) {
    allocations.destroy!(
        |<a href="../rtd_system/genesis.md#rtd_system_genesis_TokenAllocation">TokenAllocation</a> { recipient_address, amount_mist, staked_with_validator }| {
            <b>let</b> allocation_balance = rtd_supply.split(amount_mist);
            <b>if</b> (staked_with_validator.is_some()) {
                <b>let</b> validator_address = staked_with_validator.destroy_some();
                <b>let</b> <a href="../rtd_system/validator.md#rtd_system_validator">validator</a> = <a href="../rtd_system/validator_set.md#rtd_system_validator_set_get_validator_mut">validator_set::get_validator_mut</a>(validators, validator_address);
                <a href="../rtd_system/validator.md#rtd_system_validator">validator</a>.request_add_stake_at_genesis(
                    allocation_balance,
                    recipient_address,
                    ctx,
                );
            } <b>else</b> {
                transfer::public_transfer(allocation_balance.into_coin(ctx), recipient_address);
            };
        },
    );
    // should be none left at this point.
    // Provided allocations must fully allocate the rtd_supply and there
    rtd_supply.destroy_zero();
}
</code></pre>



</details>
