module a::b;

fun f() {
    let x = rtd::dynamic_field::borrow<vector<u8>, u64>(&parent, b"");
    let x = ::rtd::dynamic_field::borrow<vector<u8>, u64>(&parent, b"");
}
