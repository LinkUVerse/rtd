---
title: Module `rtd::linked_table`
---

Similar to <code><a href="../rtd/table.md#rtd_table">rtd::table</a></code> but the values are linked together, allowing for ordered insertion and
removal


-  [Struct `LinkedTable`](#rtd_linked_table_LinkedTable)
-  [Struct `Node`](#rtd_linked_table_Node)
-  [Constants](#@Constants_0)
-  [Function `new`](#rtd_linked_table_new)
-  [Function `front`](#rtd_linked_table_front)
-  [Function `back`](#rtd_linked_table_back)
-  [Function `push_front`](#rtd_linked_table_push_front)
-  [Function `push_back`](#rtd_linked_table_push_back)
-  [Function `borrow`](#rtd_linked_table_borrow)
-  [Function `borrow_mut`](#rtd_linked_table_borrow_mut)
-  [Function `prev`](#rtd_linked_table_prev)
-  [Function `next`](#rtd_linked_table_next)
-  [Function `remove`](#rtd_linked_table_remove)
-  [Function `pop_front`](#rtd_linked_table_pop_front)
-  [Function `pop_back`](#rtd_linked_table_pop_back)
-  [Function `contains`](#rtd_linked_table_contains)
-  [Function `length`](#rtd_linked_table_length)
-  [Function `is_empty`](#rtd_linked_table_is_empty)
-  [Function `destroy_empty`](#rtd_linked_table_destroy_empty)
-  [Function `drop`](#rtd_linked_table_drop)


<pre><code><b>use</b> <a href="../rtd/address.md#rtd_address">rtd::address</a>;
<b>use</b> <a href="../rtd/dynamic_field.md#rtd_dynamic_field">rtd::dynamic_field</a>;
<b>use</b> <a href="../rtd/hex.md#rtd_hex">rtd::hex</a>;
<b>use</b> <a href="../rtd/object.md#rtd_object">rtd::object</a>;
<b>use</b> <a href="../rtd/tx_context.md#rtd_tx_context">rtd::tx_context</a>;
<b>use</b> <a href="../std/ascii.md#std_ascii">std::ascii</a>;
<b>use</b> <a href="../std/bcs.md#std_bcs">std::bcs</a>;
<b>use</b> <a href="../std/option.md#std_option">std::option</a>;
<b>use</b> <a href="../std/string.md#std_string">std::string</a>;
<b>use</b> <a href="../std/vector.md#std_vector">std::vector</a>;
</code></pre>



<a name="rtd_linked_table_LinkedTable"></a>

## Struct `LinkedTable`



<pre><code><b>public</b> <b>struct</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, <b>phantom</b> V: store&gt; <b>has</b> key, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="../rtd/object.md#rtd_object_UID">rtd::object::UID</a></code>
</dt>
<dd>
 the ID of this table
</dd>
<dt>
<code>size: u64</code>
</dt>
<dd>
 the number of key-value pairs in the table
</dd>
<dt>
<code>head: <a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;K&gt;</code>
</dt>
<dd>
 the front of the table, i.e. the key of the first entry
</dd>
<dt>
<code>tail: <a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;K&gt;</code>
</dt>
<dd>
 the back of the table, i.e. the key of the last entry
</dd>
</dl>


</details>

<a name="rtd_linked_table_Node"></a>

## Struct `Node`



<pre><code><b>public</b> <b>struct</b> <a href="../rtd/linked_table.md#rtd_linked_table_Node">Node</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt; <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="../rtd/linked_table.md#rtd_linked_table_prev">prev</a>: <a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;K&gt;</code>
</dt>
<dd>
 the previous key
</dd>
<dt>
<code><a href="../rtd/linked_table.md#rtd_linked_table_next">next</a>: <a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;K&gt;</code>
</dt>
<dd>
 the next key
</dd>
<dt>
<code>value: V</code>
</dt>
<dd>
 the value being stored
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="rtd_linked_table_ETableNotEmpty"></a>



<pre><code><b>const</b> <a href="../rtd/linked_table.md#rtd_linked_table_ETableNotEmpty">ETableNotEmpty</a>: u64 = 0;
</code></pre>



<a name="rtd_linked_table_ETableIsEmpty"></a>



<pre><code><b>const</b> <a href="../rtd/linked_table.md#rtd_linked_table_ETableIsEmpty">ETableIsEmpty</a>: u64 = 1;
</code></pre>



<a name="rtd_linked_table_new"></a>

## Function `new`

Creates a new, empty table


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_new">new</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt;(ctx: &<b>mut</b> <a href="../rtd/tx_context.md#rtd_tx_context_TxContext">rtd::tx_context::TxContext</a>): <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_new">new</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: store&gt;(ctx: &<b>mut</b> TxContext): <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt; {
    <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a> {
        id: <a href="../rtd/object.md#rtd_object_new">object::new</a>(ctx),
        size: 0,
        head: option::none(),
        tail: option::none(),
    }
}
</code></pre>



</details>

<a name="rtd_linked_table_front"></a>

## Function `front`

Returns the key for the first element in the table, or None if the table is empty


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_front">front</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;): &<a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;K&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_front">front</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;): &Option&lt;K&gt; {
    &<a href="../rtd/table.md#rtd_table">table</a>.head
}
</code></pre>



</details>

<a name="rtd_linked_table_back"></a>

## Function `back`

Returns the key for the last element in the table, or None if the table is empty


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_back">back</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;): &<a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;K&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_back">back</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;): &Option&lt;K&gt; {
    &<a href="../rtd/table.md#rtd_table">table</a>.tail
}
</code></pre>



</details>

<a name="rtd_linked_table_push_front"></a>

## Function `push_front`

Inserts a key-value pair at the front of the table, i.e. the newly inserted pair will be
the first element in the table
Aborts with <code><a href="../rtd/dynamic_field.md#rtd_dynamic_field_EFieldAlreadyExists">rtd::dynamic_field::EFieldAlreadyExists</a></code> if the table already has an entry with
that key <code>k: K</code>.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_push_front">push_front</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<b>mut</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;, k: K, value: V)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_push_front">push_front</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: store&gt;(
    <a href="../rtd/table.md#rtd_table">table</a>: &<b>mut</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;,
    k: K,
    value: V,
) {
    <b>let</b> old_head = <a href="../rtd/table.md#rtd_table">table</a>.head.swap_or_fill(k);
    <b>if</b> (<a href="../rtd/table.md#rtd_table">table</a>.tail.is_none()) <a href="../rtd/table.md#rtd_table">table</a>.tail.fill(k);
    <b>let</b> <a href="../rtd/linked_table.md#rtd_linked_table_prev">prev</a> = option::none();
    <b>let</b> <a href="../rtd/linked_table.md#rtd_linked_table_next">next</a> = <b>if</b> (old_head.is_some()) {
        <b>let</b> old_head_k = old_head.destroy_some();
        field::borrow_mut&lt;K, <a href="../rtd/linked_table.md#rtd_linked_table_Node">Node</a>&lt;K, V&gt;&gt;(&<b>mut</b> <a href="../rtd/table.md#rtd_table">table</a>.id, old_head_k).<a href="../rtd/linked_table.md#rtd_linked_table_prev">prev</a> = option::some(k);
        option::some(old_head_k)
    } <b>else</b> {
        option::none()
    };
    field::add(&<b>mut</b> <a href="../rtd/table.md#rtd_table">table</a>.id, k, <a href="../rtd/linked_table.md#rtd_linked_table_Node">Node</a> { <a href="../rtd/linked_table.md#rtd_linked_table_prev">prev</a>, <a href="../rtd/linked_table.md#rtd_linked_table_next">next</a>, value });
    <a href="../rtd/table.md#rtd_table">table</a>.size = <a href="../rtd/table.md#rtd_table">table</a>.size + 1;
}
</code></pre>



</details>

<a name="rtd_linked_table_push_back"></a>

## Function `push_back`

Inserts a key-value pair at the back of the table, i.e. the newly inserted pair will be
the last element in the table
Aborts with <code><a href="../rtd/dynamic_field.md#rtd_dynamic_field_EFieldAlreadyExists">rtd::dynamic_field::EFieldAlreadyExists</a></code> if the table already has an entry with
that key <code>k: K</code>.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_push_back">push_back</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<b>mut</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;, k: K, value: V)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_push_back">push_back</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: store&gt;(
    <a href="../rtd/table.md#rtd_table">table</a>: &<b>mut</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;,
    k: K,
    value: V,
) {
    <b>if</b> (<a href="../rtd/table.md#rtd_table">table</a>.head.is_none()) <a href="../rtd/table.md#rtd_table">table</a>.head.fill(k);
    <b>let</b> old_tail = <a href="../rtd/table.md#rtd_table">table</a>.tail.swap_or_fill(k);
    <b>let</b> <a href="../rtd/linked_table.md#rtd_linked_table_prev">prev</a> = <b>if</b> (old_tail.is_some()) {
        <b>let</b> old_tail_k = old_tail.destroy_some();
        field::borrow_mut&lt;K, <a href="../rtd/linked_table.md#rtd_linked_table_Node">Node</a>&lt;K, V&gt;&gt;(&<b>mut</b> <a href="../rtd/table.md#rtd_table">table</a>.id, old_tail_k).<a href="../rtd/linked_table.md#rtd_linked_table_next">next</a> = option::some(k);
        option::some(old_tail_k)
    } <b>else</b> {
        option::none()
    };
    <b>let</b> <a href="../rtd/linked_table.md#rtd_linked_table_next">next</a> = option::none();
    field::add(&<b>mut</b> <a href="../rtd/table.md#rtd_table">table</a>.id, k, <a href="../rtd/linked_table.md#rtd_linked_table_Node">Node</a> { <a href="../rtd/linked_table.md#rtd_linked_table_prev">prev</a>, <a href="../rtd/linked_table.md#rtd_linked_table_next">next</a>, value });
    <a href="../rtd/table.md#rtd_table">table</a>.size = <a href="../rtd/table.md#rtd_table">table</a>.size + 1;
}
</code></pre>



</details>

<a name="rtd_linked_table_borrow"></a>

## Function `borrow`

Immutable borrows the value associated with the key in the table <code><a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;</code>.
Aborts with <code><a href="../rtd/dynamic_field.md#rtd_dynamic_field_EFieldDoesNotExist">rtd::dynamic_field::EFieldDoesNotExist</a></code> if the table does not have an entry with
that key <code>k: K</code>.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/borrow.md#rtd_borrow">borrow</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;, k: K): &V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/borrow.md#rtd_borrow">borrow</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;, k: K): &V {
    &field::borrow&lt;K, <a href="../rtd/linked_table.md#rtd_linked_table_Node">Node</a>&lt;K, V&gt;&gt;(&<a href="../rtd/table.md#rtd_table">table</a>.id, k).value
}
</code></pre>



</details>

<a name="rtd_linked_table_borrow_mut"></a>

## Function `borrow_mut`

Mutably borrows the value associated with the key in the table <code><a href="../rtd/table.md#rtd_table">table</a>: &<b>mut</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;</code>.
Aborts with <code><a href="../rtd/dynamic_field.md#rtd_dynamic_field_EFieldDoesNotExist">rtd::dynamic_field::EFieldDoesNotExist</a></code> if the table does not have an entry with
that key <code>k: K</code>.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_borrow_mut">borrow_mut</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<b>mut</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;, k: K): &<b>mut</b> V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_borrow_mut">borrow_mut</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: store&gt;(
    <a href="../rtd/table.md#rtd_table">table</a>: &<b>mut</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;,
    k: K,
): &<b>mut</b> V {
    &<b>mut</b> field::borrow_mut&lt;K, <a href="../rtd/linked_table.md#rtd_linked_table_Node">Node</a>&lt;K, V&gt;&gt;(&<b>mut</b> <a href="../rtd/table.md#rtd_table">table</a>.id, k).value
}
</code></pre>



</details>

<a name="rtd_linked_table_prev"></a>

## Function `prev`

Borrows the key for the previous entry of the specified key <code>k: K</code> in the table
<code><a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;</code>. Returns None if the entry does not have a predecessor.
Aborts with <code><a href="../rtd/dynamic_field.md#rtd_dynamic_field_EFieldDoesNotExist">rtd::dynamic_field::EFieldDoesNotExist</a></code> if the table does not have an entry with
that key <code>k: K</code>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_prev">prev</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;, k: K): &<a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;K&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_prev">prev</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;, k: K): &Option&lt;K&gt; {
    &field::borrow&lt;K, <a href="../rtd/linked_table.md#rtd_linked_table_Node">Node</a>&lt;K, V&gt;&gt;(&<a href="../rtd/table.md#rtd_table">table</a>.id, k).<a href="../rtd/linked_table.md#rtd_linked_table_prev">prev</a>
}
</code></pre>



</details>

<a name="rtd_linked_table_next"></a>

## Function `next`

Borrows the key for the next entry of the specified key <code>k: K</code> in the table
<code><a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;</code>. Returns None if the entry does not have a predecessor.
Aborts with <code><a href="../rtd/dynamic_field.md#rtd_dynamic_field_EFieldDoesNotExist">rtd::dynamic_field::EFieldDoesNotExist</a></code> if the table does not have an entry with
that key <code>k: K</code>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_next">next</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;, k: K): &<a href="../std/option.md#std_option_Option">std::option::Option</a>&lt;K&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_next">next</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;, k: K): &Option&lt;K&gt; {
    &field::borrow&lt;K, <a href="../rtd/linked_table.md#rtd_linked_table_Node">Node</a>&lt;K, V&gt;&gt;(&<a href="../rtd/table.md#rtd_table">table</a>.id, k).<a href="../rtd/linked_table.md#rtd_linked_table_next">next</a>
}
</code></pre>



</details>

<a name="rtd_linked_table_remove"></a>

## Function `remove`

Removes the key-value pair in the table <code><a href="../rtd/table.md#rtd_table">table</a>: &<b>mut</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;</code> and returns the value.
This splices the element out of the ordering.
Aborts with <code><a href="../rtd/dynamic_field.md#rtd_dynamic_field_EFieldDoesNotExist">rtd::dynamic_field::EFieldDoesNotExist</a></code> if the table does not have an entry with
that key <code>k: K</code>. Note: this is also what happens when the table is empty.


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_remove">remove</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<b>mut</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;, k: K): V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_remove">remove</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<b>mut</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;, k: K): V {
    <b>let</b> <a href="../rtd/linked_table.md#rtd_linked_table_Node">Node</a>&lt;K, V&gt; { <a href="../rtd/linked_table.md#rtd_linked_table_prev">prev</a>, <a href="../rtd/linked_table.md#rtd_linked_table_next">next</a>, value } = field::remove(&<b>mut</b> <a href="../rtd/table.md#rtd_table">table</a>.id, k);
    <a href="../rtd/table.md#rtd_table">table</a>.size = <a href="../rtd/table.md#rtd_table">table</a>.size - 1;
    <b>if</b> (<a href="../rtd/linked_table.md#rtd_linked_table_prev">prev</a>.is_some()) {
        field::borrow_mut&lt;K, <a href="../rtd/linked_table.md#rtd_linked_table_Node">Node</a>&lt;K, V&gt;&gt;(&<b>mut</b> <a href="../rtd/table.md#rtd_table">table</a>.id, *<a href="../rtd/linked_table.md#rtd_linked_table_prev">prev</a>.<a href="../rtd/borrow.md#rtd_borrow">borrow</a>()).<a href="../rtd/linked_table.md#rtd_linked_table_next">next</a> = <a href="../rtd/linked_table.md#rtd_linked_table_next">next</a>
    };
    <b>if</b> (<a href="../rtd/linked_table.md#rtd_linked_table_next">next</a>.is_some()) {
        field::borrow_mut&lt;K, <a href="../rtd/linked_table.md#rtd_linked_table_Node">Node</a>&lt;K, V&gt;&gt;(&<b>mut</b> <a href="../rtd/table.md#rtd_table">table</a>.id, *<a href="../rtd/linked_table.md#rtd_linked_table_next">next</a>.<a href="../rtd/borrow.md#rtd_borrow">borrow</a>()).<a href="../rtd/linked_table.md#rtd_linked_table_prev">prev</a> = <a href="../rtd/linked_table.md#rtd_linked_table_prev">prev</a>
    };
    <b>if</b> (<a href="../rtd/table.md#rtd_table">table</a>.head.<a href="../rtd/borrow.md#rtd_borrow">borrow</a>() == &k) <a href="../rtd/table.md#rtd_table">table</a>.head = <a href="../rtd/linked_table.md#rtd_linked_table_next">next</a>;
    <b>if</b> (<a href="../rtd/table.md#rtd_table">table</a>.tail.<a href="../rtd/borrow.md#rtd_borrow">borrow</a>() == &k) <a href="../rtd/table.md#rtd_table">table</a>.tail = <a href="../rtd/linked_table.md#rtd_linked_table_prev">prev</a>;
    value
}
</code></pre>



</details>

<a name="rtd_linked_table_pop_front"></a>

## Function `pop_front`

Removes the front of the table <code><a href="../rtd/table.md#rtd_table">table</a>: &<b>mut</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;</code>, returns the key and value.
Aborts with <code><a href="../rtd/linked_table.md#rtd_linked_table_ETableIsEmpty">ETableIsEmpty</a></code> if the table is empty


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_pop_front">pop_front</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<b>mut</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;): (K, V)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_pop_front">pop_front</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<b>mut</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;): (K, V) {
    <b>assert</b>!(<a href="../rtd/table.md#rtd_table">table</a>.head.is_some(), <a href="../rtd/linked_table.md#rtd_linked_table_ETableIsEmpty">ETableIsEmpty</a>);
    <b>let</b> head = *<a href="../rtd/table.md#rtd_table">table</a>.head.<a href="../rtd/borrow.md#rtd_borrow">borrow</a>();
    (head, <a href="../rtd/table.md#rtd_table">table</a>.<a href="../rtd/linked_table.md#rtd_linked_table_remove">remove</a>(head))
}
</code></pre>



</details>

<a name="rtd_linked_table_pop_back"></a>

## Function `pop_back`

Removes the back of the table <code><a href="../rtd/table.md#rtd_table">table</a>: &<b>mut</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;</code>, returns the key and value.
Aborts with <code><a href="../rtd/linked_table.md#rtd_linked_table_ETableIsEmpty">ETableIsEmpty</a></code> if the table is empty


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_pop_back">pop_back</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<b>mut</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;): (K, V)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_pop_back">pop_back</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<b>mut</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;): (K, V) {
    <b>assert</b>!(<a href="../rtd/table.md#rtd_table">table</a>.tail.is_some(), <a href="../rtd/linked_table.md#rtd_linked_table_ETableIsEmpty">ETableIsEmpty</a>);
    <b>let</b> tail = *<a href="../rtd/table.md#rtd_table">table</a>.tail.<a href="../rtd/borrow.md#rtd_borrow">borrow</a>();
    (tail, <a href="../rtd/table.md#rtd_table">table</a>.<a href="../rtd/linked_table.md#rtd_linked_table_remove">remove</a>(tail))
}
</code></pre>



</details>

<a name="rtd_linked_table_contains"></a>

## Function `contains`

Returns true iff there is a value associated with the key <code>k: K</code> in table
<code><a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;</code>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_contains">contains</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;, k: K): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_contains">contains</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;, k: K): bool {
    field::exists_with_type&lt;K, <a href="../rtd/linked_table.md#rtd_linked_table_Node">Node</a>&lt;K, V&gt;&gt;(&<a href="../rtd/table.md#rtd_table">table</a>.id, k)
}
</code></pre>



</details>

<a name="rtd_linked_table_length"></a>

## Function `length`

Returns the size of the table, the number of key-value pairs


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_length">length</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_length">length</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;): u64 {
    <a href="../rtd/table.md#rtd_table">table</a>.size
}
</code></pre>



</details>

<a name="rtd_linked_table_is_empty"></a>

## Function `is_empty`

Returns true iff the table is empty (if <code><a href="../rtd/linked_table.md#rtd_linked_table_length">length</a></code> returns <code>0</code>)


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_is_empty">is_empty</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_is_empty">is_empty</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: &<a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;): bool {
    <a href="../rtd/table.md#rtd_table">table</a>.size == 0
}
</code></pre>



</details>

<a name="rtd_linked_table_destroy_empty"></a>

## Function `destroy_empty`

Destroys an empty table
Aborts with <code><a href="../rtd/linked_table.md#rtd_linked_table_ETableNotEmpty">ETableNotEmpty</a></code> if the table still contains values


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_destroy_empty">destroy_empty</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_destroy_empty">destroy_empty</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;) {
    <b>let</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a> { id, size, head: _, tail: _ } = <a href="../rtd/table.md#rtd_table">table</a>;
    <b>assert</b>!(size == 0, <a href="../rtd/linked_table.md#rtd_linked_table_ETableNotEmpty">ETableNotEmpty</a>);
    id.delete()
}
</code></pre>



</details>

<a name="rtd_linked_table_drop"></a>

## Function `drop`

Drop a possibly non-empty table.
Usable only if the value type <code>V</code> has the <code><a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a></code> ability


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>&lt;K: <b>copy</b>, <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store, V: <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>, store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">rtd::linked_table::LinkedTable</a>&lt;K, V&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a>&lt;K: <b>copy</b> + <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store, V: <a href="../rtd/linked_table.md#rtd_linked_table_drop">drop</a> + store&gt;(<a href="../rtd/table.md#rtd_table">table</a>: <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a>&lt;K, V&gt;) {
    <b>let</b> <a href="../rtd/linked_table.md#rtd_linked_table_LinkedTable">LinkedTable</a> { id, size: _, head: _, tail: _ } = <a href="../rtd/table.md#rtd_table">table</a>;
    id.delete()
}
</code></pre>



</details>
