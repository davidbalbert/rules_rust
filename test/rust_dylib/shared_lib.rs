//! A C ABI shared library that depends on a Rust ABI dylib.

#[no_mangle]
pub extern "C" fn shared_lib_fn() -> u32 {
    rust_dylib_lib::example_test_dep_fn() + 1
}
