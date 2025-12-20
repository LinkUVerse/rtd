---
title: Module `rtd::coin_registry`
---

Defines the system object for managing coin data in a central
registry. This module provides a centralized way to store and manage
metadata for all currencies in the Rtd ecosystem, including their
supply information, regulatory status, and metadata capabilities.


-  [Struct `CoinRegistry`](#rtd_coin_registry_CoinRegistry)
-  [Struct `ExtraField`](#rtd_coin_registry_ExtraField)
-  [Struct `CurrencyKey`](#rtd_coin_registry_CurrencyKey)
-  [Struct `LegacyMetadataKey`](#rtd_coin_registry_LegacyMetadataKey)
-  [Struct `MetadataCap`](#rtd_coin_registry_MetadataCap)
-  [Struct `Borrow`](#rtd_coin_registry_Borrow)
-  [Struct `Currency`](#rtd_coin_registry_Currency)
-  [Struct `CurrencyInitializer`](#rtd_coin_registry_CurrencyInitializer)
-  [Enum `SupplyState`](#rtd_coin_registry_SupplyState)
-  [Enum `RegulatedState`](#rtd_coin_registry_RegulatedState)
-  [Enum `MetadataCapState`](#rtd_coin_registry_MetadataCapState)
-  [Constants](#@Constants_0)
-  [Function `new_currency`](#rtd_coin_registry_new_currency)
-  [Function `new_currency_with_otw`](#rtd_coin_registry_new_currency_with_otw)
-  [Function `claim_metadata_cap`](#rtd_coin_registry_claim_metadata_cap)
-  [Function `make_regulated`](#rtd_coin_registry_make_regulated)
-  [Function `make_supply_fixed_init`](#rtd_coin_registry_make_supply_fixed_init)
-  [Function `make_supply_burn_only_init`](#rtd_coin_registry_make_supply_burn_only_init)
-  [Function `make_supply_fixed`](#rtd_coin_registry_make_supply_fixed)
-  [Function `make_supply_burn_only`](#rtd_coin_registry_make_supply_burn_only)
-  [Function `finalize`](#rtd_coin_registry_finalize)
-  [Function `finalize_and_delete_metadata_cap`](#rtd_coin_registry_finalize_and_delete_metadata_cap)
-  [Function `finalize_registration`](#rtd_coin_registry_finalize_registration)
-  [Function `delete_metadata_cap`](#rtd_coin_registry_delete_metadata_cap)
-  [Function `burn`](#rtd_coin_registry_burn)
-  [Function `burn_balance`](#rtd_coin_registry_burn_balance)
-  [Function `set_name`](#rtd_coin_registry_set_name)
-  [Function `set_description`](#rtd_coin_registry_set_description)
-  [Function `set_icon_url`](#rtd_coin_registry_set_icon_url)
-  [Function `set_treasury_cap_id`](#rtd_coin_registry_set_treasury_cap_id)
-  [Function `migrate_legacy_metadata`](#rtd_coin_registry_migrate_legacy_metadata)
-  [Function `update_from_legacy_metadata`](#rtd_coin_registry_update_from_legacy_metadata)
-  [Function `delete_migrated_legacy_metadata`](#rtd_coin_registry_delete_migrated_legacy_metadata)
-  [Function `migrate_regulated_state_by_metadata`](#rtd_coin_registry_migrate_regulated_state_by_metadata)
-  [Function `migrate_regulated_state_by_cap`](#rtd_coin_registry_migrate_regulated_state_by_cap)
-  [Function `borrow_legacy_metadata`](#rtd_coin_registry_borrow_legacy_metadata)
-  [Function `return_borrowed_legacy_metadata`](#rtd_coin_registry_return_borrowed_legacy_metadata)
-  [Function `decimals`](#rtd_coin_registry_decimals)
-  [Function `name`](#rtd_coin_registry_name)
-  [Function `symbol`](#rtd_coin_registry_symbol)
-  [Function `description`](#rtd_coin_registry_description)
-  [Function `icon_url`](#rtd_coin_registry_icon_url)
-  [Function `is_metadata_cap_claimed`](#rtd_coin_registry_is_metadata_cap_claimed)
-  [Function `is_metadata_cap_deleted`](#rtd_coin_registry_is_metadata_cap_deleted)
-  [Function `metadata_cap_id`](#rtd_coin_registry_metadata_cap_id)
-  [Function `treasury_cap_id`](#rtd_coin_registry_treasury_cap_id)
-  [Function `deny_cap_id`](#rtd_coin_registry_deny_cap_id)
-  [Function `is_supply_fixed`](#rtd_coin_registry_is_supply_fixed)
-  [Function `is_supply_burn_only`](#rtd_coin_registry_is_supply_burn_only)
-  [Function `is_regulated`](#rtd_coin_registry_is_regulated)
-  [Function `total_supply`](#rtd_coin_registry_total_supply)
-  [Function `exists`](#rtd_coin_registry_exists)
-  [Function `is_migrated_from_legacy`](#rtd_coin_registry_is_migrated_from_legacy)
-  [Function `to_legacy_metadata`](#rtd_coin_registry_to_legacy_metadata)
-  [Function `create`](#rtd_coin_registry_create)
-  [Macro function `finalize_impl`](#rtd_coin_registry_finalize_impl)
-  [Macro function `migrate_legacy_metadata_impl`](#rtd_coin_registry_migrate_legacy_metadata_impl)
-  [Macro function `is_ascii_printable`](#rtd_coin_registry_is_ascii_printable)


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
<b>use</b> <a href="../rtd/derived_object.md#rtd_derived_object">rtd::derived_object</a>;
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



<a name="rtd_coin_registry_CoinRegistry"></a>

## Struct `CoinRegistry`

System object found at address <code>0xc</code> that stores coin data for all
registered coin types. This is a shared object that acts as a central
registry for coin metadata, supply information, and regulatory status.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">CoinRegistry</a> <b>has</b> key
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

<a name="rtd_coin_registry_ExtraField"></a>

## Struct `ExtraField`

Store only object that enables more flexible coin data
registration, allowing for additional fields to be added
without changing the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code> structure.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_ExtraField">ExtraField</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>0: <a href="../std/type_name.md#std_type_name_TypeName">std::type_name::TypeName</a></code>
</dt>
<dd>
</dd>
<dt>
<code>1: vector&lt;u8&gt;</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="rtd_coin_registry_CurrencyKey"></a>

## Struct `CurrencyKey`

Key used to derive addresses when creating <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;</code> objects.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyKey">CurrencyKey</a>&lt;<b>phantom</b> T&gt; <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
</dl>


</details>

<a name="rtd_coin_registry_LegacyMetadataKey"></a>

## Struct `LegacyMetadataKey`

Key used to store the legacy <code>CoinMetadata</code> for a <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code>.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_LegacyMetadataKey">LegacyMetadataKey</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
</dl>


</details>

<a name="rtd_coin_registry_MetadataCap"></a>

## Struct `MetadataCap`

Capability object that gates metadata (name, description, icon_url, symbol)
changes in the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code>. It can only be created (or claimed) once, and can
be deleted to prevent changes to the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code> metadata.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a>&lt;<b>phantom</b> T&gt; <b>has</b> key, store
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

<a name="rtd_coin_registry_Borrow"></a>

## Struct `Borrow`

Potato callback for the legacy <code>CoinMetadata</code> borrowing.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Borrow">Borrow</a>&lt;<b>phantom</b> T&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
</dl>


</details>

<a name="rtd_coin_registry_Currency"></a>

## Struct `Currency`

Currency stores metadata such as name, symbol, decimals, icon_url and description,
as well as supply states (optional) and regulatory status.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;<b>phantom</b> T&gt; <b>has</b> key
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
<code><a href="../rtd/coin_registry.md#rtd_coin_registry_decimals">decimals</a>: u8</code>
</dt>
<dd>
 Number of decimal places the coin uses for display purposes.
</dd>
<dt>
<code><a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>: <a href="../std/string.md#std_string_String">std::string::String</a></code>
</dt>
<dd>
 Human-readable name for the coin.
</dd>
<dt>
<code><a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>: <a href="../std/string.md#std_string_String">std::string::String</a></code>
</dt>
<dd>
 Short symbol/ticker for the coin.
</dd>
<dt>
<code><a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>: <a href="../std/string.md#std_string_String">std::string::String</a></code>
</dt>
<dd>
 Detailed description of the coin.
</dd>
<dt>
<code><a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>: <a href="../std/string.md#std_string_String">std::string::String</a></code>
</dt>
<dd>
 URL for the coin's icon/logo.
</dd>
<dt>
<code>supply: <a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;<a href="../rtd/coin_registry.md#rtd_coin_registry_SupplyState">rtd::coin_registry::SupplyState</a>&lt;T&gt;&gt;</code>
</dt>
<dd>
 Current supply state of the coin (fixed supply or unknown)
 Note: We're using <code>Option</code> because <code><a href="../rtd/coin_registry.md#rtd_coin_registry_SupplyState">SupplyState</a></code> does not have drop,
 meaning we cannot swap out its value at a later state.
</dd>
<dt>
<code>regulated: <a href="../rtd/coin_registry.md#rtd_coin_registry_RegulatedState">rtd::coin_registry::RegulatedState</a></code>
</dt>
<dd>
 Regulatory status of the coin (regulated with deny cap or unknown)
</dd>
<dt>
<code><a href="../rtd/coin_registry.md#rtd_coin_registry_treasury_cap_id">treasury_cap_id</a>: <a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;<a href="../rtd/object.md#rtd_object_ID">rtd::object::ID</a>&gt;</code>
</dt>
<dd>
 ID of the treasury cap for this coin type, if registered.
</dd>
<dt>
<code><a href="../rtd/coin_registry.md#rtd_coin_registry_metadata_cap_id">metadata_cap_id</a>: <a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCapState">rtd::coin_registry::MetadataCapState</a></code>
</dt>
<dd>
 ID of the metadata capability for this coin type, if claimed.
</dd>
<dt>
<code>extra_fields: <a href="../rtd/vec_map.md#rtd_vec_map_VecMap">rtd::vec_map::VecMap</a>&lt;<a href="../std/string.md#std_string_String">std::string::String</a>, <a href="../rtd/coin_registry.md#rtd_coin_registry_ExtraField">rtd::coin_registry::ExtraField</a>&gt;</code>
</dt>
<dd>
 Additional fields for extensibility.
</dd>
</dl>


</details>

<a name="rtd_coin_registry_CurrencyInitializer"></a>

## Struct `CurrencyInitializer`

Hot potato wrapper to enforce registration after "new_currency" data creation.
Destroyed in the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_finalize">finalize</a></code> call and either transferred to the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">CoinRegistry</a></code>
(in case of an OTW registration) or shared directly (for dynamically created
currencies).


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">CurrencyInitializer</a>&lt;<b>phantom</b> T&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>currency: <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>extra_fields: <a href="../rtd/bag.md#rtd_bag_Bag">rtd::bag::Bag</a></code>
</dt>
<dd>
</dd>
<dt>
<code>is_otw: bool</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="rtd_coin_registry_SupplyState"></a>

## Enum `SupplyState`

Supply state marks the type of Currency Supply, which can be
- Fixed: no minting or burning;
- BurnOnly: no minting, burning is allowed;
- Unknown: flexible (supply is controlled by its <code>TreasuryCap</code>);


<pre><code><b>public</b> <b>enum</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_SupplyState">SupplyState</a>&lt;<b>phantom</b> T&gt; <b>has</b> store
</code></pre>



<details>
<summary>Variants</summary>


<dl>
<dt>
Variant <code>Fixed</code>
</dt>
<dd>
 Coin has a fixed supply with the given Supply object.
</dd>

<dl>
<dt>
<code>0: <a href="../rtd/balance.md#rtd_balance_Supply">rtd::balance::Supply</a>&lt;T&gt;</code>
</dt>
<dd>
</dd>
</dl>

<dt>
Variant <code>BurnOnly</code>
</dt>
<dd>
 Coin has a supply that can ONLY decrease.
</dd>

<dl>
<dt>
<code>0: <a href="../rtd/balance.md#rtd_balance_Supply">rtd::balance::Supply</a>&lt;T&gt;</code>
</dt>
<dd>
</dd>
</dl>

<dt>
Variant <code>Unknown</code>
</dt>
<dd>
 Supply information is not yet known or registered.
</dd>
</dl>


</details>

<a name="rtd_coin_registry_RegulatedState"></a>

## Enum `RegulatedState`

Regulated state of a coin type.
- Regulated: <code>DenyCap</code> exists or a <code>RegulatedCoinMetadata</code> used to mark currency as regulated;
- Unregulated: the currency was created without deny list;
- Unknown: the regulatory status is unknown.


<pre><code><b>public</b> <b>enum</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_RegulatedState">RegulatedState</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Variants</summary>


<dl>
<dt>
Variant <code>Regulated</code>
</dt>
<dd>
 Coin is regulated with a deny cap for address restrictions.
 <code>allow_global_pause</code> is <code>None</code> if the information is unknown (has not been migrated from <code>DenyCapV2</code>).
</dd>

<dl>
<dt>
<code>cap: <a href="../rtd/object.md#rtd_object_ID">rtd::object::ID</a></code>
</dt>
<dd>
</dd>
</dl>


<dl>
<dt>
<code>allow_global_pause: <a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;bool&gt;</code>
</dt>
<dd>
</dd>
</dl>


<dl>
<dt>
<code>variant: u8</code>
</dt>
<dd>
</dd>
</dl>

<dt>
Variant <code>Unregulated</code>
</dt>
<dd>
 The coin has been created without deny list.
</dd>
<dt>
Variant <code>Unknown</code>
</dt>
<dd>
 Regulatory status is unknown.
 Result of a legacy migration for that coin (from <code><a href="../rtd/coin.md#rtd_coin">coin</a>.<b>move</b></code> constructors)
</dd>
</dl>


</details>

<a name="rtd_coin_registry_MetadataCapState"></a>

## Enum `MetadataCapState`

State of the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a></code> for a single <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code>.


<pre><code><b>public</b> <b>enum</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCapState">MetadataCapState</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Variants</summary>


<dl>
<dt>
Variant <code>Claimed</code>
</dt>
<dd>
 The metadata cap has been claimed.
</dd>

<dl>
<dt>
<code>0: <a href="../rtd/object.md#rtd_object_ID">rtd::object::ID</a></code>
</dt>
<dd>
</dd>
</dl>

<dt>
Variant <code>Unclaimed</code>
</dt>
<dd>
 The metadata cap has not been claimed.
</dd>
<dt>
Variant <code>Deleted</code>
</dt>
<dd>
 The metadata cap has been claimed and then deleted.
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="rtd_coin_registry_EMetadataCapAlreadyClaimed"></a>

Metadata cap already claimed


<pre><code>#[error]
<b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_EMetadataCapAlreadyClaimed">EMetadataCapAlreadyClaimed</a>: vector&lt;u8&gt; = b"Metadata cap already claimed";
</code></pre>



<a name="rtd_coin_registry_ENotSystemAddress"></a>

Only the system address can create the registry


<pre><code>#[error]
<b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_ENotSystemAddress">ENotSystemAddress</a>: vector&lt;u8&gt; = b"Only the system can <a href="../rtd/coin_registry.md#rtd_coin_registry_create">create</a> the registry.";
</code></pre>



<a name="rtd_coin_registry_ECurrencyAlreadyExists"></a>

Currency for this coin type already exists


<pre><code>#[error]
<b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_ECurrencyAlreadyExists">ECurrencyAlreadyExists</a>: vector&lt;u8&gt; = b"<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a> <b>for</b> this <a href="../rtd/coin.md#rtd_coin">coin</a> type already <a href="../rtd/coin_registry.md#rtd_coin_registry_exists">exists</a>.";
</code></pre>



<a name="rtd_coin_registry_EDenyListStateAlreadySet"></a>

Attempt to set the deny list state permissionlessly while it has already been set.


<pre><code>#[error]
<b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_EDenyListStateAlreadySet">EDenyListStateAlreadySet</a>: vector&lt;u8&gt; = b"Cannot set the deny list state <b>as</b> it <b>has</b> already been set.";
</code></pre>



<a name="rtd_coin_registry_ECannotUpdateManagedMetadata"></a>

Attempt to update <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code> with legacy metadata after the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a></code> has
been claimed. Updates are only allowed if the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a></code> has not yet been
claimed or deleted.


<pre><code>#[error]
<b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_ECannotUpdateManagedMetadata">ECannotUpdateManagedMetadata</a>: vector&lt;u8&gt; = b"Cannot update metadata whose `<a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a>` <b>has</b> already been claimed";
</code></pre>



<a name="rtd_coin_registry_EInvalidSymbol"></a>

Attempt to set the symbol to a non-ASCII printable character


<pre><code>#[error]
<b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_EInvalidSymbol">EInvalidSymbol</a>: vector&lt;u8&gt; = b"Symbol <b>has</b> to be ASCII printable";
</code></pre>



<a name="rtd_coin_registry_EDenyCapAlreadyCreated"></a>



<pre><code>#[error]
<b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_EDenyCapAlreadyCreated">EDenyCapAlreadyCreated</a>: vector&lt;u8&gt; = b"Cannot claim the deny cap twice";
</code></pre>



<a name="rtd_coin_registry_ECurrencyAlreadyRegistered"></a>

Attempt to migrate legacy metadata for a <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code> that already exists.


<pre><code>#[error]
<b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_ECurrencyAlreadyRegistered">ECurrencyAlreadyRegistered</a>: vector&lt;u8&gt; = b"<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a> already registered";
</code></pre>



<a name="rtd_coin_registry_EEmptySupply"></a>



<pre><code>#[error]
<b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_EEmptySupply">EEmptySupply</a>: vector&lt;u8&gt; = b"Supply cannot be empty";
</code></pre>



<a name="rtd_coin_registry_ESupplyNotBurnOnly"></a>



<pre><code>#[error]
<b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_ESupplyNotBurnOnly">ESupplyNotBurnOnly</a>: vector&lt;u8&gt; = b"Cannot <a href="../rtd/coin_registry.md#rtd_coin_registry_burn">burn</a> on a non <a href="../rtd/coin_registry.md#rtd_coin_registry_burn">burn</a>-only supply";
</code></pre>



<a name="rtd_coin_registry_EInvariantViolation"></a>



<pre><code>#[error]
<b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_EInvariantViolation">EInvariantViolation</a>: vector&lt;u8&gt; = b"Code <b>invariant</b> violation";
</code></pre>



<a name="rtd_coin_registry_EDeletionNotSupported"></a>



<pre><code>#[error]
<b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_EDeletionNotSupported">EDeletionNotSupported</a>: vector&lt;u8&gt; = b"Deleting legacy metadata is not supported";
</code></pre>



<a name="rtd_coin_registry_ENotOneTimeWitness"></a>



<pre><code>#[error]
<b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_ENotOneTimeWitness">ENotOneTimeWitness</a>: vector&lt;u8&gt; = b"Type is expected to be OTW";
</code></pre>



<a name="rtd_coin_registry_EBorrowLegacyMetadata"></a>



<pre><code>#[error]
<b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_EBorrowLegacyMetadata">EBorrowLegacyMetadata</a>: vector&lt;u8&gt; = b"Cannot <a href="../rtd/borrow.md#rtd_borrow">borrow</a> legacy metadata <b>for</b> migrated currency";
</code></pre>



<a name="rtd_coin_registry_EDuplicateBorrow"></a>



<pre><code>#[error]
<b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_EDuplicateBorrow">EDuplicateBorrow</a>: vector&lt;u8&gt; = b"Attempt to <b>return</b> duplicate borrowed CoinMetadata";
</code></pre>



<a name="rtd_coin_registry_REGULATED_COIN_VERSION"></a>

Incremental identifier for regulated coin versions in the deny list.
We start from <code>0</code> in the new system, which aligns with the state of <code>DenyCapV2</code>.


<pre><code><b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_REGULATED_COIN_VERSION">REGULATED_COIN_VERSION</a>: u8 = 0;
</code></pre>



<a name="rtd_coin_registry_NEW_CURRENCY_MARKER"></a>

Marker used in metadata to indicate that the currency is not migrated.


<pre><code><b>const</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_NEW_CURRENCY_MARKER">NEW_CURRENCY_MARKER</a>: vector&lt;u8&gt; = vector[105, 115, 95, 110, 101, 119, 95, 99, 117, 114, 114, 101, 110, 99, 121];
</code></pre>



<a name="rtd_coin_registry_new_currency"></a>

## Function `new_currency`

Creates a new currency.

Note: This constructor has no long term difference from <code><a href="../rtd/coin_registry.md#rtd_coin_registry_new_currency_with_otw">new_currency_with_otw</a></code>.
This can be called from the module that defines <code>T</code> any time after it has been published.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_new_currency">new_currency</a>&lt;T: key&gt;(registry: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">rtd::coin_registry::CoinRegistry</a>, <a href="../rtd/coin_registry.md#rtd_coin_registry_decimals">decimals</a>: u8, <a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>: <a href="../std/string.md#std_string_String">std::string::String</a>, <a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>: <a href="../std/string.md#std_string_String">std::string::String</a>, <a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>: <a href="../std/string.md#std_string_String">std::string::String</a>, <a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>: <a href="../std/string.md#std_string_String">std::string::String</a>, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): (<a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">rtd::coin_registry::CurrencyInitializer</a>&lt;T&gt;, <a href="../rtd/coin.md#rtd_coin_TreasuryCap">rtd::coin::TreasuryCap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_new_currency">new_currency</a>&lt;T: /* internal */ key&gt;(
    registry: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">CoinRegistry</a>,
    <a href="../rtd/coin_registry.md#rtd_coin_registry_decimals">decimals</a>: u8,
    <a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>: String,
    <a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>: String,
    <a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>: String,
    <a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>: String,
    ctx: &<b>mut</b> TxContext,
): (<a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">CurrencyInitializer</a>&lt;T&gt;, TreasuryCap&lt;T&gt;) {
    <b>assert</b>!(!registry.<a href="../rtd/coin_registry.md#rtd_coin_registry_exists">exists</a>&lt;T&gt;(), <a href="../rtd/coin_registry.md#rtd_coin_registry_ECurrencyAlreadyExists">ECurrencyAlreadyExists</a>);
    <b>assert</b>!(<a href="../rtd/coin_registry.md#rtd_coin_registry_is_ascii_printable">is_ascii_printable</a>!(&<a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>), <a href="../rtd/coin_registry.md#rtd_coin_registry_EInvalidSymbol">EInvalidSymbol</a>);
    <b>let</b> treasury_cap = <a href="../rtd/coin.md#rtd_coin_new_treasury_cap">coin::new_treasury_cap</a>(ctx);
    <b>let</b> currency = <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt; {
        id: <a href="../rtd/derived_object.md#rtd_derived_object_claim">derived_object::claim</a>(&<b>mut</b> registry.id, <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyKey">CurrencyKey</a>&lt;T&gt;()),
        <a href="../rtd/coin_registry.md#rtd_coin_registry_decimals">decimals</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>,
        supply: option::some(SupplyState::Unknown),
        regulated: RegulatedState::Unregulated,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_treasury_cap_id">treasury_cap_id</a>: option::some(<a href="../rtd/object.md#rtd_object_id">object::id</a>(&treasury_cap)),
        <a href="../rtd/coin_registry.md#rtd_coin_registry_metadata_cap_id">metadata_cap_id</a>: MetadataCapState::Unclaimed,
        extra_fields: <a href="../rtd/vec_map.md#rtd_vec_map_empty">vec_map::empty</a>(),
    };
    (<a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">CurrencyInitializer</a> { currency, is_otw: <b>false</b>, extra_fields: <a href="../rtd/bag.md#rtd_bag_new">bag::new</a>(ctx) }, treasury_cap)
}
</code></pre>



</details>

<a name="rtd_coin_registry_new_currency_with_otw"></a>

## Function `new_currency_with_otw`

Creates a new currency with using an OTW as proof of uniqueness.

This is a two-step operation:
1. <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code> is constructed in the <code>init</code> function and sent to the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">CoinRegistry</a></code>;
2. <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code> is promoted to a shared object in the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_finalize_registration">finalize_registration</a></code> call;


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_new_currency_with_otw">new_currency_with_otw</a>&lt;T: drop&gt;(otw: T, <a href="../rtd/coin_registry.md#rtd_coin_registry_decimals">decimals</a>: u8, <a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>: <a href="../std/string.md#std_string_String">std::string::String</a>, <a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>: <a href="../std/string.md#std_string_String">std::string::String</a>, <a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>: <a href="../std/string.md#std_string_String">std::string::String</a>, <a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>: <a href="../std/string.md#std_string_String">std::string::String</a>, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): (<a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">rtd::coin_registry::CurrencyInitializer</a>&lt;T&gt;, <a href="../rtd/coin.md#rtd_coin_TreasuryCap">rtd::coin::TreasuryCap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_new_currency_with_otw">new_currency_with_otw</a>&lt;T: drop&gt;(
    otw: T,
    <a href="../rtd/coin_registry.md#rtd_coin_registry_decimals">decimals</a>: u8,
    <a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>: String,
    <a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>: String,
    <a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>: String,
    <a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>: String,
    ctx: &<b>mut</b> TxContext,
): (<a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">CurrencyInitializer</a>&lt;T&gt;, TreasuryCap&lt;T&gt;) {
    <b>assert</b>!(<a href="../rtd/types.md#rtd_types_is_one_time_witness">rtd::types::is_one_time_witness</a>(&otw), <a href="../rtd/coin_registry.md#rtd_coin_registry_ENotOneTimeWitness">ENotOneTimeWitness</a>);
    <b>assert</b>!(<a href="../rtd/coin_registry.md#rtd_coin_registry_is_ascii_printable">is_ascii_printable</a>!(&<a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>), <a href="../rtd/coin_registry.md#rtd_coin_registry_EInvalidSymbol">EInvalidSymbol</a>);
    <b>let</b> treasury_cap = <a href="../rtd/coin.md#rtd_coin_new_treasury_cap">coin::new_treasury_cap</a>(ctx);
    <b>let</b> currency = <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt; {
        id: <a href="../rtd/object.md#rtd_object_new">object::new</a>(ctx),
        <a href="../rtd/coin_registry.md#rtd_coin_registry_decimals">decimals</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>,
        supply: option::some(SupplyState::Unknown),
        regulated: RegulatedState::Unregulated,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_treasury_cap_id">treasury_cap_id</a>: option::some(<a href="../rtd/object.md#rtd_object_id">object::id</a>(&treasury_cap)),
        <a href="../rtd/coin_registry.md#rtd_coin_registry_metadata_cap_id">metadata_cap_id</a>: MetadataCapState::Unclaimed,
        extra_fields: <a href="../rtd/vec_map.md#rtd_vec_map_empty">vec_map::empty</a>(),
    };
    (<a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">CurrencyInitializer</a> { currency, is_otw: <b>true</b>, extra_fields: <a href="../rtd/bag.md#rtd_bag_new">bag::new</a>(ctx) }, treasury_cap)
}
</code></pre>



</details>

<a name="rtd_coin_registry_claim_metadata_cap"></a>

## Function `claim_metadata_cap`

Claim a <code><a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a></code> for a coin type.
Only allowed from the owner of <code>TreasuryCap</code>, and only once.

Aborts if the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a></code> has already been claimed.
Deleted <code><a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a></code> cannot be reclaimed.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_claim_metadata_cap">claim_metadata_cap</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, _: &<a href="../rtd/coin.md#rtd_coin_TreasuryCap">rtd::coin::TreasuryCap</a>&lt;T&gt;, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): <a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">rtd::coin_registry::MetadataCap</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_claim_metadata_cap">claim_metadata_cap</a>&lt;T&gt;(
    currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;,
    _: &TreasuryCap&lt;T&gt;,
    ctx: &<b>mut</b> TxContext,
): <a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a>&lt;T&gt; {
    <b>assert</b>!(!currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_is_metadata_cap_claimed">is_metadata_cap_claimed</a>(), <a href="../rtd/coin_registry.md#rtd_coin_registry_EMetadataCapAlreadyClaimed">EMetadataCapAlreadyClaimed</a>);
    <b>let</b> id = <a href="../rtd/object.md#rtd_object_new">object::new</a>(ctx);
    currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_metadata_cap_id">metadata_cap_id</a> = MetadataCapState::Claimed(id.to_inner());
    <a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a> { id }
}
</code></pre>



</details>

<a name="rtd_coin_registry_make_regulated"></a>

## Function `make_regulated`

Allows converting a currency, on init, to regulated, which creates
a <code>DenyCapV2</code> object, and a denylist entry. Sets regulated state to
<code>Regulated</code>.

This action is irreversible.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_make_regulated">make_regulated</a>&lt;T&gt;(init: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">rtd::coin_registry::CurrencyInitializer</a>&lt;T&gt;, allow_global_pause: bool, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): <a href="../rtd/coin.md#rtd_coin_DenyCapV2">rtd::coin::DenyCapV2</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_make_regulated">make_regulated</a>&lt;T&gt;(
    init: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">CurrencyInitializer</a>&lt;T&gt;,
    allow_global_pause: bool,
    ctx: &<b>mut</b> TxContext,
): DenyCapV2&lt;T&gt; {
    <b>assert</b>!(init.currency.regulated == RegulatedState::Unregulated, <a href="../rtd/coin_registry.md#rtd_coin_registry_EDenyCapAlreadyCreated">EDenyCapAlreadyCreated</a>);
    <b>let</b> deny_cap = <a href="../rtd/coin.md#rtd_coin_new_deny_cap_v2">coin::new_deny_cap_v2</a>&lt;T&gt;(allow_global_pause, ctx);
    init.currency.regulated =
        RegulatedState::Regulated {
            cap: <a href="../rtd/object.md#rtd_object_id">object::id</a>(&deny_cap),
            allow_global_pause: option::some(allow_global_pause),
            variant: <a href="../rtd/coin_registry.md#rtd_coin_registry_REGULATED_COIN_VERSION">REGULATED_COIN_VERSION</a>,
        };
    deny_cap
}
</code></pre>



</details>

<a name="rtd_coin_registry_make_supply_fixed_init"></a>

## Function `make_supply_fixed_init`

Initializer function to make the supply fixed.
Aborts if Supply is <code>0</code> to enforce minting during initialization.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_make_supply_fixed_init">make_supply_fixed_init</a>&lt;T&gt;(init: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">rtd::coin_registry::CurrencyInitializer</a>&lt;T&gt;, cap: <a href="../rtd/coin.md#rtd_coin_TreasuryCap">rtd::coin::TreasuryCap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_make_supply_fixed_init">make_supply_fixed_init</a>&lt;T&gt;(init: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">CurrencyInitializer</a>&lt;T&gt;, cap: TreasuryCap&lt;T&gt;) {
    <b>assert</b>!(cap.<a href="../rtd/coin_registry.md#rtd_coin_registry_total_supply">total_supply</a>() &gt; 0, <a href="../rtd/coin_registry.md#rtd_coin_registry_EEmptySupply">EEmptySupply</a>);
    init.currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_make_supply_fixed">make_supply_fixed</a>(cap)
}
</code></pre>



</details>

<a name="rtd_coin_registry_make_supply_burn_only_init"></a>

## Function `make_supply_burn_only_init`

Initializer function to make the supply burn-only.
Aborts if Supply is <code>0</code> to enforce minting during initialization.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_make_supply_burn_only_init">make_supply_burn_only_init</a>&lt;T&gt;(init: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">rtd::coin_registry::CurrencyInitializer</a>&lt;T&gt;, cap: <a href="../rtd/coin.md#rtd_coin_TreasuryCap">rtd::coin::TreasuryCap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_make_supply_burn_only_init">make_supply_burn_only_init</a>&lt;T&gt;(init: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">CurrencyInitializer</a>&lt;T&gt;, cap: TreasuryCap&lt;T&gt;) {
    <b>assert</b>!(cap.<a href="../rtd/coin_registry.md#rtd_coin_registry_total_supply">total_supply</a>() &gt; 0, <a href="../rtd/coin_registry.md#rtd_coin_registry_EEmptySupply">EEmptySupply</a>);
    init.currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_make_supply_burn_only">make_supply_burn_only</a>(cap)
}
</code></pre>



</details>

<a name="rtd_coin_registry_make_supply_fixed"></a>

## Function `make_supply_fixed`

Freeze the supply by destroying the <code>TreasuryCap</code> and storing it in the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_make_supply_fixed">make_supply_fixed</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, cap: <a href="../rtd/coin.md#rtd_coin_TreasuryCap">rtd::coin::TreasuryCap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_make_supply_fixed">make_supply_fixed</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;, cap: TreasuryCap&lt;T&gt;) {
    match (currency.supply.swap(SupplyState::Fixed(cap.into_supply()))) {
        // Impossible: We cannot fix a supply or make a supply <a href="../rtd/coin_registry.md#rtd_coin_registry_burn">burn</a>-only twice.
        SupplyState::Fixed(_supply) | SupplyState::BurnOnly(_supply) =&gt; <b>abort</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_EInvariantViolation">EInvariantViolation</a>,
        // We replaced "unknown" with fixed supply.
        SupplyState::Unknown =&gt; (),
    };
}
</code></pre>



</details>

<a name="rtd_coin_registry_make_supply_burn_only"></a>

## Function `make_supply_burn_only`

Make the supply <code>BurnOnly</code> by giving up the <code>TreasuryCap</code>, and allowing
burning of Coins through the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_make_supply_burn_only">make_supply_burn_only</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, cap: <a href="../rtd/coin.md#rtd_coin_TreasuryCap">rtd::coin::TreasuryCap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_make_supply_burn_only">make_supply_burn_only</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;, cap: TreasuryCap&lt;T&gt;) {
    match (currency.supply.swap(SupplyState::BurnOnly(cap.into_supply()))) {
        // Impossible: We cannot fix a supply or make a supply <a href="../rtd/coin_registry.md#rtd_coin_registry_burn">burn</a>-only twice.
        SupplyState::Fixed(_supply) | SupplyState::BurnOnly(_supply) =&gt; <b>abort</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_EInvariantViolation">EInvariantViolation</a>,
        // We replaced "unknown" with frozen supply.
        SupplyState::Unknown =&gt; (),
    };
}
</code></pre>



</details>

<a name="rtd_coin_registry_finalize"></a>

## Function `finalize`

Finalize the coin initialization, returning <code><a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_finalize">finalize</a>&lt;T&gt;(builder: <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">rtd::coin_registry::CurrencyInitializer</a>&lt;T&gt;, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): <a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">rtd::coin_registry::MetadataCap</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_finalize">finalize</a>&lt;T&gt;(builder: <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">CurrencyInitializer</a>&lt;T&gt;, ctx: &<b>mut</b> TxContext): <a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a>&lt;T&gt; {
    <b>let</b> is_otw = builder.is_otw;
    <b>let</b> (currency, metadata_cap) = <a href="../rtd/coin_registry.md#rtd_coin_registry_finalize_impl">finalize_impl</a>!(builder, ctx);
    // Either share directly (`<a href="../rtd/coin_registry.md#rtd_coin_registry_new_currency">new_currency</a>` scenario), or <a href="../rtd/transfer.md#rtd_transfer">transfer</a> <b>as</b> TTO to `<a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">CoinRegistry</a>`.
    <b>if</b> (is_otw) <a href="../rtd/transfer.md#rtd_transfer_transfer">transfer::transfer</a>(currency, <a href="../rtd/object.md#rtd_object_rtd_coin_registry_address">object::rtd_coin_registry_address</a>())
    <b>else</b> <a href="../rtd/transfer.md#rtd_transfer_share_object">transfer::share_object</a>(currency);
    metadata_cap
}
</code></pre>



</details>

<a name="rtd_coin_registry_finalize_and_delete_metadata_cap"></a>

## Function `finalize_and_delete_metadata_cap`

Does the same as <code><a href="../rtd/coin_registry.md#rtd_coin_registry_finalize">finalize</a></code>, but also deletes the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a></code> after finalization.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_finalize_and_delete_metadata_cap">finalize_and_delete_metadata_cap</a>&lt;T&gt;(builder: <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">rtd::coin_registry::CurrencyInitializer</a>&lt;T&gt;, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_finalize_and_delete_metadata_cap">finalize_and_delete_metadata_cap</a>&lt;T&gt;(
    builder: <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">CurrencyInitializer</a>&lt;T&gt;,
    ctx: &<b>mut</b> TxContext,
) {
    <b>let</b> is_otw = builder.is_otw;
    <b>let</b> (<b>mut</b> currency, metadata_cap) = <a href="../rtd/coin_registry.md#rtd_coin_registry_finalize_impl">finalize_impl</a>!(builder, ctx);
    currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_delete_metadata_cap">delete_metadata_cap</a>(metadata_cap);
    // Either share directly (`<a href="../rtd/coin_registry.md#rtd_coin_registry_new_currency">new_currency</a>` scenario), or <a href="../rtd/transfer.md#rtd_transfer">transfer</a> <b>as</b> TTO to `<a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">CoinRegistry</a>`.
    <b>if</b> (is_otw) <a href="../rtd/transfer.md#rtd_transfer_transfer">transfer::transfer</a>(currency, <a href="../rtd/object.md#rtd_object_rtd_coin_registry_address">object::rtd_coin_registry_address</a>())
    <b>else</b> <a href="../rtd/transfer.md#rtd_transfer_share_object">transfer::share_object</a>(currency);
}
</code></pre>



</details>

<a name="rtd_coin_registry_finalize_registration"></a>

## Function `finalize_registration`

The second step in the "otw" initialization of coin metadata, that takes in
the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;</code> that was transferred from init, and transforms it in to a
"derived address" shared object.

Can be performed by anyone.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_finalize_registration">finalize_registration</a>&lt;T&gt;(registry: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">rtd::coin_registry::CoinRegistry</a>, currency: <a href="../rtd/transfer.md#rtd_transfer_Receiving">rtd::transfer::Receiving</a>&lt;<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;&gt;, _ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_finalize_registration">finalize_registration</a>&lt;T&gt;(
    registry: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">CoinRegistry</a>,
    currency: Receiving&lt;<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;&gt;,
    _ctx: &<b>mut</b> TxContext,
) {
    // 1. Consume <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>
    // 2. Re-<a href="../rtd/coin_registry.md#rtd_coin_registry_create">create</a> it with a "derived" <b>address</b>.
    <b>let</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a> {
        id,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_decimals">decimals</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>,
        supply,
        regulated,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_treasury_cap_id">treasury_cap_id</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_metadata_cap_id">metadata_cap_id</a>,
        extra_fields,
    } = <a href="../rtd/transfer.md#rtd_transfer_receive">transfer::receive</a>(&<b>mut</b> registry.id, currency);
    id.delete();
    // Now, <a href="../rtd/coin_registry.md#rtd_coin_registry_create">create</a> the derived version of the <a href="../rtd/coin.md#rtd_coin">coin</a> currency.
    <a href="../rtd/transfer.md#rtd_transfer_share_object">transfer::share_object</a>(<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a> {
        id: <a href="../rtd/derived_object.md#rtd_derived_object_claim">derived_object::claim</a>(&<b>mut</b> registry.id, <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyKey">CurrencyKey</a>&lt;T&gt;()),
        <a href="../rtd/coin_registry.md#rtd_coin_registry_decimals">decimals</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>,
        supply,
        regulated,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_treasury_cap_id">treasury_cap_id</a>,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_metadata_cap_id">metadata_cap_id</a>,
        extra_fields,
    })
}
</code></pre>



</details>

<a name="rtd_coin_registry_delete_metadata_cap"></a>

## Function `delete_metadata_cap`

Delete the metadata cap making further updates of <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code> metadata impossible.
This action is IRREVERSIBLE, and the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a></code> can no longer be claimed.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_delete_metadata_cap">delete_metadata_cap</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, cap: <a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">rtd::coin_registry::MetadataCap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_delete_metadata_cap">delete_metadata_cap</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;, cap: <a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a>&lt;T&gt;) {
    <b>let</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a> { id } = cap;
    currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_metadata_cap_id">metadata_cap_id</a> = MetadataCapState::Deleted;
    id.delete();
}
</code></pre>



</details>

<a name="rtd_coin_registry_burn"></a>

## Function `burn`

Burn the <code>Coin</code> if the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code> has a <code>BurnOnly</code> supply state.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_burn">burn</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, <a href="../rtd/coin.md#rtd_coin">coin</a>: <a href="../rtd/coin.md#rtd_coin_Coin">rtd::coin::Coin</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_burn">burn</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;, <a href="../rtd/coin.md#rtd_coin">coin</a>: Coin&lt;T&gt;) {
    currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_burn_balance">burn_balance</a>(<a href="../rtd/coin.md#rtd_coin">coin</a>.into_balance());
}
</code></pre>



</details>

<a name="rtd_coin_registry_burn_balance"></a>

## Function `burn_balance`

Burn the <code>Balance</code> if the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code> has a <code>BurnOnly</code> supply state.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_burn_balance">burn_balance</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, <a href="../rtd/balance.md#rtd_balance">balance</a>: <a href="../rtd/balance.md#rtd_balance_Balance">rtd::balance::Balance</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_burn_balance">burn_balance</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;, <a href="../rtd/balance.md#rtd_balance">balance</a>: Balance&lt;T&gt;) {
    <b>assert</b>!(currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_is_supply_burn_only">is_supply_burn_only</a>(), <a href="../rtd/coin_registry.md#rtd_coin_registry_ESupplyNotBurnOnly">ESupplyNotBurnOnly</a>);
    match (currency.supply.borrow_mut()) {
        SupplyState::BurnOnly(supply) =&gt; { supply.decrease_supply(<a href="../rtd/balance.md#rtd_balance">balance</a>); },
        _ =&gt; <b>abort</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_EInvariantViolation">EInvariantViolation</a>, // unreachable
    }
}
</code></pre>



</details>

<a name="rtd_coin_registry_set_name"></a>

## Function `set_name`

Update the name of the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_set_name">set_name</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, _: &<a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">rtd::coin_registry::MetadataCap</a>&lt;T&gt;, <a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>: <a href="../std/string.md#std_string_String">std::string::String</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_set_name">set_name</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;, _: &<a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a>&lt;T&gt;, <a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>: String) {
    currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a> = <a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>;
}
</code></pre>



</details>

<a name="rtd_coin_registry_set_description"></a>

## Function `set_description`

Update the description of the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_set_description">set_description</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, _: &<a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">rtd::coin_registry::MetadataCap</a>&lt;T&gt;, <a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>: <a href="../std/string.md#std_string_String">std::string::String</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_set_description">set_description</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;, _: &<a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a>&lt;T&gt;, <a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>: String) {
    currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a> = <a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>;
}
</code></pre>



</details>

<a name="rtd_coin_registry_set_icon_url"></a>

## Function `set_icon_url`

Update the icon URL of the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_set_icon_url">set_icon_url</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, _: &<a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">rtd::coin_registry::MetadataCap</a>&lt;T&gt;, <a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>: <a href="../std/string.md#std_string_String">std::string::String</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_set_icon_url">set_icon_url</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;, _: &<a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a>&lt;T&gt;, <a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>: String) {
    currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a> = <a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>;
}
</code></pre>



</details>

<a name="rtd_coin_registry_set_treasury_cap_id"></a>

## Function `set_treasury_cap_id`

Register the treasury cap ID for a migrated <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code>. All currencies created with
<code><a href="../rtd/coin_registry.md#rtd_coin_registry_new_currency">new_currency</a></code> or <code><a href="../rtd/coin_registry.md#rtd_coin_registry_new_currency_with_otw">new_currency_with_otw</a></code> have their treasury cap ID set during
initialization.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_set_treasury_cap_id">set_treasury_cap_id</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, cap: &<a href="../rtd/coin.md#rtd_coin_TreasuryCap">rtd::coin::TreasuryCap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_set_treasury_cap_id">set_treasury_cap_id</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;, cap: &TreasuryCap&lt;T&gt;) {
    currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_treasury_cap_id">treasury_cap_id</a>.fill(<a href="../rtd/object.md#rtd_object_id">object::id</a>(cap));
}
</code></pre>



</details>

<a name="rtd_coin_registry_migrate_legacy_metadata"></a>

## Function `migrate_legacy_metadata`

Register <code>CoinMetadata</code> in the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">CoinRegistry</a></code>. This can happen only once, if the
<code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code> did not exist yet. Further updates are possible through
<code><a href="../rtd/coin_registry.md#rtd_coin_registry_update_from_legacy_metadata">update_from_legacy_metadata</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_migrate_legacy_metadata">migrate_legacy_metadata</a>&lt;T&gt;(registry: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">rtd::coin_registry::CoinRegistry</a>, legacy: &<a href="../rtd/coin.md#rtd_coin_CoinMetadata">rtd::coin::CoinMetadata</a>&lt;T&gt;, _ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_migrate_legacy_metadata">migrate_legacy_metadata</a>&lt;T&gt;(
    registry: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">CoinRegistry</a>,
    legacy: &CoinMetadata&lt;T&gt;,
    _ctx: &<b>mut</b> TxContext,
) {
    <b>let</b> currency = <a href="../rtd/coin_registry.md#rtd_coin_registry_migrate_legacy_metadata_impl">migrate_legacy_metadata_impl</a>!(registry, legacy);
    <a href="../rtd/transfer.md#rtd_transfer_share_object">transfer::share_object</a>(currency);
}
</code></pre>



</details>

<a name="rtd_coin_registry_update_from_legacy_metadata"></a>

## Function `update_from_legacy_metadata`

Update <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code> from <code>CoinMetadata</code> if the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a></code> is not claimed. After
the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a></code> is claimed, updates can only be made through <code>set_*</code> functions.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_update_from_legacy_metadata">update_from_legacy_metadata</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, legacy: &<a href="../rtd/coin.md#rtd_coin_CoinMetadata">rtd::coin::CoinMetadata</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_update_from_legacy_metadata">update_from_legacy_metadata</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;, legacy: &CoinMetadata&lt;T&gt;) {
    <b>assert</b>!(!currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_is_metadata_cap_claimed">is_metadata_cap_claimed</a>(), <a href="../rtd/coin_registry.md#rtd_coin_registry_ECannotUpdateManagedMetadata">ECannotUpdateManagedMetadata</a>);
    currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a> = legacy.get_name();
    currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a> = legacy.get_symbol().to_string();
    currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a> = legacy.get_description();
    currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_decimals">decimals</a> = legacy.get_decimals();
    currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a> =
        legacy.get_icon_url().map!(|<a href="../rtd/url.md#rtd_url">url</a>| <a href="../rtd/url.md#rtd_url">url</a>.inner_url().to_string()).destroy_or!(b"".to_string());
}
</code></pre>



</details>

<a name="rtd_coin_registry_delete_migrated_legacy_metadata"></a>

## Function `delete_migrated_legacy_metadata`



<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_delete_migrated_legacy_metadata">delete_migrated_legacy_metadata</a>&lt;T&gt;(_: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, _: <a href="../rtd/coin.md#rtd_coin_CoinMetadata">rtd::coin::CoinMetadata</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_delete_migrated_legacy_metadata">delete_migrated_legacy_metadata</a>&lt;T&gt;(_: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;, _: CoinMetadata&lt;T&gt;) {
    <b>abort</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_EDeletionNotSupported">EDeletionNotSupported</a>
}
</code></pre>



</details>

<a name="rtd_coin_registry_migrate_regulated_state_by_metadata"></a>

## Function `migrate_regulated_state_by_metadata`

Allow migrating the regulated state by access to <code>RegulatedCoinMetadata</code> frozen object.
This is a permissionless operation which can be performed only once.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_migrate_regulated_state_by_metadata">migrate_regulated_state_by_metadata</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, metadata: &<a href="../rtd/coin.md#rtd_coin_RegulatedCoinMetadata">rtd::coin::RegulatedCoinMetadata</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_migrate_regulated_state_by_metadata">migrate_regulated_state_by_metadata</a>&lt;T&gt;(
    currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;,
    metadata: &RegulatedCoinMetadata&lt;T&gt;,
) {
    // Only allow <b>if</b> this hasn't been migrated before.
    <b>assert</b>!(currency.regulated == RegulatedState::Unknown, <a href="../rtd/coin_registry.md#rtd_coin_registry_EDenyListStateAlreadySet">EDenyListStateAlreadySet</a>);
    currency.regulated =
        RegulatedState::Regulated {
            cap: metadata.<a href="../rtd/coin_registry.md#rtd_coin_registry_deny_cap_id">deny_cap_id</a>(),
            allow_global_pause: option::none(),
            variant: <a href="../rtd/coin_registry.md#rtd_coin_registry_REGULATED_COIN_VERSION">REGULATED_COIN_VERSION</a>,
        };
}
</code></pre>



</details>

<a name="rtd_coin_registry_migrate_regulated_state_by_cap"></a>

## Function `migrate_regulated_state_by_cap`

Mark regulated state by showing the <code>DenyCapV2</code> object for the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_migrate_regulated_state_by_cap">migrate_regulated_state_by_cap</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, cap: &<a href="../rtd/coin.md#rtd_coin_DenyCapV2">rtd::coin::DenyCapV2</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_migrate_regulated_state_by_cap">migrate_regulated_state_by_cap</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;, cap: &DenyCapV2&lt;T&gt;) {
    currency.regulated =
        RegulatedState::Regulated {
            cap: <a href="../rtd/object.md#rtd_object_id">object::id</a>(cap),
            allow_global_pause: option::some(cap.allow_global_pause()),
            variant: <a href="../rtd/coin_registry.md#rtd_coin_registry_REGULATED_COIN_VERSION">REGULATED_COIN_VERSION</a>,
        };
}
</code></pre>



</details>

<a name="rtd_coin_registry_borrow_legacy_metadata"></a>

## Function `borrow_legacy_metadata`

Borrow the legacy <code>CoinMetadata</code> from a new <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code>. To preserve the <code>ID</code>
of the legacy <code>CoinMetadata</code>, we create it on request and then store it as a
dynamic field for future borrows.

<code><a href="../rtd/coin_registry.md#rtd_coin_registry_Borrow">Borrow</a>&lt;T&gt;</code> ensures that the <code>CoinMetadata</code> is returned in the same transaction.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_borrow_legacy_metadata">borrow_legacy_metadata</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): (<a href="../rtd/coin.md#rtd_coin_CoinMetadata">rtd::coin::CoinMetadata</a>&lt;T&gt;, <a href="../rtd/coin_registry.md#rtd_coin_registry_Borrow">rtd::coin_registry::Borrow</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_borrow_legacy_metadata">borrow_legacy_metadata</a>&lt;T&gt;(
    currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;,
    ctx: &<b>mut</b> TxContext,
): (CoinMetadata&lt;T&gt;, <a href="../rtd/coin_registry.md#rtd_coin_registry_Borrow">Borrow</a>&lt;T&gt;) {
    <b>assert</b>!(!currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_is_migrated_from_legacy">is_migrated_from_legacy</a>(), <a href="../rtd/coin_registry.md#rtd_coin_registry_EBorrowLegacyMetadata">EBorrowLegacyMetadata</a>);
    <b>if</b> (!df::exists_(&currency.id, <a href="../rtd/coin_registry.md#rtd_coin_registry_LegacyMetadataKey">LegacyMetadataKey</a>())) {
        <b>let</b> legacy = currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_to_legacy_metadata">to_legacy_metadata</a>(ctx);
        df::add(&<b>mut</b> currency.id, <a href="../rtd/coin_registry.md#rtd_coin_registry_LegacyMetadataKey">LegacyMetadataKey</a>(), legacy);
    };
    <b>let</b> <b>mut</b> legacy: CoinMetadata&lt;T&gt; = df::remove(&<b>mut</b> currency.id, <a href="../rtd/coin_registry.md#rtd_coin_registry_LegacyMetadataKey">LegacyMetadataKey</a>());
    legacy.update_coin_metadata(
        currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>,
        currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>.to_ascii(),
        currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>,
        currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>.to_ascii(),
    );
    (legacy, <a href="../rtd/coin_registry.md#rtd_coin_registry_Borrow">Borrow</a> {})
}
</code></pre>



</details>

<a name="rtd_coin_registry_return_borrowed_legacy_metadata"></a>

## Function `return_borrowed_legacy_metadata`

Return the borrowed <code>CoinMetadata</code> and the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Borrow">Borrow</a></code> potato to the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code>.

Note to self: Borrow requirement prevents deletion through this method.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_return_borrowed_legacy_metadata">return_borrowed_legacy_metadata</a>&lt;T&gt;(currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, legacy: <a href="../rtd/coin.md#rtd_coin_CoinMetadata">rtd::coin::CoinMetadata</a>&lt;T&gt;, <a href="../rtd/borrow.md#rtd_borrow">borrow</a>: <a href="../rtd/coin_registry.md#rtd_coin_registry_Borrow">rtd::coin_registry::Borrow</a>&lt;T&gt;, _ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_return_borrowed_legacy_metadata">return_borrowed_legacy_metadata</a>&lt;T&gt;(
    currency: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;,
    <b>mut</b> legacy: CoinMetadata&lt;T&gt;,
    <a href="../rtd/borrow.md#rtd_borrow">borrow</a>: <a href="../rtd/coin_registry.md#rtd_coin_registry_Borrow">Borrow</a>&lt;T&gt;,
    _ctx: &<b>mut</b> TxContext,
) {
    <b>assert</b>!(!df::exists_(&currency.id, <a href="../rtd/coin_registry.md#rtd_coin_registry_LegacyMetadataKey">LegacyMetadataKey</a>()), <a href="../rtd/coin_registry.md#rtd_coin_registry_EDuplicateBorrow">EDuplicateBorrow</a>);
    <b>let</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_Borrow">Borrow</a> {} = <a href="../rtd/borrow.md#rtd_borrow">borrow</a>;
    // Always store up to date value.
    legacy.update_coin_metadata(
        currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>,
        currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>.to_ascii(),
        currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>,
        currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>.to_ascii(),
    );
    df::add(&<b>mut</b> currency.id, <a href="../rtd/coin_registry.md#rtd_coin_registry_LegacyMetadataKey">LegacyMetadataKey</a>(), legacy);
}
</code></pre>



</details>

<a name="rtd_coin_registry_decimals"></a>

## Function `decimals`

Get the number of decimal places for the coin type.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_decimals">decimals</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_decimals">decimals</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;): u8 { currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_decimals">decimals</a> }
</code></pre>



</details>

<a name="rtd_coin_registry_name"></a>

## Function `name`

Get the human-readable name of the coin.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;): <a href="../std/string.md#std_string_String">std::string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;): String { currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a> }
</code></pre>



</details>

<a name="rtd_coin_registry_symbol"></a>

## Function `symbol`

Get the symbol/ticker of the coin.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;): <a href="../std/string.md#std_string_String">std::string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;): String { currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a> }
</code></pre>



</details>

<a name="rtd_coin_registry_description"></a>

## Function `description`

Get the description of the coin.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;): <a href="../std/string.md#std_string_String">std::string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;): String { currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a> }
</code></pre>



</details>

<a name="rtd_coin_registry_icon_url"></a>

## Function `icon_url`

Get the icon URL for the coin.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;): <a href="../std/string.md#std_string_String">std::string::String</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;): String { currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a> }
</code></pre>



</details>

<a name="rtd_coin_registry_is_metadata_cap_claimed"></a>

## Function `is_metadata_cap_claimed`

Check if the metadata capability has been claimed for this <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code> type.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_is_metadata_cap_claimed">is_metadata_cap_claimed</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_is_metadata_cap_claimed">is_metadata_cap_claimed</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;): bool {
    match (currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_metadata_cap_id">metadata_cap_id</a>) {
        MetadataCapState::Claimed(_) | MetadataCapState::Deleted =&gt; <b>true</b>,
        _ =&gt; <b>false</b>,
    }
}
</code></pre>



</details>

<a name="rtd_coin_registry_is_metadata_cap_deleted"></a>

## Function `is_metadata_cap_deleted`

Check if the metadata capability has been deleted for this <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code> type.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_is_metadata_cap_deleted">is_metadata_cap_deleted</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_is_metadata_cap_deleted">is_metadata_cap_deleted</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;): bool {
    match (currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_metadata_cap_id">metadata_cap_id</a>) {
        MetadataCapState::Deleted =&gt; <b>true</b>,
        _ =&gt; <b>false</b>,
    }
}
</code></pre>



</details>

<a name="rtd_coin_registry_metadata_cap_id"></a>

## Function `metadata_cap_id`

Get the metadata cap ID, or none if it has not been claimed.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_metadata_cap_id">metadata_cap_id</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;): <a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;<a href="../rtd/object.md#rtd_object_ID">rtd::object::ID</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_metadata_cap_id">metadata_cap_id</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;): Option&lt;ID&gt; {
    match (currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_metadata_cap_id">metadata_cap_id</a>) {
        MetadataCapState::Claimed(id) =&gt; option::some(id),
        _ =&gt; option::none(),
    }
}
</code></pre>



</details>

<a name="rtd_coin_registry_treasury_cap_id"></a>

## Function `treasury_cap_id`

Get the treasury cap ID for this coin type, if registered.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_treasury_cap_id">treasury_cap_id</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;): <a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;<a href="../rtd/object.md#rtd_object_ID">rtd::object::ID</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_treasury_cap_id">treasury_cap_id</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;): Option&lt;ID&gt; {
    currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_treasury_cap_id">treasury_cap_id</a>
}
</code></pre>



</details>

<a name="rtd_coin_registry_deny_cap_id"></a>

## Function `deny_cap_id`

Get the deny cap ID for this coin type, if it's a regulated coin.
Returns <code>None</code> if:
- The <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code> is not regulated;
- The <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code> is migrated from legacy, and its regulated state has not been set;


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_deny_cap_id">deny_cap_id</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;): <a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;<a href="../rtd/object.md#rtd_object_ID">rtd::object::ID</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_deny_cap_id">deny_cap_id</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;): Option&lt;ID&gt; {
    match (currency.regulated) {
        RegulatedState::Regulated { cap, .. } =&gt; option::some(cap),
        RegulatedState::Unregulated | RegulatedState::Unknown =&gt; option::none(),
    }
}
</code></pre>



</details>

<a name="rtd_coin_registry_is_supply_fixed"></a>

## Function `is_supply_fixed`

Check if the supply is fixed.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_is_supply_fixed">is_supply_fixed</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_is_supply_fixed">is_supply_fixed</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;): bool {
    match (currency.supply.<a href="../rtd/borrow.md#rtd_borrow">borrow</a>()) {
        SupplyState::Fixed(_) =&gt; <b>true</b>,
        _ =&gt; <b>false</b>,
    }
}
</code></pre>



</details>

<a name="rtd_coin_registry_is_supply_burn_only"></a>

## Function `is_supply_burn_only`

Check if the supply is burn-only.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_is_supply_burn_only">is_supply_burn_only</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_is_supply_burn_only">is_supply_burn_only</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;): bool {
    match (currency.supply.<a href="../rtd/borrow.md#rtd_borrow">borrow</a>()) {
        SupplyState::BurnOnly(_) =&gt; <b>true</b>,
        _ =&gt; <b>false</b>,
    }
}
</code></pre>



</details>

<a name="rtd_coin_registry_is_regulated"></a>

## Function `is_regulated`

Check if the currency is regulated.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_is_regulated">is_regulated</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_is_regulated">is_regulated</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;): bool {
    match (currency.regulated) {
        RegulatedState::Regulated { .. } =&gt; <b>true</b>,
        _ =&gt; <b>false</b>,
    }
}
</code></pre>



</details>

<a name="rtd_coin_registry_total_supply"></a>

## Function `total_supply`

Get the total supply for the <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;</code> if the Supply is in fixed or
burn-only state. Returns <code>None</code> if the SupplyState is Unknown.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_total_supply">total_supply</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;): <a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_total_supply">total_supply</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;): Option&lt;u64&gt; {
    match (currency.supply.<a href="../rtd/borrow.md#rtd_borrow">borrow</a>()) {
        SupplyState::Fixed(supply) =&gt; option::some(supply.value()),
        SupplyState::BurnOnly(supply) =&gt; option::some(supply.value()),
        SupplyState::Unknown =&gt; option::none(),
    }
}
</code></pre>



</details>

<a name="rtd_coin_registry_exists"></a>

## Function `exists`

Check if coin data exists for the given type T in the registry.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_exists">exists</a>&lt;T&gt;(registry: &<a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">rtd::coin_registry::CoinRegistry</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_exists">exists</a>&lt;T&gt;(registry: &<a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">CoinRegistry</a>): bool {
    <a href="../rtd/derived_object.md#rtd_derived_object_exists">derived_object::exists</a>(&registry.id, <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyKey">CurrencyKey</a>&lt;T&gt;())
}
</code></pre>



</details>

<a name="rtd_coin_registry_is_migrated_from_legacy"></a>

## Function `is_migrated_from_legacy`

Whether the currency is migrated from legacy.


<pre><code><b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_is_migrated_from_legacy">is_migrated_from_legacy</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_is_migrated_from_legacy">is_migrated_from_legacy</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;): bool {
    !currency.extra_fields.contains(&<a href="../rtd/coin_registry.md#rtd_coin_registry_NEW_CURRENCY_MARKER">NEW_CURRENCY_MARKER</a>.to_string())
}
</code></pre>



</details>

<a name="rtd_coin_registry_to_legacy_metadata"></a>

## Function `to_legacy_metadata`

Create a new legacy <code>CoinMetadata</code> from a <code><a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a></code>.


<pre><code><b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_to_legacy_metadata">to_legacy_metadata</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;T&gt;, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): <a href="../rtd/coin.md#rtd_coin_CoinMetadata">rtd::coin::CoinMetadata</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_to_legacy_metadata">to_legacy_metadata</a>&lt;T&gt;(currency: &<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;T&gt;, ctx: &<b>mut</b> TxContext): CoinMetadata&lt;T&gt; {
    <a href="../rtd/coin.md#rtd_coin_new_coin_metadata">coin::new_coin_metadata</a>(
        currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_decimals">decimals</a>,
        currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>,
        currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>.to_ascii(),
        currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>,
        currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>.to_ascii(),
        ctx,
    )
}
</code></pre>



</details>

<a name="rtd_coin_registry_create"></a>

## Function `create`

Create and share the singleton <code><a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">CoinRegistry</a></code> -- this function is
called exactly once, during the upgrade epoch.
Only the system address (0x0) can create the registry.


<pre><code><b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_create">create</a>(ctx: &<a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_create">create</a>(ctx: &TxContext) {
    <b>assert</b>!(ctx.sender() == @0x0, <a href="../rtd/coin_registry.md#rtd_coin_registry_ENotSystemAddress">ENotSystemAddress</a>);
    <a href="../rtd/transfer.md#rtd_transfer_share_object">transfer::share_object</a>(<a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">CoinRegistry</a> {
        id: <a href="../rtd/object.md#rtd_object_rtd_coin_registry_object_id">object::rtd_coin_registry_object_id</a>(),
    });
}
</code></pre>



</details>

<a name="rtd_coin_registry_finalize_impl"></a>

## Macro function `finalize_impl`

Internal macro to keep implementation between build and test modes.


<pre><code><b>macro</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_finalize_impl">finalize_impl</a>&lt;$T&gt;($builder: <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">rtd::coin_registry::CurrencyInitializer</a>&lt;$T&gt;, $ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): (<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;$T&gt;, <a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">rtd::coin_registry::MetadataCap</a>&lt;$T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>macro</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_finalize_impl">finalize_impl</a>&lt;$T&gt;(
    $builder: <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">CurrencyInitializer</a>&lt;$T&gt;,
    $ctx: &<b>mut</b> TxContext,
): (<a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;$T&gt;, <a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a>&lt;$T&gt;) {
    <b>let</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyInitializer">CurrencyInitializer</a> { <b>mut</b> currency, extra_fields, is_otw: _ } = $builder;
    extra_fields.destroy_empty();
    <b>let</b> id = <a href="../rtd/object.md#rtd_object_new">object::new</a>($ctx);
    currency.<a href="../rtd/coin_registry.md#rtd_coin_registry_metadata_cap_id">metadata_cap_id</a> = MetadataCapState::Claimed(id.to_inner());
    // Mark the currency <b>as</b> new, so in the future we can support borrowing of the
    // legacy metadata.
    currency
        .extra_fields
        .insert(
            <a href="../rtd/coin_registry.md#rtd_coin_registry_NEW_CURRENCY_MARKER">NEW_CURRENCY_MARKER</a>.to_string(),
            <a href="../rtd/coin_registry.md#rtd_coin_registry_ExtraField">ExtraField</a>(type_name::with_original_ids&lt;bool&gt;(), <a href="../rtd/coin_registry.md#rtd_coin_registry_NEW_CURRENCY_MARKER">NEW_CURRENCY_MARKER</a>),
        );
    (currency, <a href="../rtd/coin_registry.md#rtd_coin_registry_MetadataCap">MetadataCap</a>&lt;$T&gt; { id })
}
</code></pre>



</details>

<a name="rtd_coin_registry_migrate_legacy_metadata_impl"></a>

## Macro function `migrate_legacy_metadata_impl`

Internal macro to keep implementation between build and test modes.


<pre><code><b>macro</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_migrate_legacy_metadata_impl">migrate_legacy_metadata_impl</a>&lt;$T&gt;($registry: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">rtd::coin_registry::CoinRegistry</a>, $legacy: &<a href="../rtd/coin.md#rtd_coin_CoinMetadata">rtd::coin::CoinMetadata</a>&lt;$T&gt;): <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">rtd::coin_registry::Currency</a>&lt;$T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>macro</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_migrate_legacy_metadata_impl">migrate_legacy_metadata_impl</a>&lt;$T&gt;(
    $registry: &<b>mut</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_CoinRegistry">CoinRegistry</a>,
    $legacy: &CoinMetadata&lt;$T&gt;,
): <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;$T&gt; {
    <b>let</b> registry = $registry;
    <b>let</b> legacy = $legacy;
    <b>assert</b>!(!registry.<a href="../rtd/coin_registry.md#rtd_coin_registry_exists">exists</a>&lt;$T&gt;(), <a href="../rtd/coin_registry.md#rtd_coin_registry_ECurrencyAlreadyRegistered">ECurrencyAlreadyRegistered</a>);
    <b>assert</b>!(<a href="../rtd/coin_registry.md#rtd_coin_registry_is_ascii_printable">is_ascii_printable</a>!(&legacy.get_symbol().to_string()), <a href="../rtd/coin_registry.md#rtd_coin_registry_EInvalidSymbol">EInvalidSymbol</a>);
    <a href="../rtd/coin_registry.md#rtd_coin_registry_Currency">Currency</a>&lt;$T&gt; {
        id: <a href="../rtd/derived_object.md#rtd_derived_object_claim">derived_object::claim</a>(&<b>mut</b> registry.id, <a href="../rtd/coin_registry.md#rtd_coin_registry_CurrencyKey">CurrencyKey</a>&lt;$T&gt;()),
        <a href="../rtd/coin_registry.md#rtd_coin_registry_decimals">decimals</a>: legacy.get_decimals(),
        <a href="../rtd/coin_registry.md#rtd_coin_registry_name">name</a>: legacy.get_name(),
        <a href="../rtd/coin_registry.md#rtd_coin_registry_symbol">symbol</a>: legacy.get_symbol().to_string(),
        <a href="../rtd/coin_registry.md#rtd_coin_registry_description">description</a>: legacy.get_description(),
        <a href="../rtd/coin_registry.md#rtd_coin_registry_icon_url">icon_url</a>: legacy
            .get_icon_url()
            .map!(|<a href="../rtd/url.md#rtd_url">url</a>| <a href="../rtd/url.md#rtd_url">url</a>.inner_url().to_string())
            .destroy_or!(b"".to_string()),
        supply: option::some(SupplyState::Unknown),
        regulated: RegulatedState::Unknown,
        <a href="../rtd/coin_registry.md#rtd_coin_registry_treasury_cap_id">treasury_cap_id</a>: option::none(),
        <a href="../rtd/coin_registry.md#rtd_coin_registry_metadata_cap_id">metadata_cap_id</a>: MetadataCapState::Unclaimed,
        extra_fields: <a href="../rtd/vec_map.md#rtd_vec_map_empty">vec_map::empty</a>(),
    }
}
</code></pre>



</details>

<a name="rtd_coin_registry_is_ascii_printable"></a>

## Macro function `is_ascii_printable`

Nit: consider adding this function to <code><a href="../std/string.md#std_string">std::string</a></code> in the future.


<pre><code><b>macro</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_is_ascii_printable">is_ascii_printable</a>($s: &<a href="../std/string.md#std_string_String">std::string::String</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>macro</b> <b>fun</b> <a href="../rtd/coin_registry.md#rtd_coin_registry_is_ascii_printable">is_ascii_printable</a>($s: &String): bool {
    <b>let</b> s = $s;
    s.as_bytes().all!(|b| ascii::is_printable_char(*b))
}
</code></pre>



</details>
