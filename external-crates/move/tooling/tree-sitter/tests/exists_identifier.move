module a::m;

use rtd::dynamic_field::exists;

fun f(id: &UID) {
    assert!(exists<vector<u8>>(id, b""), 0);
    assert!(exists<u64>(id, 0u64), 0);
}
