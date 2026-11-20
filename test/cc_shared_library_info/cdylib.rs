//! A C ABI shared library.

#[no_mangle]
pub extern "C" fn cdylib_value() -> u32 {
    1
}
