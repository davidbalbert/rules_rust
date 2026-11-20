//! A C ABI shared library that depends on a Rust ABI dylib.

#[no_mangle]
pub extern "C" fn cdylib_with_dylib_dep_value() -> u32 {
    dylib::dylib_value() + 1
}
