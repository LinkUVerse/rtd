---
title: Module `rtd::display_registry`
---



-  [Struct `DisplayRegistry`](#rtd_display_registry_DisplayRegistry)
-  [Struct `SystemMigrationCap`](#rtd_display_registry_SystemMigrationCap)
-  [Struct `Display`](#rtd_display_registry_Display)
-  [Struct `DisplayCap`](#rtd_display_registry_DisplayCap)
-  [Struct `DisplayKey`](#rtd_display_registry_DisplayKey)
-  [Constants](#@Constants_0)
-  [Function `new`](#rtd_display_registry_new)
-  [Function `new_with_publisher`](#rtd_display_registry_new_with_publisher)
-  [Function `unset`](#rtd_display_registry_unset)
-  [Function `set`](#rtd_display_registry_set)
-  [Function `clear`](#rtd_display_registry_clear)
-  [Function `share`](#rtd_display_registry_share)
-  [Function `claim`](#rtd_display_registry_claim)
-  [Function `claim_with_publisher`](#rtd_display_registry_claim_with_publisher)
-  [Function `system_migration`](#rtd_display_registry_system_migration)
-  [Function `migrate_v1_to_v2`](#rtd_display_registry_migrate_v1_to_v2)
-  [Function `destroy_system_migration_cap`](#rtd_display_registry_destroy_system_migration_cap)
-  [Function `transfer_migration_cap`](#rtd_display_registry_transfer_migration_cap)
-  [Function `delete_legacy`](#rtd_display_registry_delete_legacy)
-  [Function `fields`](#rtd_display_registry_fields)
-  [Function `cap_id`](#rtd_display_registry_cap_id)
-  [Function `migration_cap_receiver`](#rtd_display_registry_migration_cap_receiver)
-  [Function `new_display`](#rtd_display_registry_new_display)
-  [Function `create`](#rtd_display_registry_create)


<pre><code><b>use</b> <a href="../rtd/accumulator.md#rtd_accumulator">rtd::accumulator</a>;
<b>use</b> <a href="../rtd/accumulator_settlement.md#rtd_accumulator_settlement">rtd::accumulator_settlement</a>;
<b>use</b> <a href="../rtd/address.md#rtd_address">rtd::address</a>;
<b>use</b> <a href="../rtd/bcs.md#rtd_bcs">rtd::bcs</a>;
<b>use</b> <a href="../rtd/derived_object.md#rtd_derived_object">rtd::derived_object</a>;
<b>use</b> <a href="../rtd/display.md#rtd_display">rtd::display</a>;
<b>use</b> <a href="../rtd/dynamic_field.md#rtd_dynamic_field">rtd::dynamic_field</a>;
<b>use</b> <a href="../rtd/event.md#rtd_event">rtd::event</a>;
<b>use</b> <a href="../rtd/hash.md#rtd_hash">rtd::hash</a>;
<b>use</b> <a href="../rtd/hex.md#rtd_hex">rtd::hex</a>;
<b>use</b> <a href="../rtd/object.md#rtd_object">rtd::object</a>;
<b>use</b> <a href="../rtd/package.md#rtd_package">rtd::package</a>;
<b>use</b> <a href="../rtd/party.md#rtd_party">rtd::party</a>;
<b>use</b> <a href="../rtd/transfer.md#rtd_transfer">rtd::transfer</a>;
<b>use</b> <a href="../rtd/tx_context.md#rtd_tx_context">rtd::tx_context</a>;
<b>use</b> <a href="../rtd/types.md#rtd_types">rtd::types</a>;
<b>use</b> <a href="../rtd/vec_map.md#rtd_vec_map">rtd::vec_map</a>;
<b>use</b> <a href="../std/address.md#std_address">std::address</a>;
<b>use</b> <a href="../std/ascii.md#std_ascii">std::ascii</a>;
<b>use</b> <a href="../std/bcs.md#std_bcs">std::bcs</a>;
<b>use</b> <a href="../std/internal.md#std_internal">std::internal</a>;
<b>use</b> <a href="../std/option.md#std_option">std::option</a>;
<b>use</b> <a href="../std/string.md#std_string">std::string</a>;
<b>use</b> <a href="../std/type_name.md#std_type_name">std::type_name</a>;
<b>use</b> <a href="../std/vector.md#std_vector">std::vector</a>;
</code></pre>



<a name="rtd_display_registry_DisplayRegistry"></a>

## Struct `DisplayRegistry`

The root of display, to enable derivation of addresses.
The address is system-generated at <code>0xd</code>


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/display_registry.md#rtd_display_registry_DisplayRegistry">DisplayRegistry</a> <b>has</b> key
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

<a name="rtd_display_registry_SystemMigrationCap"></a>

## Struct `SystemMigrationCap`

A singleton capability object to enable migrating all V1 displays into V2.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/display_registry.md#rtd_display_registry_SystemMigrationCap">SystemMigrationCap</a> <b>has</b> key
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

<a name="rtd_display_registry_Display"></a>

## Struct `Display`

This is the struct that holds the display values for a type T.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a>&lt;<b>phantom</b> T&gt; <b>has</b> key
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
<code><a href="../rtd/display_registry.md#rtd_display_registry_fields">fields</a>: <a href="../rtd/vec_map.md#rtd_vec_map_VecMap">rtd::vec_map::VecMap</a>&lt;<a href="../std/string.md#std_string_String">std::string::String</a>, <a href="../std/string.md#std_string_String">std::string::String</a>&gt;</code>
</dt>
<dd>
 All the (key,value) entries for a given display object.
</dd>
<dt>
<code><a href="../rtd/display_registry.md#rtd_display_registry_cap_id">cap_id</a>: <a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;<a href="../rtd/object.md#rtd_object_ID">rtd::object::ID</a>&gt;</code>
</dt>
<dd>
 The capability object ID. It's <code>Option</code> because legacy Displays will need claiming.
</dd>
</dl>


</details>

<a name="rtd_display_registry_DisplayCap"></a>

## Struct `DisplayCap`

The capability object that is used to manage the display.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">DisplayCap</a>&lt;<b>phantom</b> T&gt; <b>has</b> key, store
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

<a name="rtd_display_registry_DisplayKey"></a>

## Struct `DisplayKey`

The key used for deriving the instance of <code><a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a></code>.


<pre><code><b>public</b> <b>struct</b> <a href="../rtd/display_registry.md#rtd_display_registry_DisplayKey">DisplayKey</a>&lt;<b>phantom</b> T&gt; <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="rtd_display_registry_SYSTEM_MIGRATION_ADDRESS"></a>

This is a multi-sig address responsible for the migration of V1 displays into V2.


<pre><code><b>const</b> <a href="../rtd/display_registry.md#rtd_display_registry_SYSTEM_MIGRATION_ADDRESS">SYSTEM_MIGRATION_ADDRESS</a>: <b>address</b> = 0x80e8249451c1a94b0d4ec317d9dd040f11344dcce6f917218086caf2bb1d7bdd;
</code></pre>



<a name="rtd_display_registry_ENotSystemAddress"></a>



<pre><code>#[error]
<b>const</b> <a href="../rtd/display_registry.md#rtd_display_registry_ENotSystemAddress">ENotSystemAddress</a>: vector&lt;u8&gt; = b"This is only callable from system <b>address</b>.";
</code></pre>



<a name="rtd_display_registry_EDisplayAlreadyExists"></a>



<pre><code>#[error]
<b>const</b> <a href="../rtd/display_registry.md#rtd_display_registry_EDisplayAlreadyExists">EDisplayAlreadyExists</a>: vector&lt;u8&gt; = b"<a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a> <b>for</b> the supplied type already exists.";
</code></pre>



<a name="rtd_display_registry_ECapAlreadyClaimed"></a>



<pre><code>#[error]
<b>const</b> <a href="../rtd/display_registry.md#rtd_display_registry_ECapAlreadyClaimed">ECapAlreadyClaimed</a>: vector&lt;u8&gt; = b"Cap <b>for</b> this <a href="../rtd/display.md#rtd_display">display</a> <a href="../rtd/object.md#rtd_object">object</a> <b>has</b> already been claimed.";
</code></pre>



<a name="rtd_display_registry_ENotValidPublisher"></a>



<pre><code>#[error]
<b>const</b> <a href="../rtd/display_registry.md#rtd_display_registry_ENotValidPublisher">ENotValidPublisher</a>: vector&lt;u8&gt; = b"The publisher is not valid <b>for</b> the supplied type.";
</code></pre>



<a name="rtd_display_registry_EFieldDoesNotExist"></a>



<pre><code>#[error]
<b>const</b> <a href="../rtd/display_registry.md#rtd_display_registry_EFieldDoesNotExist">EFieldDoesNotExist</a>: vector&lt;u8&gt; = b"Field does not exist in the <a href="../rtd/display.md#rtd_display">display</a>.";
</code></pre>



<a name="rtd_display_registry_ECapNotClaimed"></a>



<pre><code>#[error]
<b>const</b> <a href="../rtd/display_registry.md#rtd_display_registry_ECapNotClaimed">ECapNotClaimed</a>: vector&lt;u8&gt; = b"Cap <b>for</b> this <a href="../rtd/display.md#rtd_display">display</a> <a href="../rtd/object.md#rtd_object">object</a> <b>has</b> not been claimed so you cannot delete the legacy <a href="../rtd/display.md#rtd_display">display</a> yet.";
</code></pre>



<a name="rtd_display_registry_new"></a>

## Function `new`

Create a new Display object for a given type <code>T</code> using <code>internal::Permit</code> to
prove type ownership.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_new">new</a>&lt;T&gt;(registry: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_DisplayRegistry">rtd::display_registry::DisplayRegistry</a>, _: <a href="../std/internal.md#std_internal_Permit">std::internal::Permit</a>&lt;T&gt;, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): (<a href="../rtd/display_registry.md#rtd_display_registry_Display">rtd::display_registry::Display</a>&lt;T&gt;, <a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">rtd::display_registry::DisplayCap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_new">new</a>&lt;T&gt;(
    registry: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_DisplayRegistry">DisplayRegistry</a>,
    _: internal::Permit&lt;T&gt;,
    ctx: &<b>mut</b> TxContext,
): (<a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a>&lt;T&gt;, <a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">DisplayCap</a>&lt;T&gt;) {
    <b>let</b> (<a href="../rtd/display.md#rtd_display">display</a>, cap) = <a href="../rtd/display_registry.md#rtd_display_registry_new_display">new_display</a>&lt;T&gt;(registry, ctx);
    (<a href="../rtd/display.md#rtd_display">display</a>, cap)
}
</code></pre>



</details>

<a name="rtd_display_registry_new_with_publisher"></a>

## Function `new_with_publisher`

Create a new display object using the <code>Publisher</code> object.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_new_with_publisher">new_with_publisher</a>&lt;T&gt;(registry: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_DisplayRegistry">rtd::display_registry::DisplayRegistry</a>, publisher: &<b>mut</b> <a href="../rtd/package.md#rtd_package_Publisher">rtd::package::Publisher</a>, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): (<a href="../rtd/display_registry.md#rtd_display_registry_Display">rtd::display_registry::Display</a>&lt;T&gt;, <a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">rtd::display_registry::DisplayCap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_new_with_publisher">new_with_publisher</a>&lt;T&gt;(
    registry: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_DisplayRegistry">DisplayRegistry</a>,
    publisher: &<b>mut</b> Publisher,
    ctx: &<b>mut</b> TxContext,
): (<a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a>&lt;T&gt;, <a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">DisplayCap</a>&lt;T&gt;) {
    <b>assert</b>!(publisher.from_package&lt;T&gt;(), <a href="../rtd/display_registry.md#rtd_display_registry_ENotValidPublisher">ENotValidPublisher</a>);
    <b>let</b> (<a href="../rtd/display.md#rtd_display">display</a>, cap) = <a href="../rtd/display_registry.md#rtd_display_registry_new_display">new_display</a>&lt;T&gt;(registry, ctx);
    (<a href="../rtd/display.md#rtd_display">display</a>, cap)
}
</code></pre>



</details>

<a name="rtd_display_registry_unset"></a>

## Function `unset`

Unset a key from display.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_unset">unset</a>&lt;T&gt;(<a href="../rtd/display.md#rtd_display">display</a>: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_Display">rtd::display_registry::Display</a>&lt;T&gt;, _: &<a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">rtd::display_registry::DisplayCap</a>&lt;T&gt;, name: <a href="../std/string.md#std_string_String">std::string::String</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_unset">unset</a>&lt;T&gt;(<a href="../rtd/display.md#rtd_display">display</a>: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a>&lt;T&gt;, _: &<a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">DisplayCap</a>&lt;T&gt;, name: String) {
    <b>assert</b>!(<a href="../rtd/display.md#rtd_display">display</a>.<a href="../rtd/display_registry.md#rtd_display_registry_fields">fields</a>.contains(&name), <a href="../rtd/display_registry.md#rtd_display_registry_EFieldDoesNotExist">EFieldDoesNotExist</a>);
    <a href="../rtd/display.md#rtd_display">display</a>.<a href="../rtd/display_registry.md#rtd_display_registry_fields">fields</a>.remove(&name);
}
</code></pre>



</details>

<a name="rtd_display_registry_set"></a>

## Function `set`

Set a value for the specified key, replaces existing value if it exists.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_set">set</a>&lt;T&gt;(<a href="../rtd/display.md#rtd_display">display</a>: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_Display">rtd::display_registry::Display</a>&lt;T&gt;, _: &<a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">rtd::display_registry::DisplayCap</a>&lt;T&gt;, name: <a href="../std/string.md#std_string_String">std::string::String</a>, value: <a href="../std/string.md#std_string_String">std::string::String</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_set">set</a>&lt;T&gt;(<a href="../rtd/display.md#rtd_display">display</a>: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a>&lt;T&gt;, _: &<a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">DisplayCap</a>&lt;T&gt;, name: String, value: String) {
    <b>if</b> (<a href="../rtd/display.md#rtd_display">display</a>.<a href="../rtd/display_registry.md#rtd_display_registry_fields">fields</a>.contains(&name)) {
        <a href="../rtd/display.md#rtd_display">display</a>.<a href="../rtd/display_registry.md#rtd_display_registry_fields">fields</a>.remove(&name);
    };
    <a href="../rtd/display.md#rtd_display">display</a>.<a href="../rtd/display_registry.md#rtd_display_registry_fields">fields</a>.insert(name, value);
}
</code></pre>



</details>

<a name="rtd_display_registry_clear"></a>

## Function `clear`

Clear the display vec_map, allowing a fresh re-creation of fields


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_clear">clear</a>&lt;T&gt;(<a href="../rtd/display.md#rtd_display">display</a>: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_Display">rtd::display_registry::Display</a>&lt;T&gt;, _: &<a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">rtd::display_registry::DisplayCap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_clear">clear</a>&lt;T&gt;(<a href="../rtd/display.md#rtd_display">display</a>: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a>&lt;T&gt;, _: &<a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">DisplayCap</a>&lt;T&gt;) {
    <a href="../rtd/display.md#rtd_display">display</a>.<a href="../rtd/display_registry.md#rtd_display_registry_fields">fields</a> = <a href="../rtd/vec_map.md#rtd_vec_map_empty">vec_map::empty</a>();
}
</code></pre>



</details>

<a name="rtd_display_registry_share"></a>

## Function `share`

Share the <code><a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a></code> object to finalize the creation.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_share">share</a>&lt;T&gt;(<a href="../rtd/display.md#rtd_display">display</a>: <a href="../rtd/display_registry.md#rtd_display_registry_Display">rtd::display_registry::Display</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_share">share</a>&lt;T&gt;(<a href="../rtd/display.md#rtd_display">display</a>: <a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a>&lt;T&gt;) {
    <a href="../rtd/transfer.md#rtd_transfer_share_object">transfer::share_object</a>(<a href="../rtd/display.md#rtd_display">display</a>)
}
</code></pre>



</details>

<a name="rtd_display_registry_claim"></a>

## Function `claim`

Allow a legacy Display holder to claim the capability object.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_claim">claim</a>&lt;T: key&gt;(<a href="../rtd/display.md#rtd_display">display</a>: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_Display">rtd::display_registry::Display</a>&lt;T&gt;, legacy: <a href="../rtd/display.md#rtd_display_Display">rtd::display::Display</a>&lt;T&gt;, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): <a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">rtd::display_registry::DisplayCap</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_claim">claim</a>&lt;T: key&gt;(
    <a href="../rtd/display.md#rtd_display">display</a>: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a>&lt;T&gt;,
    legacy: LegacyDisplay&lt;T&gt;,
    ctx: &<b>mut</b> TxContext,
): <a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">DisplayCap</a>&lt;T&gt; {
    <b>assert</b>!(<a href="../rtd/display.md#rtd_display">display</a>.<a href="../rtd/display_registry.md#rtd_display_registry_cap_id">cap_id</a>.is_none(), <a href="../rtd/display_registry.md#rtd_display_registry_ECapAlreadyClaimed">ECapAlreadyClaimed</a>);
    <b>let</b> cap = <a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">DisplayCap</a>&lt;T&gt; { id: <a href="../rtd/object.md#rtd_object_new">object::new</a>(ctx) };
    <a href="../rtd/display.md#rtd_display">display</a>.<a href="../rtd/display_registry.md#rtd_display_registry_cap_id">cap_id</a>.fill(cap.id.to_inner());
    legacy.destroy();
    cap
}
</code></pre>



</details>

<a name="rtd_display_registry_claim_with_publisher"></a>

## Function `claim_with_publisher`

Allow claiming a new display using <code>Publisher</code> as proof of ownership.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_claim_with_publisher">claim_with_publisher</a>&lt;T: key&gt;(<a href="../rtd/display.md#rtd_display">display</a>: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_Display">rtd::display_registry::Display</a>&lt;T&gt;, publisher: &<b>mut</b> <a href="../rtd/package.md#rtd_package_Publisher">rtd::package::Publisher</a>, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): <a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">rtd::display_registry::DisplayCap</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_claim_with_publisher">claim_with_publisher</a>&lt;T: key&gt;(
    <a href="../rtd/display.md#rtd_display">display</a>: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a>&lt;T&gt;,
    publisher: &<b>mut</b> Publisher,
    ctx: &<b>mut</b> TxContext,
): <a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">DisplayCap</a>&lt;T&gt; {
    <b>assert</b>!(<a href="../rtd/display.md#rtd_display">display</a>.<a href="../rtd/display_registry.md#rtd_display_registry_cap_id">cap_id</a>.is_none(), <a href="../rtd/display_registry.md#rtd_display_registry_ECapAlreadyClaimed">ECapAlreadyClaimed</a>);
    <b>assert</b>!(publisher.from_package&lt;T&gt;(), <a href="../rtd/display_registry.md#rtd_display_registry_ENotValidPublisher">ENotValidPublisher</a>);
    <b>let</b> cap = <a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">DisplayCap</a>&lt;T&gt; { id: <a href="../rtd/object.md#rtd_object_new">object::new</a>(ctx) };
    <a href="../rtd/display.md#rtd_display">display</a>.<a href="../rtd/display_registry.md#rtd_display_registry_cap_id">cap_id</a>.fill(cap.id.to_inner());
    cap
}
</code></pre>



</details>

<a name="rtd_display_registry_system_migration"></a>

## Function `system_migration`

Allow the <code><a href="../rtd/display_registry.md#rtd_display_registry_SystemMigrationCap">SystemMigrationCap</a></code> holder to create display objects with supplied
values. The migration is performed once on launch of the DisplayRegistry,
further migrations will have to be performed for each object, and will only
be possible until legacy <code><a href="../rtd/display.md#rtd_display">display</a></code> methods are finally deprecated.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_system_migration">system_migration</a>&lt;T: key&gt;(registry: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_DisplayRegistry">rtd::display_registry::DisplayRegistry</a>, _: &<a href="../rtd/display_registry.md#rtd_display_registry_SystemMigrationCap">rtd::display_registry::SystemMigrationCap</a>, keys: vector&lt;<a href="../std/string.md#std_string_String">std::string::String</a>&gt;, values: vector&lt;<a href="../std/string.md#std_string_String">std::string::String</a>&gt;, _ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_system_migration">system_migration</a>&lt;T: key&gt;(
    registry: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_DisplayRegistry">DisplayRegistry</a>,
    _: &<a href="../rtd/display_registry.md#rtd_display_registry_SystemMigrationCap">SystemMigrationCap</a>,
    keys: vector&lt;String&gt;,
    values: vector&lt;String&gt;,
    _ctx: &<b>mut</b> TxContext,
) {
    <b>let</b> key = <a href="../rtd/display_registry.md#rtd_display_registry_DisplayKey">DisplayKey</a>&lt;T&gt;();
    // Gracefully <b>return</b> to avoid batching issues <b>if</b> someone migrates before our script.
    <b>if</b> (<a href="../rtd/derived_object.md#rtd_derived_object_exists">derived_object::exists</a>(&registry.id, key)) <b>return</b>;
    <a href="../rtd/transfer.md#rtd_transfer_share_object">transfer::share_object</a>(<a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a>&lt;T&gt; {
        id: <a href="../rtd/derived_object.md#rtd_derived_object_claim">derived_object::claim</a>(&<b>mut</b> registry.id, key),
        <a href="../rtd/display_registry.md#rtd_display_registry_fields">fields</a>: <a href="../rtd/vec_map.md#rtd_vec_map_from_keys_values">vec_map::from_keys_values</a>(keys, values),
        <a href="../rtd/display_registry.md#rtd_display_registry_cap_id">cap_id</a>: option::none(),
    });
}
</code></pre>



</details>

<a name="rtd_display_registry_migrate_v1_to_v2"></a>

## Function `migrate_v1_to_v2`

Enables migrating legacy display into the new one,
if a new one has not yet been created.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_migrate_v1_to_v2">migrate_v1_to_v2</a>&lt;T: key&gt;(registry: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_DisplayRegistry">rtd::display_registry::DisplayRegistry</a>, legacy: <a href="../rtd/display.md#rtd_display_Display">rtd::display::Display</a>&lt;T&gt;, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): (<a href="../rtd/display_registry.md#rtd_display_registry_Display">rtd::display_registry::Display</a>&lt;T&gt;, <a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">rtd::display_registry::DisplayCap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_migrate_v1_to_v2">migrate_v1_to_v2</a>&lt;T: key&gt;(
    registry: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_DisplayRegistry">DisplayRegistry</a>,
    legacy: LegacyDisplay&lt;T&gt;,
    ctx: &<b>mut</b> TxContext,
): (<a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a>&lt;T&gt;, <a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">DisplayCap</a>&lt;T&gt;) {
    <b>let</b> (<b>mut</b> <a href="../rtd/display.md#rtd_display">display</a>, cap) = <a href="../rtd/display_registry.md#rtd_display_registry_new_display">new_display</a>&lt;T&gt;(registry, ctx);
    <a href="../rtd/display.md#rtd_display">display</a>.<a href="../rtd/display_registry.md#rtd_display_registry_fields">fields</a> = *legacy.<a href="../rtd/display_registry.md#rtd_display_registry_fields">fields</a>();
    legacy.destroy();
    (<a href="../rtd/display.md#rtd_display">display</a>, cap)
}
</code></pre>



</details>

<a name="rtd_display_registry_destroy_system_migration_cap"></a>

## Function `destroy_system_migration_cap`

Destroy the <code><a href="../rtd/display_registry.md#rtd_display_registry_SystemMigrationCap">SystemMigrationCap</a></code> after successfully migrating all V1 instances.


<pre><code><b>entry</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_destroy_system_migration_cap">destroy_system_migration_cap</a>(cap: <a href="../rtd/display_registry.md#rtd_display_registry_SystemMigrationCap">rtd::display_registry::SystemMigrationCap</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_destroy_system_migration_cap">destroy_system_migration_cap</a>(cap: <a href="../rtd/display_registry.md#rtd_display_registry_SystemMigrationCap">SystemMigrationCap</a>) {
    <b>let</b> <a href="../rtd/display_registry.md#rtd_display_registry_SystemMigrationCap">SystemMigrationCap</a> { id } = cap;
    id.delete();
}
</code></pre>



</details>

<a name="rtd_display_registry_transfer_migration_cap"></a>

## Function `transfer_migration_cap`



<pre><code><b>entry</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_transfer_migration_cap">transfer_migration_cap</a>(cap: <a href="../rtd/display_registry.md#rtd_display_registry_SystemMigrationCap">rtd::display_registry::SystemMigrationCap</a>, recipient: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_transfer_migration_cap">transfer_migration_cap</a>(cap: <a href="../rtd/display_registry.md#rtd_display_registry_SystemMigrationCap">SystemMigrationCap</a>, recipient: <b>address</b>) {
    <a href="../rtd/transfer.md#rtd_transfer_transfer">transfer::transfer</a>(cap, recipient);
}
</code></pre>



</details>

<a name="rtd_display_registry_delete_legacy"></a>

## Function `delete_legacy`

Allow deleting legacy display objects, as long as the cap has been claimed first.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_delete_legacy">delete_legacy</a>&lt;T: key&gt;(<a href="../rtd/display.md#rtd_display">display</a>: &<a href="../rtd/display_registry.md#rtd_display_registry_Display">rtd::display_registry::Display</a>&lt;T&gt;, legacy: <a href="../rtd/display.md#rtd_display_Display">rtd::display::Display</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_delete_legacy">delete_legacy</a>&lt;T: key&gt;(<a href="../rtd/display.md#rtd_display">display</a>: &<a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a>&lt;T&gt;, legacy: LegacyDisplay&lt;T&gt;) {
    <b>assert</b>!(<a href="../rtd/display.md#rtd_display">display</a>.<a href="../rtd/display_registry.md#rtd_display_registry_cap_id">cap_id</a>.is_some(), <a href="../rtd/display_registry.md#rtd_display_registry_ECapNotClaimed">ECapNotClaimed</a>);
    legacy.destroy();
}
</code></pre>



</details>

<a name="rtd_display_registry_fields"></a>

## Function `fields`

Get a reference to the fields of display.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_fields">fields</a>&lt;T&gt;(<a href="../rtd/display.md#rtd_display">display</a>: &<a href="../rtd/display_registry.md#rtd_display_registry_Display">rtd::display_registry::Display</a>&lt;T&gt;): &<a href="../rtd/vec_map.md#rtd_vec_map_VecMap">rtd::vec_map::VecMap</a>&lt;<a href="../std/string.md#std_string_String">std::string::String</a>, <a href="../std/string.md#std_string_String">std::string::String</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_fields">fields</a>&lt;T&gt;(<a href="../rtd/display.md#rtd_display">display</a>: &<a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a>&lt;T&gt;): &VecMap&lt;String, String&gt; {
    &<a href="../rtd/display.md#rtd_display">display</a>.<a href="../rtd/display_registry.md#rtd_display_registry_fields">fields</a>
}
</code></pre>



</details>

<a name="rtd_display_registry_cap_id"></a>

## Function `cap_id`

Get the cap ID for the display.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_cap_id">cap_id</a>&lt;T&gt;(<a href="../rtd/display.md#rtd_display">display</a>: &<a href="../rtd/display_registry.md#rtd_display_registry_Display">rtd::display_registry::Display</a>&lt;T&gt;): <a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;<a href="../rtd/object.md#rtd_object_ID">rtd::object::ID</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_cap_id">cap_id</a>&lt;T&gt;(<a href="../rtd/display.md#rtd_display">display</a>: &<a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a>&lt;T&gt;): Option&lt;ID&gt; {
    <a href="../rtd/display.md#rtd_display">display</a>.<a href="../rtd/display_registry.md#rtd_display_registry_cap_id">cap_id</a>
}
</code></pre>



</details>

<a name="rtd_display_registry_migration_cap_receiver"></a>

## Function `migration_cap_receiver`



<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_migration_cap_receiver">migration_cap_receiver</a>(): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<a href="../rtd/package.md#rtd_package">package</a>) <b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_migration_cap_receiver">migration_cap_receiver</a>(): <b>address</b> {
    <a href="../rtd/display_registry.md#rtd_display_registry_SYSTEM_MIGRATION_ADDRESS">SYSTEM_MIGRATION_ADDRESS</a>
}
</code></pre>



</details>

<a name="rtd_display_registry_new_display"></a>

## Function `new_display`



<pre><code><b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_new_display">new_display</a>&lt;T&gt;(registry: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_DisplayRegistry">rtd::display_registry::DisplayRegistry</a>, ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): (<a href="../rtd/display_registry.md#rtd_display_registry_Display">rtd::display_registry::Display</a>&lt;T&gt;, <a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">rtd::display_registry::DisplayCap</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_new_display">new_display</a>&lt;T&gt;(
    registry: &<b>mut</b> <a href="../rtd/display_registry.md#rtd_display_registry_DisplayRegistry">DisplayRegistry</a>,
    ctx: &<b>mut</b> TxContext,
): (<a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a>&lt;T&gt;, <a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">DisplayCap</a>&lt;T&gt;) {
    <b>let</b> key = <a href="../rtd/display_registry.md#rtd_display_registry_DisplayKey">DisplayKey</a>&lt;T&gt;();
    <b>assert</b>!(!<a href="../rtd/derived_object.md#rtd_derived_object_exists">derived_object::exists</a>(&registry.id, key), <a href="../rtd/display_registry.md#rtd_display_registry_EDisplayAlreadyExists">EDisplayAlreadyExists</a>);
    <b>let</b> cap = <a href="../rtd/display_registry.md#rtd_display_registry_DisplayCap">DisplayCap</a>&lt;T&gt; { id: <a href="../rtd/object.md#rtd_object_new">object::new</a>(ctx) };
    <b>let</b> <a href="../rtd/display.md#rtd_display">display</a> = <a href="../rtd/display_registry.md#rtd_display_registry_Display">Display</a>&lt;T&gt; {
        id: <a href="../rtd/derived_object.md#rtd_derived_object_claim">derived_object::claim</a>(&<b>mut</b> registry.id, key),
        <a href="../rtd/display_registry.md#rtd_display_registry_fields">fields</a>: <a href="../rtd/vec_map.md#rtd_vec_map_empty">vec_map::empty</a>(),
        <a href="../rtd/display_registry.md#rtd_display_registry_cap_id">cap_id</a>: option::some(cap.id.to_inner()),
    };
    (<a href="../rtd/display.md#rtd_display">display</a>, cap)
}
</code></pre>



</details>

<a name="rtd_display_registry_create"></a>

## Function `create`

Create a new display registry object callable only from 0x0 (end of epoch)


<pre><code><b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_create">create</a>(ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../rtd/display_registry.md#rtd_display_registry_create">create</a>(ctx: &<b>mut</b> TxContext) {
    <b>assert</b>!(ctx.sender() == @0x0, <a href="../rtd/display_registry.md#rtd_display_registry_ENotSystemAddress">ENotSystemAddress</a>);
    <a href="../rtd/transfer.md#rtd_transfer_share_object">transfer::share_object</a>(<a href="../rtd/display_registry.md#rtd_display_registry_DisplayRegistry">DisplayRegistry</a> {
        id: <a href="../rtd/object.md#rtd_object_rtd_display_registry_object_id">object::rtd_display_registry_object_id</a>(),
    });
    <a href="../rtd/transfer.md#rtd_transfer_transfer">transfer::transfer</a>(
        <a href="../rtd/display_registry.md#rtd_display_registry_SystemMigrationCap">SystemMigrationCap</a> { id: <a href="../rtd/object.md#rtd_object_new">object::new</a>(ctx) },
        <a href="../rtd/display_registry.md#rtd_display_registry_SYSTEM_MIGRATION_ADDRESS">SYSTEM_MIGRATION_ADDRESS</a>,
    );
}
</code></pre>



</details>
